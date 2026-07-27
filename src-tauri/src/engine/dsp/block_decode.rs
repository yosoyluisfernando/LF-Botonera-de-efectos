//! Decodificacion para analisis DSP, por paquetes en vez de muestra a muestra.
use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub struct DecodedAudio {
    pub samples: Vec<f32>,
    pub channels: u16,
    pub sample_rate: u32,
}

/// Usa Symphonia directamente para copiar bloques completos. `None` permite
/// caer al decodificador central, que conserva el soporte Ogg/Opus especial.
pub fn decode(path: &str) -> Option<DecodedAudio> {
    let file = File::open(path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(extension) = Path::new(path).extension().and_then(|value| value.to_str()) {
        hint.with_extension(extension);
    }
    let format_options = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_options, &MetadataOptions::default())
        .ok()?;
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|track| track.codec_params.codec != CODEC_TYPE_NULL)?;
    let track_id = track.id;
    let params = track.codec_params.clone();
    let channels = params.channels?.count().clamp(1, u16::MAX as usize) as u16;
    let sample_rate = params.sample_rate?.max(1);
    let mut decoder = symphonia::default::get_codecs()
        .make(&params, &DecoderOptions::default())
        .ok()?;
    let capacity = params
        .n_frames
        .and_then(|frames| usize::try_from(frames).ok())
        .and_then(|frames| frames.checked_mul(channels as usize))
        .unwrap_or(0);
    let mut samples = Vec::with_capacity(capacity);

    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }
        let Ok(decoded) = decoder.decode(&packet) else {
            continue;
        };
        let mut buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
        buffer.copy_interleaved_ref(decoded);
        samples.extend_from_slice(buffer.samples());
    }
    (!samples.is_empty()).then_some(DecodedAudio {
        samples,
        channels,
        sample_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::audio::decode as audio_decode;
    use rodio::Source;
    use std::fs;

    #[test]
    fn block_decode_matches_shared_decoder_for_wav() {
        let path = std::env::temp_dir().join(format!("lf_block_{}.wav", std::process::id()));
        write_wav(&path);
        let expected = audio_decode::source_from_path(path.to_str().unwrap(), false).unwrap();
        let expected_channels = expected.channels();
        let expected_rate = expected.sample_rate();
        let expected = expected.collect::<Vec<_>>();
        let got = decode(path.to_str().unwrap()).unwrap();
        let _ = fs::remove_file(path);
        assert_eq!(got.channels, expected_channels);
        assert_eq!(got.sample_rate, expected_rate);
        assert_eq!(got.samples, expected);
    }

    #[test]
    #[ignore = "requiere LF_TEST_ANALYSIS_TRACK y abre un audio real en solo lectura"]
    fn real_block_decode_preserves_shared_decoder_timing() {
        let path = std::env::var("LF_TEST_ANALYSIS_TRACK").expect("falta LF_TEST_ANALYSIS_TRACK");
        let expected = audio_decode::source_from_path(&path, false).unwrap();
        let expected_channels = expected.channels();
        let expected_rate = expected.sample_rate();
        let expected = expected.collect::<Vec<_>>();
        let got = decode(&path).unwrap();
        assert_eq!(got.channels, expected_channels);
        assert_eq!(got.sample_rate, expected_rate);
        assert_eq!(got.samples.len(), expected.len());
        let max_delta = got
            .samples
            .iter()
            .zip(expected)
            .map(|(left, right)| (left - right).abs())
            .fold(0.0f32, f32::max);
        assert!(max_delta <= 1.0 / i16::MAX as f32);
    }

    fn write_wav(path: &Path) {
        let samples = [-10_000i16, 0, 10_000, 20_000];
        let data_len = (samples.len() * 2) as u32;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&8_000u32.to_le_bytes());
        bytes.extend_from_slice(&16_000u32.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        fs::write(path, bytes).unwrap();
    }
}
