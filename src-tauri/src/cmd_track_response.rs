use crate::track_analysis_cache::CachedTrackAnalysis;
use crate::model::track::TrackMeta;
use serde::Serialize;

#[derive(Serialize)]
pub struct AnalyzeResponse {
    pub meta: TrackMeta,
    pub waveform: Vec<f32>,
    pub duration_s: f64,
    pub peak_db: Option<f64>,
    pub lufs: Option<f64>,
    pub suggested_norm_db: f64,
    pub detected_start_s: Option<f64>,
    pub detected_end_s: Option<f64>,
}

pub fn response_from(
    item: &CachedTrackAnalysis,
    meta: TrackMeta,
    buckets: usize,
    detected: (Option<f64>, Option<f64>),
) -> AnalyzeResponse {
    let duration_s = item.envelope.duration_s();
    AnalyzeResponse {
        peak_db: item.meta.measured_peak_db,
        lufs: item.meta.measured_lufs,
        suggested_norm_db: item.meta.norm_gain_db,
        waveform: item.envelope.view(0.0, duration_s, buckets),
        duration_s,
        detected_start_s: detected.0,
        detected_end_s: detected.1,
        meta,
    }
}
