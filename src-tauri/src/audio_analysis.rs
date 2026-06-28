/// Módulo: audio_analysis.rs
/// Propósito: análisis de un archivo en UNA pasada de decodificación (en hilo de
/// comando, NUNCA en el hilo de audio): pico real (dBFS), loudness integrado
/// (LUFS, crate ebur128) y envolvente de onda. Reutiliza el decodificador
/// existente (audio_decode) para soportar los mismos formatos, incluido Opus.
use crate::audio_decode;
use crate::cached_source::CachedPcm;
use crate::types_norm::NormConfig;
use crate::types_track::TrackMeta;
use crate::waveform::WaveEnvelope;
use ebur128::{EbuR128, Mode};
use rodio::Source;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Tope de puntos del envolvente (acota memoria en archivos largos).
const MAX_ENVELOPE_POINTS: usize = 120_000;

/// Resultado del análisis: metadatos para la DB + envolvente para dibujar + el
/// PCM decodificado (i16) para cachearlo y que la previa del editor tenga seek
/// O(1) (scrubbing instantáneo, sin re-decodificar).
pub struct AnalysisResult {
    pub meta: TrackMeta,
    pub envelope: WaveEnvelope,
    pub pcm: CachedPcm,
}

/// Decodifica el archivo y calcula pico, LUFS, ganancia sugerida y envolvente.
pub fn analyze(path: &str, norm: &NormConfig) -> Result<AnalysisResult, String> {
    let src = audio_decode::source_from_path(path, false).ok_or("unsupported_audio_format")?;
    let channels = src.channels().max(1);
    let rate = src.sample_rate().max(1);
    let samples: Vec<f32> = src.collect();
    if samples.is_empty() {
        return Err("empty_audio".to_string());
    }

    let frames = samples.len() / channels as usize;
    let duration_s = frames as f64 / rate as f64;
    let peak = samples.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
    let peak_db = if peak > 0.0 {
        20.0 * (peak as f64).log10()
    } else {
        -120.0
    };
    let lufs = measure_lufs(&samples, channels, rate);
    let suggested = suggest_gain(lufs, peak_db, norm);
    let envelope = WaveEnvelope::build(&samples, channels, rate, MAX_ENVELOPE_POINTS);

    let (mtime, size) = file_stamp(path);
    let mut meta = TrackMeta::new(path.to_string(), mtime, size, duration_s, rate, channels);
    meta.measured_peak_db = Some(peak_db);
    meta.measured_lufs = lufs;
    meta.norm_gain_db = suggested;
    meta.analyzed_at = Some(now_epoch());

    // Reutiliza la decodificación: convierte a i16 para cachear (consume samples).
    let pcm = CachedPcm {
        data: samples
            .into_iter()
            .map(|s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
            .collect(),
        channels,
        sample_rate: rate,
    };

    Ok(AnalysisResult { meta, envelope, pcm })
}

/// Mide el loudness integrado (LUFS). None si el audio es silencio o no medible.
fn measure_lufs(interleaved: &[f32], channels: u16, rate: u32) -> Option<f64> {
    let mut ebu = EbuR128::new(channels as u32, rate, Mode::I).ok()?;
    ebu.add_frames_f32(interleaved).ok()?;
    let l = ebu.loudness_global().ok()?;
    if l.is_finite() && l > -70.0 {
        Some(l)
    } else {
        None
    }
}

/// Ganancia (dB) para llegar al objetivo según el modo de normalización.
/// - "lufs": ajusta por loudness integrado, limitado por el techo de pico.
/// - "peak": ajusta al pico máximo directamente.
fn suggest_gain(lufs: Option<f64>, peak_db: f64, norm: &NormConfig) -> f64 {
    if norm.mode == "peak" {
        return (norm.target - peak_db).clamp(-24.0, 24.0);
    }
    // Modo LUFS (defecto)
    match lufs {
        Some(l) => {
            let by_loudness = norm.target - l;
            let by_peak = norm.ceiling_db - peak_db;
            by_loudness.min(by_peak).clamp(-24.0, 24.0)
        }
        None => 0.0,
    }
}

/// Fecha de modificación (epoch s) y tamaño del archivo, para invalidar caché.
pub fn file_stamp(path: &str) -> (i64, i64) {
    match fs::metadata(path) {
        Ok(m) => {
            let mtime = m
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            (mtime, m.len() as i64)
        }
        Err(_) => (0, 0),
    }
}

fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norm_lufs() -> NormConfig {
        NormConfig { mode: "lufs".to_string(), target: -14.0, ceiling_db: -1.0 }
    }
    fn norm_peak() -> NormConfig {
        NormConfig { mode: "peak".to_string(), target: -1.0, ceiling_db: -1.0 }
    }

    #[test]
    fn gain_respects_peak_ceiling() {
        // LUFS muy bajo pediría +20 dB, pero el pico a -2 dBFS solo permite +1.
        let g = suggest_gain(Some(-34.0), -2.0, &norm_lufs());
        assert!((g - 1.0).abs() < 1e-9);
    }

    #[test]
    fn silence_yields_zero_gain() {
        assert_eq!(suggest_gain(None, -120.0, &norm_lufs()), 0.0);
    }

    #[test]
    fn loud_track_gets_attenuated() {
        let g = suggest_gain(Some(-8.0), -1.0, &norm_lufs());
        assert!((g - (-6.0)).abs() < 1e-9);
    }

    #[test]
    fn peak_mode_uses_peak_target() {
        let g = suggest_gain(Some(-14.0), -6.0, &norm_peak());
        assert!((g - 5.0).abs() < 1e-9);
    }
}
