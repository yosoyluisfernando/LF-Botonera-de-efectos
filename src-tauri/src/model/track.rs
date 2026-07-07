/// Módulo: types_track.rs
/// Propósito: esquema serializable de los metadatos de pista (cue, dB, LUFS…)
/// que se guardan por archivo en tracks.db. Compartido entre Rust y la UI.
/// La forma de onda NO vive aquí: se calcula al vuelo y se descarta (ver plan).
use serde::{Deserialize, Serialize};

/// Una fila de la tabla `track`: todo lo editado o medido de un archivo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackMeta {
    /// Ruta del archivo, ya normalizada como clave (ver db::normalize_key).
    pub path: String,
    /// Fecha de modificación del archivo (epoch); invalida la fila si cambia.
    pub mtime: i64,
    /// Tamaño en bytes; verificación secundaria de invalidación.
    pub size: i64,
    pub duration_s: f64,
    pub sample_rate: u32,
    pub channels: u16,
    /// Punto de inicio manual (cue), en segundos. Lo edita el usuario.
    #[serde(default)]
    pub cue_start_s: f64,
    /// Punto de fin (recorte); None = hasta el final del archivo.
    #[serde(default)]
    pub cue_end_s: Option<f64>,
    /// Trim manual del usuario, en dB.
    #[serde(default)]
    pub gain_db: f64,
    /// ¿Aplicar la normalización automática calculada?
    #[serde(default)]
    pub norm_enabled: bool,
    /// Ganancia (dB) que calcula el auto-normalizador para llegar al objetivo.
    #[serde(default)]
    pub norm_gain_db: f64,
    /// Pico real medido (dBFS).
    #[serde(default)]
    pub measured_peak_db: Option<f64>,
    /// Loudness integrado medido (LUFS).
    #[serde(default)]
    pub measured_lufs: Option<f64>,
    /// Epoch del último análisis.
    #[serde(default)]
    pub analyzed_at: Option<i64>,
    /// Epoch de la última reproducción (lo usará la precarga; ver §12 del plan).
    #[serde(default)]
    pub last_played: Option<i64>,
}

impl TrackMeta {
    /// Crea una fila recién analizada con cue/dB en valores neutros.
    pub fn new(
        path: String,
        mtime: i64,
        size: i64,
        duration_s: f64,
        sample_rate: u32,
        channels: u16,
    ) -> Self {
        Self {
            path,
            mtime,
            size,
            duration_s,
            sample_rate,
            channels,
            cue_start_s: 0.0,
            cue_end_s: None,
            gain_db: 0.0,
            norm_enabled: true,
            norm_gain_db: 0.0,
            measured_peak_db: None,
            measured_lufs: None,
            analyzed_at: None,
            last_played: None,
        }
    }

    /// Cue saneado contra la duración real del archivo: evita que un inicio
    /// fuera de rango (p.ej. tras reemplazar el archivo por uno más corto) deje
    /// el botón en silencio. Devuelve (inicio, fin) ya recortados a [0, dur].
    pub fn sanitized_cue(&self) -> (f64, Option<f64>) {
        let dur = self.duration_s.max(0.0);
        let start = self.cue_start_s.clamp(0.0, (dur - 0.01).max(0.0));
        let end = self
            .cue_end_s
            .filter(|&e| e > start && e <= dur + 0.001);
        (start, end)
    }

    /// Duración efectiva (lo que se oye) tras aplicar el cue saneado.
    pub fn effective_duration_s(&self) -> f64 {
        let (start, end) = self.sanitized_cue();
        (end.unwrap_or(self.duration_s) - start).max(0.0)
    }

    /// Ganancia total en dB al reproducir = normalización (si activa) + manual.
    pub fn effective_gain_db(&self) -> f64 {
        self.norm_gain_db + self.gain_db
    }

    /// Ganancia efectiva en multiplicador lineal (capa 1 del modelo de 3 capas).
    pub fn effective_gain_linear(&self) -> f32 {
        db_to_linear(self.effective_gain_db())
    }

    /// ¿Sigue siendo válida la fila para el archivo actual en disco?
    pub fn matches(&self, mtime: i64, size: i64) -> bool {
        self.mtime == mtime && self.size == size
    }
}

/// Convierte decibelios a multiplicador lineal: 10^(dB/20).
pub fn db_to_linear(db: f64) -> f32 {
    10f64.powf(db / 20.0) as f32
}

#[cfg(test)]
mod tests {
    use super::TrackMeta;

    fn meta(dur: f64, start: f64, end: Option<f64>) -> TrackMeta {
        let mut m = TrackMeta::new("x".into(), 0, 0, dur, 48000, 2);
        m.cue_start_s = start;
        m.cue_end_s = end;
        m
    }

    #[test]
    fn cue_within_range_is_kept() {
        let (s, e) = meta(10.0, 2.0, Some(8.0)).sanitized_cue();
        assert_eq!(s, 2.0);
        assert_eq!(e, Some(8.0));
    }

    #[test]
    fn start_beyond_end_is_clamped_not_silent() {
        // Inicio 30s en un archivo de 5s (reemplazado) → no deja sonido en silencio.
        let (s, e) = meta(5.0, 30.0, None).sanitized_cue();
        assert!(s < 5.0);
        assert_eq!(e, None);
        assert!(meta(5.0, 30.0, None).effective_duration_s() > 0.0);
    }

    #[test]
    fn end_before_start_is_discarded() {
        let (_, e) = meta(10.0, 6.0, Some(3.0)).sanitized_cue();
        assert_eq!(e, None);
    }
}
