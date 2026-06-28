/// Analisis DSP de pistas fuera del hilo de audio.
use crate::audio_decode;
use crate::cached_source::CachedPcm;
use crate::cue_detect;
use crate::types_norm::{CueDetectConfig, NormConfig};
use crate::types_track::TrackMeta;
use crate::waveform::WaveEnvelope;
use ebur128::{EbuR128, Mode};
use rodio::Source;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_ENVELOPE_POINTS: usize = 120_000;

pub struct AnalysisResult {
    pub meta: TrackMeta,
    pub envelope: WaveEnvelope,
    pub pcm: CachedPcm,
    pub auto_cue_start_s: Option<f64>,
    pub auto_cue_end_s: Option<f64>,
}

pub fn analyze(
    path: &str,
    norm: &NormConfig,
    cue: &CueDetectConfig,
) -> Result<AnalysisResult, String> {
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
    let pcm_data: Vec<i16> = samples
        .into_iter()
        .map(|s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
        .collect();
    let (auto_cue_start_s, auto_cue_end_s) =
        cue_detect::detect_boundaries(&pcm_data, rate, channels, cue);
    let pcm = CachedPcm {
        data: pcm_data,
        channels,
        sample_rate: rate,
    };
    Ok(AnalysisResult {
        meta,
        envelope,
        pcm,
        auto_cue_start_s,
        auto_cue_end_s,
    })
}

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

pub fn suggest_gain(lufs: Option<f64>, peak_db: f64, norm: &NormConfig) -> f64 {
    if norm.mode == "peak" {
        return (norm.target - peak_db).clamp(-24.0, 24.0);
    }
    match lufs {
        Some(l) => (norm.target - l)
            .min(norm.ceiling_db - peak_db)
            .clamp(-24.0, 24.0),
        None => 0.0,
    }
}

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
        NormConfig {
            mode: "lufs".to_string(),
            target: -14.0,
            ceiling_db: -1.0,
        }
    }
    fn norm_peak() -> NormConfig {
        NormConfig {
            mode: "peak".to_string(),
            target: -1.0,
            ceiling_db: -1.0,
        }
    }

    #[test]
    fn gain_respects_peak_ceiling() {
        assert!((suggest_gain(Some(-34.0), -2.0, &norm_lufs()) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn silence_yields_zero_gain() {
        assert_eq!(suggest_gain(None, -120.0, &norm_lufs()), 0.0);
    }

    #[test]
    fn loud_track_gets_attenuated() {
        assert!((suggest_gain(Some(-8.0), -1.0, &norm_lufs()) - (-6.0)).abs() < 1e-9);
    }

    #[test]
    fn peak_mode_uses_peak_target() {
        assert!((suggest_gain(Some(-14.0), -6.0, &norm_peak()) - 5.0).abs() < 1e-9);
    }
}
