/// Modulo: audio_decode.rs
/// Proposito: decodificacion central, incluyendo Ogg/Opus de WhatsApp.
use opus_decoder::OpusDecoder;
use rodio::{buffer::SamplesBuffer, Decoder, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub type BoxSource = Box<dyn Source<Item = f32> + Send + 'static>;

/// Abre un archivo como fuente de audio. Usa Rodio primero y cae a Ogg/Opus.
pub fn source_from_path(path: &str, loop_mode: bool) -> Option<BoxSource> {
    rodio_source(path, loop_mode).or_else(|| opus_source(path, loop_mode))
}

/// Indica si el backend puede decodificar realmente este archivo.
pub fn can_decode(path: &str) -> bool {
    source_from_path(path, false).is_some()
}

fn rodio_source(path: &str, loop_mode: bool) -> Option<BoxSource> {
    let file = File::open(path).ok()?;
    let decoder = Decoder::new(BufReader::new(file)).ok()?;
    if loop_mode {
        Some(Box::new(decoder.repeat_infinite().convert_samples::<f32>()))
    } else {
        Some(Box::new(decoder.convert_samples::<f32>()))
    }
}

fn opus_source(path: &str, loop_mode: bool) -> Option<BoxSource> {
    let (channels, sample_rate, samples) = decode_ogg_opus(path).ok()?;
    let source = SamplesBuffer::new(channels, sample_rate, samples);
    if loop_mode {
        Some(Box::new(source.repeat_infinite()))
    } else {
        Some(Box::new(source))
    }
}

fn decode_ogg_opus(path: &str) -> Result<(u16, u32, Vec<f32>), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
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
        .map_err(|e| e.to_string())?;
    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or("unsupported_audio_format")?
        .clone();

    let sample_rate = track.codec_params.sample_rate.unwrap_or(48_000);
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count())
        .unwrap_or(1)
        .clamp(1, 2);
    let mut decoder = OpusDecoder::new(sample_rate, channels).map_err(|e| e.to_string())?;
    let mut frame = vec![0.0f32; decoder.max_frame_size_per_channel() * channels];
    let mut samples = Vec::new();

    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track.id || is_opus_header(&packet.data) {
            continue;
        }
        let decoded = decoder
            .decode_float(&packet.data, &mut frame, false)
            .map_err(|e| e.to_string())?;
        samples.extend_from_slice(&frame[..decoded * channels]);
    }

    if samples.is_empty() {
        return Err("unsupported_audio_format".to_string());
    }
    Ok((channels as u16, sample_rate, samples))
}

fn is_opus_header(packet: &[u8]) -> bool {
    packet.starts_with(b"OpusHead") || packet.starts_with(b"OpusTags")
}
