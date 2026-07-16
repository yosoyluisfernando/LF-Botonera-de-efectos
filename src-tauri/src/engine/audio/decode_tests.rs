//! Pruebas de la decodificacion y del salto de posicion visto desde fuera
//! (`source_from_path_at`, que es la puerta que usan los dos motores).
use super::*;
use std::fs;
use std::path::Path;

#[test]
fn source_from_path_at_reads_near_requested_position() {
    let path = std::env::temp_dir().join(format!("lf_seek_{}.wav", std::process::id()));
    let samples = (0..8000)
        .map(|i| if i < 4000 { 1000i16 } else { 10000i16 })
        .collect::<Vec<_>>();
    write_mono_wav(&path, 8000, &samples);

    let mut source = source_from_path_at(path.to_str().unwrap(), false, 0.5).unwrap();
    let first = source.next().unwrap();
    let _ = fs::remove_file(path);

    assert!(first > 0.2);
}

#[test]
#[ignore]
fn manual_seek_probe_from_env() {
    let path = std::env::var("LF_SEEK_TEST_FILE").unwrap();
    let pos = std::env::var("LF_SEEK_TEST_POS")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(180.0);
    let start = std::time::Instant::now();
    let mut source = source_from_path_at(&path, false, pos).unwrap();
    let opened_ms = start.elapsed().as_millis();
    let _ = source.next();
    let first_ms = start.elapsed().as_millis();
    println!("seek probe: open={}ms first_sample={}ms", opened_ms, first_ms);
    assert!(first_ms < 1500);
}

fn write_mono_wav(path: &Path, sample_rate: u32, samples: &[i16]) {
    let mut bytes = Vec::new();
    let data_len = (samples.len() * 2) as u32;
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(path, bytes).unwrap();
}
