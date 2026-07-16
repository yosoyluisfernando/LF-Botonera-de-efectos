//! Modulo: engine/audio/seek_source.rs
//! Proposito: abrir un archivo de audio YA POSICIONADO, con seek de verdad.
//!
//! **Por que existe.** `rodio::Decoder` envuelve el lector en su `ReadSeekSource`,
//! que informa `byte_len() = None`. Symphonia necesita el tamano del archivo para
//! posicionarse en formatos sin indice, asi que su `try_seek` falla siempre
//! (FLAC: `Unseekable`; MP3: `end of stream`). Al fallar, la aplicacion caia a
//! `CuedSource`, que llega al punto pedido **descartando las muestras una a una**:
//! medido, ~55 ms por cada segundo saltado, o sea 6,6 s de silencio para saltar a
//! los 2 minutos. Solo no se notaba en los efectos cortos, porque estan en la
//! cache de RAM y ahi el salto ya era O(1).
//!
//! La solucion es no pasar por rodio para esto: symphonia acepta el `File`
//! directamente y `File` **si** implementa `MediaSource` informando del tamano.
//! Con eso, `format.seek` posiciona de verdad, en milisegundos.
//!
//! Sirve a los dos motores: efectos y reproductor comparten `source_from_path_at`.
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
            .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
            .ok()?;
        let mut format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)?;
        let track_id = track.id;
        let params = track.codec_params.clone();
        let channels = params.channels?.count() as u16;
        let sample_rate = params.sample_rate?;

        let mut decoder = symphonia::default::get_codecs()
            .make(&params, &DecoderOptions::default())
            .ok()?;

        let channels = channels.max(1);
        let mut skip = 0usize;
        if start_s > 0.0 {
            let seeked = format
                .seek(
                    SeekMode::Accurate,
                    SeekTo::Time { time: Time::from(start_s), track_id: Some(track_id) },
                )
                .ok()?;
            // Tras saltar, el decodificador arrastra estado del punto anterior.
            decoder.reset();
            // El salto cae al principio del bloque que contiene el punto, asi que
            // sobra el trozo que va desde ahi hasta lo pedido: se descarta.
            skip = seeked.required_ts.saturating_sub(seeked.actual_ts) as usize
                * channels as usize;
        }

        Some(Self {
            format,
            decoder,
            track_id,
            channels,
            sample_rate: sample_rate.max(1),
            pending: VecDeque::new(),
            skip,
        })
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
