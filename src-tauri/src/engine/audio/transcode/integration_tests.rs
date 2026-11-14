//! Ejecutar con LF_TEST_FFMPEG y LF_FFMPEG_PATH; no abre tarjetas ni ventanas.
use super::*;
use rodio::Source;
use std::path::Path;
use std::process::Command;
fn ffmpeg() -> String {
    std::env::var("LF_TEST_FFMPEG").expect("falta LF_TEST_FFMPEG")
}
fn fixture(name: &str, profile: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("lf-convert-{}-{name}.m4a", std::process::id()));
    let mut command = Command::new(ffmpeg());
    command.args([
        "-v",
        "error",
        "-y",
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=1000:sample_rate=48000:duration=1",
    ]);
    if profile == "opus" {
        command.args(["-c:a", "libopus", "-f", "mp4"]);
    } else {
        command.args(["-c:a", "aac", "-profile:a", profile]);
    }
    let status = command.arg(&path).status().unwrap();
    assert!(status.success());
    path
}
#[test]
#[ignore = "requiere FFmpeg externo; genera audio sintético sin abrir la aplicación"]
fn converts_unsupported_mp4_codec_preserves_original_and_shared_analysis_timing() {
    let path = fixture("opus", "opus");
    let pathstr = path.to_str().unwrap();
    let original = std::fs::read(&path).unwrap();
    prepare(pathstr).unwrap();
    let resolved = for_analysis(pathstr).unwrap();
    assert_ne!(resolved, pathstr);
    assert_eq!(Path::new(&resolved).extension().unwrap(), "flac");
    assert_eq!(resolve(pathstr).unwrap(), resolved);
    let source = crate::engine::audio::decode::source_from_path(pathstr, false).unwrap();
    assert_eq!(source.sample_rate(), 48000);
    let total = source.collect::<Vec<_>>();
    let seek = crate::engine::audio::decode::source_from_path_at(pathstr, false, 0.5).unwrap();
    assert!(seek.collect::<Vec<_>>().len() < total.len());
    let result =
        crate::engine::dsp::analysis::analyze(pathstr, &Default::default(), &Default::default())
            .unwrap();
    assert_eq!(result.meta.path, pathstr);
    assert_eq!(result.meta.sample_rate, 48000);
    assert!((result.meta.duration_s - 1.0).abs() < 0.1);
    let mut button = crate::domain::button::defaults::new_button("paleta_1", 1, "Test", "#000000", "#ffffff");
    button.path = pathstr.to_string();
    button.duration = -1.0;
    let view = crate::domain::grid::view::button_to_view(&button);
    assert_eq!(view.data.path, pathstr);
    assert!((view.data.duration - 1.0).abs() < 0.1);
    assert_eq!(button.duration, -1.0);
    assert_eq!(original, std::fs::read(&path).unwrap());
    std::fs::remove_file(&path).unwrap();
    let _ = std::fs::remove_file(resolved);
}
#[test]
#[ignore = "requiere FFmpeg externo; comprueba que AAC-LC no se convierte"]
fn supported_aac_uses_the_original() {
    let path = fixture("lc", "aac_low");
    let pathstr = path.to_str().unwrap();
    assert_eq!(for_analysis(pathstr).unwrap(), pathstr);
    std::fs::remove_file(path).unwrap();
}
#[test]
#[ignore = "requiere FFmpeg externo; comprueba fallo sin publicar copias incompletas"]
fn broken_input_reports_failure_without_modifying_the_original() {
    let path = std::env::temp_dir().join(format!("lf-convert-broken-{}.m4a", std::process::id()));
    std::fs::write(&path, b"broken audio").unwrap();
    let pathstr = path.to_str().unwrap();
    let target = cache::target_for(pathstr).unwrap();
    assert_eq!(for_analysis(pathstr).unwrap_err(), "audio_convert_failed");
    assert!(!target.exists());
    assert_eq!(std::fs::read(&path).unwrap(), b"broken audio");
    std::fs::remove_file(path).unwrap();
}
