/// Modulo: audio_decode.rs
/// Proposito: decodificacion central, incluyendo Ogg/Opus de WhatsApp.
use opus_decoder::OpusDecoder;
use rodio::{buffer::SamplesBuffer, Decoder, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub type BoxSource = Box<dyn Source<Item = f32> + Send + 'static>;

/// Abre un archivo como fuente de audio. Usa Rodio primero y cae a Ogg/Opus.
pub fn source_from_path(path: &str, loop_mode: bool) -> Option<BoxSource> {
    rodio_source(path, loop_mode).or_else(|| opus_source(path, loop_mode))
}

/// Abre un archivo ya posicionado en `start_s`. En formatos soportados por
/// rodio/symphonia usa seek real; evita descartar muestras una por una.
pub fn source_from_path_at(path: &str, loop_mode: bool, start_s: f64) -> Option<BoxSource> {
    rodio_source_at(path, loop_mode, start_s).or_else(|| opus_source_at(path, loop_mode, start_s))
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

fn rodio_source_at(path: &str, loop_mode: bool, start_s: f64) -> Option<BoxSource> {
    let file = File::open(path).ok()?;
    let mut decoder = Decoder::new(BufReader::new(file)).ok()?;
    if start_s > 0.0 {
        decoder
            .try_seek(Duration::from_secs_f64(start_s))
            .ok()?;
    }
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

fn opus_source_at(path: &str, loop_mode: bool, start_s: f64) -> Option<BoxSource> {
    let (channels, sample_rate, samples) = decode_ogg_opus(path).ok()?;
    let ch = channels.max(1) as usize;
    let pos = ((start_s.max(0.0) * sample_rate as f64) as usize * ch).min(samples.len());
    let pos = pos - (pos % ch);
    let data = samples.into_iter().skip(pos).collect::<Vec<f32>>();
    let source = SamplesBuffer::new(channels, sample_rate, data);
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

#[cfg(test)]
mod tests {
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
}
