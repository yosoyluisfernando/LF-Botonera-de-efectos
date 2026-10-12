//! Captura para medición exclusivamente; no conecta la entrada a ninguna salida.
use rodio::cpal::{
    self,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, SizedSample,
};
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    Arc,
};

#[derive(Default)]
pub struct Meter {
    pub db: AtomicU32,
    pub frames: AtomicU64,
    pub failed: AtomicBool,
}

pub fn devices() -> Result<Vec<String>, String> {
    cpal::default_host()
        .input_devices()
        .map_err(|_| "mic.input_error".to_string())?
        .map(|d| d.name().map_err(|_| "mic.input_error".to_string()))
        .collect()
}

pub struct InputStream {
    _stream: cpal::Stream,
    name: String,
}

impl InputStream {
    pub fn matches(&self, input: Option<&String>) -> bool {
        same_input(&self.name, input)
    }
}

fn same_input(captured: &str, input: Option<&String>) -> bool {
    input.is_some_and(|name| name == captured)
}

fn resolve(name: &str) -> Result<cpal::Device, String> {
    let host = cpal::default_host();
    if name.is_empty() {
        host.default_input_device()
    } else {
        host.input_devices()
            .ok()
            .and_then(|mut ds| ds.find(|d| d.name().ok().as_deref() == Some(name)))
    }
    .ok_or_else(|| "mic.input_error".into())
}

pub fn input_name(selection: &str) -> Result<String, String> {
    resolve(selection)?
        .name()
        .map_err(|_| "mic.input_error".into())
}

pub fn preview_input(input: Option<String>) -> (Option<String>, Option<String>) {
    match input.map(|name| input_name(&name)).transpose() {
        Ok(input) => (input, None),
        Err(error) => (None, Some(error)),
    }
}

pub fn open(name: &str, meter: Arc<Meter>) -> Result<InputStream, String> {
    let device = resolve(name)?;
    let name = device.name().map_err(|_| "mic.input_error")?;
    let config = device
        .default_input_config()
        .map_err(|_| "mic.input_error")?;
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build::<f32>(&device, &config.into(), meter),
        cpal::SampleFormat::I16 => build::<i16>(&device, &config.into(), meter),
        cpal::SampleFormat::U16 => build::<u16>(&device, &config.into(), meter),
        cpal::SampleFormat::I32 => build::<i32>(&device, &config.into(), meter),
        cpal::SampleFormat::F64 => build::<f64>(&device, &config.into(), meter),
        _ => return Err("mic.input_format".into()),
    }
    .map_err(|_| "mic.input_error")?;
    stream.play().map_err(|_| "mic.input_error")?;
    Ok(InputStream {
        _stream: stream,
        name,
    })
}

fn build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    meter: Arc<Meter>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: SizedSample,
    f32: cpal::FromSample<T>,
{
    let failed = Arc::clone(&meter);
    meter.db.store((-100.0f32).to_bits(), Ordering::Relaxed);
    device.build_input_stream(
        config,
        move |data: &[T], _| {
            if data.is_empty() {
                return;
            }
            let sum: f64 = data
                .iter()
                .map(|s| {
                    let v = f32::from_sample(*s) as f64;
                    if v.is_finite() {
                        v * v
                    } else {
                        0.0
                    }
                })
                .sum();
            let rms = (sum / data.len() as f64).sqrt();
            let db = if rms > 0.0 {
                (20.0 * rms.log10()).max(-100.0) as f32
            } else {
                -100.0
            };
            meter.db.store(db.to_bits(), Ordering::Relaxed);
            meter.frames.fetch_add(1, Ordering::Release);
        },
        move |_| {
            failed.failed.store(true, Ordering::Relaxed);
        },
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::{preview_input, same_input};

    #[test]
    fn closing_preview_does_not_resolve_or_open_an_input() {
        assert_eq!(preview_input(None), (None, None));
    }

    #[test]
    fn resolved_names_share_only_the_same_input() {
        let resolved_default = "Microphone A".to_string();
        let selected_by_name = "Microphone A".to_string();
        assert!(same_input(&resolved_default, Some(&selected_by_name)));
        assert!(same_input(&selected_by_name, Some(&resolved_default)));
        assert!(!same_input(&resolved_default, Some(&"Microphone B".into())));
        assert!(!same_input(&resolved_default, None));
    }
}
