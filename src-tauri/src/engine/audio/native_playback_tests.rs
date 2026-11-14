//! Pruebas de audio nativo en memoria, sin procesos externos ni tarjetas físicas.
use crate::engine::{
    audio::{
        button::PlaybackGroup,
        decode, formats,
        thread_play::{play_file, PlayArgs},
    },
    cache::preload::PreloadCache,
    console::bus::{Bus, BusOutput},
};
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex};
#[test]
fn invalid_mp4_is_rejected_without_panicking() {
    let path = std::env::temp_dir().join(format!("lf-invalid-native-{}.mp4", std::process::id()));
    std::fs::write(&path, b"not a media file").unwrap();
    assert_eq!(
        formats::validate_audio_file(path.to_str().unwrap()).unwrap_err(),
        "unsupported_audio_format"
    );
    assert!(decode::source_from_path(path.to_str().unwrap(), false).is_none());
    std::fs::remove_file(path).unwrap();
}
#[test]
#[ignore = "requiere LF_TEST_NATIVE_FILES, rutas compatibles separadas por punto y coma"]
fn native_files_play_seek_and_loop_in_memory() {
    for path in std::env::var("LF_TEST_NATIVE_FILES").unwrap().split(';') {
        let original = std::fs::read(path).unwrap();
        formats::validate_audio_file(path).unwrap();
        let source = decode::source_from_path_at(path, false, 0.1).unwrap();
        assert!(source.take(48000).any(|value| value.abs() > 0.00001));
        for loop_mode in [false, true] {
            let (parent, mut mixer) = rodio::dynamic_mixer::mixer::<f32>(2, 48000);
            let bus = Bus::open(
                BusOutput::Bus(&parent),
                Arc::new(AtomicU32::new(0)),
                Arc::new(AtomicU32::new(0)),
                Arc::new(AtomicU32::new(1f32.to_bits())),
            )
            .unwrap();
            let states = Arc::new(Mutex::new(Default::default()));
            let cache = Arc::new(Mutex::new(PreloadCache::new(16)));
            assert!(play_file(
                &states,
                Some(&bus),
                &cache,
                PlayArgs {
                    id: "sample".into(),
                    path: path.into(),
                    volume: 1.0,
                    duration: formats::probe_duration_secs(path),
                    loop_mode,
                    stop_other: true,
                    overlap: false,
                    restart: true,
                    cue_start_s: 0.0,
                    cue_end_s: None,
                    file_gain: 1.0,
                    fade_in_s: 0.0,
                    fade_out_stop_s: 0.0,
                    fade_out_end_s: 0.0,
                    position_offset_s: 0.0,
                    group: PlaybackGroup::Main,
                }
            ));
            assert!(mixer
                .by_ref()
                .take(48000 * 2)
                .any(|value| value.abs() > 0.00001));
        }
        let analysis =
            crate::engine::dsp::analysis::analyze(path, &Default::default(), &Default::default())
                .unwrap();
        assert!(analysis.meta.duration_s > 0.0);
        assert_eq!(original, std::fs::read(path).unwrap());
    }
}
#[test]
#[ignore = "requiere LF_TEST_NATIVE_VIDEO con vídeo y AAC-LC mono/44,1 kHz"]
fn mp4_video_exposes_only_its_native_audio() {
    use rodio::Source;
    let path = std::env::var("LF_TEST_NATIVE_VIDEO").unwrap();
    formats::validate_audio_file(&path).unwrap();
    let source = decode::source_from_path(&path, false).unwrap();
    assert_eq!(source.channels(), 1);
    assert_eq!(source.sample_rate(), 44100);
    assert!(source.take(44100).any(|sample| sample.abs() > 0.001));
}
#[test]
#[ignore = "requiere LF_TEST_UNSUPPORTED_AUDIO con un perfil HE-AAC no admitido"]
fn unsupported_profile_is_not_partially_played_or_analyzed() {
    let path = std::env::var("LF_TEST_UNSUPPORTED_AUDIO").unwrap();
    assert_eq!(
        formats::validate_audio_file(&path).unwrap_err(),
        "unsupported_audio_format"
    );
    assert!(decode::source_from_path(&path, false).is_none());
    assert!(
        crate::engine::dsp::analysis::analyze(&path, &Default::default(), &Default::default())
            .is_err()
    );
}
