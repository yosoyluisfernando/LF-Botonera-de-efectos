//! Modulo: engine/audio/transcode/probe.rs
//! Proposito: decidir si un archivo MP4/M4A/AAC se puede reproducir TAL CUAL
//! con calidad completa, o si hay que convertirlo.
//!
//! symphonia decodifica AAC-LC y ALAC, pero no HE-AAC (SBR/PS):
//! - con señalizacion EXPLICITA (tipo de objeto 5/29, o extension 0x2B7) el
//!   decodificador falla o ignora la extension;
//! - con señalizacion IMPLICITA decodifica solo el nucleo a media frecuencia:
//!   suena, pero opaco y sin agudos. Es una perdida de calidad silenciosa, y
//!   por eso un nucleo AAC por debajo de 32 kHz se trata como HE-AAC.
use std::fs::File;
use std::path::Path;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_AAC, CODEC_TYPE_ALAC, CODEC_TYPE_NULL};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Extensiones de la familia MP4/AAC a las que se aplica esta decision.
pub const CANDIDATES: [&str; 5] = ["m4a", "mp4", "aac", "m4b", "3gp"];

pub fn is_candidate(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| CANDIDATES.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// true = se reproduce directo con calidad completa.
/// false = hay que convertirlo (o no se puede abrir: dañado, codec ajeno).
pub fn plays_natively(path: &str) -> bool {
    probe(path).unwrap_or(false)
}

fn probe(path: &str) -> Option<bool> {
    let file = File::open(path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .ok()?;
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)?
        .clone();
    let params = &track.codec_params;
    let full_quality = if params.codec == CODEC_TYPE_ALAC {
        true
    } else if params.codec == CODEC_TYPE_AAC {
        aac_is_plain_lc(
            params.extra_data.as_deref(),
            params.sample_rate.unwrap_or(0),
        )
    } else {
        // Otro codec dentro de un MP4 (MP3, por ejemplo): si symphonia lo abre,
        // lo decodifica entero.
        true
    };
    if !full_quality {
        return Some(false);
    }
    // Que la cabecera parezca buena no basta: el primer paquete tiene que
    // decodificarse de verdad (un archivo cortado o dañado falla aqui).
    let mut decoder = symphonia::default::get_codecs()
        .make(params, &DecoderOptions::default())
        .ok()?;
    loop {
        let packet = format.next_packet().ok()?;
        if packet.track_id() != track.id {
            continue;
        }
        return Some(decoder.decode(&packet).is_ok());
    }
}

/// Lee el AudioSpecificConfig (ISO 14496-3) y dice si es AAC-LC sin SBR.
/// Sin configuracion (ADTS crudo) se decide solo por la frecuencia.
pub fn aac_is_plain_lc(asc: Option<&[u8]>, sample_rate: u32) -> bool {
    if sample_rate != 0 && sample_rate < 32_000 {
        return false; // nucleo a media frecuencia: HE-AAC implicito
    }
    let Some(asc) = asc.filter(|a| a.len() >= 2) else {
        return true;
    };
    let mut bits = Bits::new(asc);
    let mut object_type = bits.read(5);
    if object_type == 31 {
        object_type = 32 + bits.read(6);
    }
    // 2 = LC. 5 = SBR (HE-AAC), 29 = PS (HE-AAC v2); 1/4 = Main/LTP, que
    // symphonia tampoco decodifica.
    if object_type != 2 {
        return false;
    }
    !has_sync_extension(asc)
}

/// La señalizacion explicita compatible: la marca 0x2B7 (11 bits) seguida del
/// tipo SBR, al final del AudioSpecificConfig.
fn has_sync_extension(asc: &[u8]) -> bool {
    let total = asc.len() * 8;
    (16..total.saturating_sub(10)).any(|start| {
        let mut bits = Bits::at(asc, start);
        bits.read(11) == 0x2B7 && bits.read(5) == 5 && bits.read(1) == 1
    })
}

struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Bits<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self::at(data, 0)
    }
    fn at(data: &'a [u8], pos: usize) -> Self {
        Self { data, pos }
    }
    fn read(&mut self, n: usize) -> u32 {
        let mut value = 0u32;
        for _ in 0..n {
            let byte = self.data.get(self.pos / 8).copied().unwrap_or(0);
            let bit = (byte >> (7 - self.pos % 8)) & 1;
            value = (value << 1) | bit as u32;
            self.pos += 1;
        }
        value
    }
}

#[cfg(test)]
#[path = "probe_tests.rs"]
mod probe_tests;
