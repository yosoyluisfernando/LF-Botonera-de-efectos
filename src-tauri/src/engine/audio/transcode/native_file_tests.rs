//! Regresión: MP4 con vídeo y audio se abre por File, sin el panic de Rodio.
use rodio::Source;
use std::process::Command;
#[test]
#[ignore = "requiere LF_TEST_FFMPEG; genera un MP4 real con vídeo y AAC-LC"]
fn native_mp4_plays_only_audio_without_conversion_or_decoder_panic() {
    let ffmpeg = std::env::var("LF_TEST_FFMPEG").unwrap();
    let path = std::env::temp_dir().join(format!("lf-native-av-{}.mp4", std::process::id()));
    assert!(Command::new(ffmpeg)
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=size=32x32:rate=10",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=1000:sample_rate=44100:duration=3",
            "-shortest",
            "-c:v",
            "mpeg4",
            "-c:a",
            "aac"
        ])
        .arg(&path)
        .status()
        .unwrap()
        .success());
    let original = std::fs::read(&path).unwrap();
    let raw = path.to_str().unwrap();
    assert_eq!(super::for_analysis(raw).unwrap(), raw);
    let mut audio = crate::engine::audio::decode::source_from_path(raw, false).unwrap();
    assert_eq!(audio.channels(), 1);
    assert_eq!(audio.sample_rate(), 44100);
    assert!(audio.by_ref().take(44100).any(|value| value.abs() > 0.001));
    let seek = crate::engine::audio::decode::source_from_path_at(raw, false, 1.0).unwrap();
    assert!(seek.take(44100).any(|value| value.abs() > 0.001));
    assert_eq!(original, std::fs::read(&path).unwrap());
    std::fs::remove_file(path).unwrap();
}
