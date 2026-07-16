//! Pruebas del salto de posicion. Se genera un WAV al vuelo: no dependen de
//! ningun archivo del equipo, y comprueban lo que de verdad importa, que es
//! caer DONDE se pide, no solo hacerlo rapido.
use super::*;
use std::fs;
use std::path::PathBuf;

/// WAV mono de 8 kHz con un escalon: la segunda mitad suena diez veces mas
/// fuerte. Asi se sabe, mirando una sola muestra, en que mitad cayo el salto.
fn write_step_wav(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("lf_seek_{}_{}.wav", name, std::process::id()));
    let samples: Vec<i16> = (0..8000)
        .map(|i| if i < 4000 { 1000i16 } else { 10000i16 })
        .collect();
    let mut bytes = Vec::new();
    let data_len = (samples.len() * 2) as u32;
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&8000u32.to_le_bytes());
    bytes.extend_from_slice(&16000u32.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for s in &samples {
        bytes.extend_from_slice(&s.to_le_bytes());
    }
    fs::write(&path, bytes).unwrap();
    path
}

/// El salto debe caer en el punto pedido, no al principio del bloque que lo
/// contiene. Sin descartar el sobrante del bloque, esto devolvia audio ANTERIOR
/// al punto: el fallo que cazo la prueba de `decode.rs`.
#[test]
fn lands_past_the_step_not_before_it() {
    let path = write_step_wav("step");
    let mut source = SeekSource::open_at(path.to_str().unwrap(), 0.5).expect("debe abrir");
    let first = source.next().expect("debe dar audio");
    let _ = fs::remove_file(&path);

    assert!(first > 0.2, "cayo antes del escalon (dio {first}): el salto se quedo corto");
}

/// Sin salto se empieza por el principio: el camino normal no debe alterarse.
#[test]
fn without_seek_it_starts_at_the_beginning() {
    let path = write_step_wav("zero");
    let mut source = SeekSource::open_at(path.to_str().unwrap(), 0.0).expect("debe abrir");
    let first = source.next().expect("debe dar audio");
    let _ = fs::remove_file(&path);

    assert!(first < 0.1, "deberia empezar en la parte floja (dio {first})");
}

/// Conserva el formato: si mintiera en canales o frecuencia, sonaria agudo o
/// grave, porque el mezclador de rodio se fia de estos valores.
#[test]
fn keeps_the_audio_format() {
    let path = write_step_wav("fmt");
    let source = SeekSource::open_at(path.to_str().unwrap(), 0.5).unwrap();
    let (ch, sr) = (source.channels(), source.sample_rate());
    let _ = fs::remove_file(&path);

    assert_eq!(ch, 1);
    assert_eq!(sr, 8000);
}

/// Saltar mas alla del final no debe reventar: simplemente no hay audio.
#[test]
fn seeking_past_the_end_yields_no_audio() {
    let path = write_step_wav("past");
    let source = SeekSource::open_at(path.to_str().unwrap(), 999.0);
    let empty = source.map(|mut s| s.next().is_none()).unwrap_or(true);
    let _ = fs::remove_file(&path);

    assert!(empty, "pasado el final no puede salir audio");
}

/// Un archivo que no existe no revienta: devuelve None y quien llama decide.
#[test]
fn a_missing_file_is_not_a_panic() {
    assert!(SeekSource::open_at(r"C:\no\existe\nada.mp3", 5.0).is_none());
}
