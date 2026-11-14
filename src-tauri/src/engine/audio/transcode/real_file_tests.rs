//! Reproducción en buses de memoria; no abre tarjetas ni la aplicación.
use crate::engine::{
    audio::{
        button::PlaybackGroup,
        thread_play::{play_file, PlayArgs},
    },
    cache::preload::PreloadCache,
    console::bus::{Bus, BusOutput},
};
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex};
#[test]
#[ignore = "requiere LF_TEST_CONVERSION_FILES con muestras reales separadas por punto y coma"]
fn real_files_survive_conversion_and_effect_bus_playback() {
    for path in std::env::var("LF_TEST_CONVERSION_FILES")
        .unwrap()
        .split(';')
    {
        super::for_analysis(path).unwrap();
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
            let duration = crate::engine::audio::formats::probe_duration_secs(path);
            assert!(play_file(
                &states,
                Some(&bus),
                &cache,
                PlayArgs {
                    id: "sample".into(),
                    path: path.into(),
                    volume: 1.0,
                    duration,
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
            let count = mixer
                .by_ref()
                .take(48000 * 2 * 3)
                .filter(|sample| sample.abs() > 0.00001)
                .count();
            assert!(count > 0, "no samples: {path}");
            println!("memory bus OK: {path}, loop={loop_mode}, count={count}");
        }
    }
}
