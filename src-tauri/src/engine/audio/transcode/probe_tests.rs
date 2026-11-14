//! Pruebas de la decision "nativo o convertir" para la familia MP4/AAC.
use super::{aac_is_plain_lc, is_candidate};

/// AAC-LC 44,1 kHz estereo: 00010 0100 0010 000 → 0x12 0x10.
const LC_44_STEREO: [u8; 2] = [0x12, 0x10];
/// HE-AAC explicito jerarquico: tipo 5 (SBR) → 00101 ...
const HE_EXPLICIT: [u8; 4] = [0x2B, 0x92, 0x08, 0x00];
/// LC 24 kHz con extension compatible 0x2B7 + SBR (lo que escribe un codificador
/// HE-AAC "backward compatible"): 0x13 0x10 0x56 0xE5 0x98.
const LC_WITH_SBR_EXT: [u8; 5] = [0x13, 0x10, 0x56, 0xE5, 0x98];

#[test]
fn aac_lc_a_44k_se_reproduce_directo() {
    assert!(aac_is_plain_lc(Some(&LC_44_STEREO), 44_100));
    assert!(aac_is_plain_lc(Some(&LC_44_STEREO), 48_000));
}

#[test]
fn he_aac_explicito_se_convierte() {
    assert!(!aac_is_plain_lc(Some(&HE_EXPLICIT), 44_100));
}

#[test]
fn extension_sbr_compatible_se_convierte() {
    assert!(!aac_is_plain_lc(Some(&LC_WITH_SBR_EXT), 48_000));
}

/// HE-AAC implicito: el contenedor dice 22,05/24 kHz (el nucleo). Reproducirlo
/// directo sonaria opaco, sin agudos.
#[test]
fn nucleo_a_media_frecuencia_se_convierte() {
    assert!(!aac_is_plain_lc(Some(&LC_44_STEREO), 22_050));
    assert!(!aac_is_plain_lc(None, 24_000));
}

#[test]
fn adts_sin_configuracion_a_frecuencia_completa_es_directo() {
    assert!(aac_is_plain_lc(None, 44_100));
}

#[test]
fn solo_la_familia_mp4_es_candidata() {
    assert!(is_candidate("C:/fx/Aplausos.M4A"));
    assert!(is_candidate("efecto.aac"));
    assert!(!is_candidate("efecto.mp3"));
    assert!(!is_candidate("sin_extension"));
}

#[test]
fn lc_with_explicitly_disabled_sbr_stays_native() {
    // FFmpeg 9 AAC-LC mono/48 kHz, sync extension 0x2b7/type 5, SBR flag = 0.
    assert!(aac_is_plain_lc(
        Some(&[0x11, 0x88, 0x56, 0xe5, 0x00]),
        48_000
    ));
    // La misma señalización, ahora con SBR presente.
    assert!(!aac_is_plain_lc(
        Some(&[0x11, 0x88, 0x56, 0xe5, 0x80]),
        48_000
    ));
}
