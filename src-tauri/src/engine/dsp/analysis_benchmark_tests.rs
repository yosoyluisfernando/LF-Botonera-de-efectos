//! Perfil manual Release del camino costoso del editor con un audio real.
use super::analyze;
use crate::engine::cache::waveform_binary;
use crate::model::norm::{CueDetectConfig, NormConfig};
use std::fs;
use std::time::Instant;

#[test]
#[ignore = "requiere LF_TEST_ANALYSIS_TRACK y abre un audio real en solo lectura"]
fn real_editor_analysis_profile() {
    let path = std::env::var("LF_TEST_ANALYSIS_TRACK").expect("falta LF_TEST_ANALYSIS_TRACK");
    let total_started = Instant::now();
    let analysis_started = Instant::now();
    let result = analyze(&path, &NormConfig::default(), &CueDetectConfig::default())
        .expect("analisis del editor");
    let analysis_ms = analysis_started.elapsed().as_millis();

    let cache_path =
        std::env::temp_dir().join(format!("lf_editor_profile_{}.wfc", std::process::id()));
    let write_started = Instant::now();
    let bytes = waveform_binary::write(&cache_path, &result.envelope).expect("guardar waveform");
    let write_ms = write_started.elapsed().as_millis();
    let read_started = Instant::now();
    let loaded = waveform_binary::read(&cache_path).expect("leer waveform");
    let read_ms = read_started.elapsed().as_millis();
    let _ = fs::remove_file(cache_path);

    assert_eq!(loaded.parts().0.len(), result.envelope.parts().0.len());
    println!(
        "duration_s={:.3} samples={} points={} analysis_ms={analysis_ms} \
         write_ms={write_ms} read_ms={read_ms} bytes={bytes} total_ms={}",
        result.meta.duration_s,
        result.pcm.data.len(),
        result.envelope.parts().0.len(),
        total_started.elapsed().as_millis()
    );
}
