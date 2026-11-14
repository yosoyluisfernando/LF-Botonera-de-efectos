//! Fuente Symphonia para reproducción y seek, con File que conserva tamaño y búsqueda.
//! Evita el adaptador de Rodio 0.19 que convierte SeekError de apertura en panic.
//! Canales y frecuencia se toman del primer bloque decodificado, no de metadatos
//! incompletos del contenedor (algunos MP4 anuncian 48 kHz para AAC de 44,1 kHz).
use crate::engine::audio::decode::BoxSource;
use rodio::Source;
use std::collections::VecDeque;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

/// Fuente de rodio que decodifica con symphonia bajo demanda, paquete a paquete.
/// No carga el archivo entero en memoria.
pub struct SeekSource {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    channels: u16,
    sample_rate: u32,
    /// Muestras ya decodificadas del paquete actual, pendientes de entregar.
    pending: VecDeque<f32>,
    /// Muestras que sobran del bloque donde cayo el salto. El seek posiciona en
    /// el bloque que CONTIENE el punto, no en el punto exacto, asi que hay que
    /// descartar lo que va desde el principio del bloque hasta donde se pidio.
    /// Son milisegundos (un bloque), no minutos: nada que ver con recorrer el
    /// archivo entero, que es lo que hacia falta antes.
    skip: usize,
}

impl SeekSource {
    /// Abre `path` posicionado en `start_s`. `None` si el formato no se puede
    /// decodificar o no admite seek: quien llama decide el plan B.
    pub fn open_at(path: &str, start_s: f64) -> Option<Self> {
        let file = File::open(path).ok()?;
        // El `File` va tal cual: symphonia le pregunta el tamano y por eso puede
        // posicionarse. Ese es todo el truco.
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions {
                    enable_gapless: true,
                    ..Default::default()
                },
                &MetadataOptions::default(),
            )
            .ok()?;
        let mut format = probed.format;

        let (track_id, params, mut decoder) = format
            .tracks()
            .iter()
            .filter(|track| track.codec_params.codec != CODEC_TYPE_NULL)
            .find_map(|track| {
                symphonia::default::get_codecs()
                    .make(&track.codec_params, &DecoderOptions::default())
                    .ok()
                    .map(|decoder| (track.id, track.codec_params.clone(), decoder))
            })?;
        let mut skip_s = 0.0;
        if start_s > 0.0 {
            let seeked = format
                .seek(
                    SeekMode::Accurate,
                    SeekTo::Time {
                        time: Time::from(start_s),
                        track_id: Some(track_id),
                    },
                )
                .ok()?;
            // Tras saltar, el decodificador arrastra estado del punto anterior.
            decoder.reset();
            // El salto cae al principio del bloque que contiene el punto, asi que
            // sobra el trozo que va desde ahi hasta lo pedido: se descarta.
            let difference = seeked.required_ts.saturating_sub(seeked.actual_ts);
            skip_s = params
                .time_base
                .map(|base| {
                    let time = base.calc_time(difference);
                    time.seconds as f64 + time.frac
                })
                .unwrap_or_else(|| {
                    difference as f64 / params.sample_rate.unwrap_or(1).max(1) as f64
                });
        }

        let mut source = Self {
            format,
            decoder,
            track_id,
            channels: 1,
            sample_rate: 1,
            pending: VecDeque::new(),
            skip: 0,
        };
        if !source.fill() {
            return None;
        }
        source.skip =
            (skip_s * source.sample_rate as f64 * source.channels as f64).round() as usize;
        Some(source)
    }

    /// Decodifica el siguiente paquete de nuestra pista. `false` = se acabo.
    fn fill(&mut self) -> bool {
        loop {
            let Ok(packet) = self.format.next_packet() else {
                return false;
            };
            if packet.track_id() != self.track_id {
                continue; // otra pista del contenedor (p. ej. video): se ignora
            }
            let Ok(decoded) = self.decoder.decode(&packet) else {
                continue; // un paquete roto no debe cortar la cancion entera
            };
            self.channels = decoded.spec().channels.count().max(1) as u16;
            self.sample_rate = decoded.spec().rate.max(1);
            let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
            buf.copy_interleaved_ref(decoded);
            self.pending.extend(buf.samples().iter().copied());
            if !self.pending.is_empty() {
                return true;
            }
        }
    }
}

impl Iterator for SeekSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        loop {
            while self.skip > 0 {
                // Sobrante del bloque del salto: se tira sin entregarlo.
                match self.pending.pop_front() {
                    Some(_) => self.skip -= 1,
                    None if self.fill() => continue,
                    None => return None,
                }
            }
            if let Some(s) = self.pending.pop_front() {
                return Some(s);
            }
            if !self.fill() {
                return None;
            }
        }
    }
}

impl Source for SeekSource {
    /// `None`: los paquetes no tienen todos el mismo tamano, y el formato
    /// (canales y frecuencia) no cambia a mitad de pista.
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        self.channels
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Abre `path` posicionado en `start_s` como fuente de rodio.
pub fn seek_source(path: &str, start_s: f64) -> Option<BoxSource> {
    SeekSource::open_at(path, start_s).map(|s| Box::new(s) as BoxSource)
}

#[cfg(test)]
#[path = "seek_source_tests.rs"]
mod seek_source_tests;
