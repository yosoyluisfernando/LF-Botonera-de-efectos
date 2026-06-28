/// Deteccion automatica de cue por pico en ventanas cortas.
use crate::types_norm::CueDetectConfig;

const WINDOW_S: f64 = 0.01;

pub fn detect_boundaries(
    pcm: &[i16],
    sample_rate: u32,
    channels: u16,
    cfg: &CueDetectConfig,
) -> (Option<f64>, Option<f64>) {
    if !cfg.enabled || pcm.is_empty() {
        return (None, None);
    }
    let frames = peak_frames(pcm, sample_rate, channels);
    let detected_start = detect_start(&frames, cfg.start_thresh_db, cfg.detect_start);
    let detected_end = detect_end(&frames, cfg.end_thresh_db, cfg.detect_end);
    (detected_start, detected_end)
}

fn peak_frames(pcm: &[i16], sample_rate: u32, channels: u16) -> Vec<f32> {
    let win = ((sample_rate as f64 * WINDOW_S) as usize).max(1) * (channels as usize).max(1);
    pcm.chunks(win)
        .map(|w| {
            w.iter()
                .map(|&s| (s as f32 / 32767.0).abs())
                .fold(0.0, f32::max)
        })
        .collect()
}

fn detect_start(frames: &[f32], thresh_db: f64, enabled: bool) -> Option<f64> {
    if !enabled {
        return None;
    }
    let th = 10f32.powf(thresh_db as f32 / 20.0);
    frames
        .windows(3)
        .position(|w| w.iter().all(|&p| p >= th))
        .map(|i| i as f64 * WINDOW_S)
        .filter(|&s| s > 0.0)
}

fn detect_end(frames: &[f32], thresh_db: f64, enabled: bool) -> Option<f64> {
    if !enabled {
        return None;
    }
    let th = 10f32.powf(thresh_db as f32 / 20.0);
    let total_s = frames.len() as f64 * WINDOW_S;
    frames
        .iter()
        .rposition(|&p| p >= th)
        .map(|i| (i + 1) as f64 * WINDOW_S)
        .filter(|&e| total_s - e > 0.2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> CueDetectConfig {
        CueDetectConfig {
            enabled: true,
            detect_start: true,
            detect_end: true,
            start_thresh_db: -36.0,
            end_thresh_db: -48.0,
        }
    }

    fn pcm_from_frames(frames: &[i16]) -> Vec<i16> {
        frames.iter().flat_map(|&v| [v; 10]).collect()
    }

    #[test]
    fn start_requires_three_consecutive_windows() {
        let pcm = pcm_from_frames(&[0, 2000, 0, 0, 0, 0, 2000, 2000, 2000, 2000]);
        let (start, _) = detect_boundaries(&pcm, 1000, 1, &cfg());
        assert!((start.unwrap() - 0.06).abs() < 1e-9);
    }

    #[test]
    fn start_keeps_ten_milliseconds_of_silence() {
        let pcm = pcm_from_frames(&[0, 2000, 2000, 2000]);
        let (start, _) = detect_boundaries(&pcm, 1000, 1, &cfg());
        assert!((start.unwrap() - 0.01).abs() < 1e-9);
    }

    #[test]
    fn end_uses_last_signal_after_middle_pause() {
        let pcm = pcm_from_frames(&[
            0, 2000, 2000, 0, 0, 2000, 2000, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let (_, end) = detect_boundaries(&pcm, 1000, 1, &cfg());
        assert!((end.unwrap() - 0.07).abs() < 1e-9);
    }

    #[test]
    fn ignores_insignificant_edges() {
        let pcm = pcm_from_frames(&[2000, 2000, 2000, 2000, 2000, 2000]);
        assert_eq!(detect_boundaries(&pcm, 1000, 1, &cfg()), (None, None));
    }
}
