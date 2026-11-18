//! Catálogo de salida: identidad persistente y nombre legible separados.
use rodio::cpal::{
    self,
    traits::{DeviceTrait, HostTrait},
};
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OutputDevice {
    pub id: String,
    pub name: String,
    pub usable: bool,
}
#[derive(Clone)]
struct Entry {
    info: OutputDevice,
    device: Option<cpal::Device>,
}
#[derive(Default)]
struct Catalog {
    initialized: bool,
    entries: Vec<Entry>,
    default: Option<cpal::Device>,
    default_key: String,
}
static CATALOG: OnceLock<Mutex<Catalog>> = OnceLock::new();
fn catalog() -> &'static Mutex<Catalog> {
    CATALOG.get_or_init(|| Mutex::new(Catalog::default()))
}
pub fn refresh() -> Vec<OutputDevice> {
    let host = cpal::default_host();
    let devices: Vec<_> = host
        .output_devices()
        .map(|d| d.collect())
        .unwrap_or_default();
    let names: Vec<_> = devices
        .iter()
        .map(|d| d.name().unwrap_or_default())
        .collect();
    #[cfg(windows)]
    let identities = super::device_windows::identities();
    #[cfg(not(windows))]
    let identities: Vec<(String, String)> = names
        .iter()
        .map(|name| {
            #[cfg(target_os = "linux")]
            let id = super::device_linux::identity(name);
            #[cfg(not(target_os = "linux"))]
            let id = String::new();
            (id, name.clone())
        })
        .collect();
    let mut entries = Vec::new();
    let counts: std::collections::HashMap<_, usize> =
        identities
            .iter()
            .fold(std::collections::HashMap::new(), |mut counts, (_, name)| {
                *counts.entry(name.clone()).or_default() += 1;
                counts
            });
    for (id, name) in identities {
        let matches: Vec<_> = names
            .iter()
            .enumerate()
            .filter(|(_, n)| **n == name)
            .collect();
        let device = (matches.len() == 1 && counts.get(&name) == Some(&1))
            .then(|| devices[matches[0].0].clone());
        entries.push(Entry {
            info: OutputDevice {
                id,
                name,
                usable: device.is_some(),
            },
            device,
        });
    }
    // Si la API nativa falla, mantener compatibilidad por nombre único, sin inventar IDs.
    for (i, name) in names.iter().enumerate() {
        if !entries.iter().any(|e| e.info.name == *name) {
            let unique = names.iter().filter(|n| *n == name).count() == 1;
            entries.push(Entry {
                info: OutputDevice {
                    id: String::new(),
                    name: name.clone(),
                    usable: unique,
                },
                device: unique.then(|| devices[i].clone()),
            });
        }
    }
    let default = host.default_output_device();
    let default_name = default
        .as_ref()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();
    let default_key = unique(&entries, &default_name)
        .map(|e| key(&e.info))
        .unwrap_or_else(|| "default".into());
    let info = entries.iter().map(|e| e.info.clone()).collect();
    *catalog().lock().unwrap() = Catalog {
        initialized: true,
        entries,
        default,
        default_key,
    };
    info
}
fn key(info: &OutputDevice) -> String {
    if info.id.is_empty() {
        info.name.clone()
    } else {
        info.id.clone()
    }
}
fn unique<'a>(entries: &'a [Entry], name: &str) -> Option<&'a Entry> {
    if name.starts_with("wasapi:") || name.starts_with("alsa:") {
        return entries.iter().find(|e| e.info.id == name);
    }
    let mut matches = entries
        .iter()
        .filter(|e| e.info.id == name || e.info.name == name);
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}
fn ensure() {
    if !catalog().lock().unwrap().initialized {
        refresh();
    }
}
pub fn list() -> Vec<OutputDevice> {
    ensure();
    catalog()
        .lock()
        .unwrap()
        .entries
        .iter()
        .map(|e| e.info.clone())
        .collect()
}
pub fn resolve(value: &str) -> String {
    ensure();
    let data = catalog().lock().unwrap();
    if value.trim().is_empty() || value == "default" {
        return data.default_key.clone();
    }
    unique(&data.entries, value)
        .map(|e| key(&e.info))
        .unwrap_or_else(|| value.trim().to_string())
}
pub fn find(value: &str) -> Option<cpal::Device> {
    ensure();
    let data = catalog().lock().unwrap();
    if value.is_empty() || value == "default" {
        return data.default.clone();
    }
    unique(&data.entries, value)?.device.clone()
}
pub fn label(value: &str) -> String {
    ensure();
    let data = catalog().lock().unwrap();
    unique(&data.entries, value)
        .map(|e| e.info.name.clone())
        .unwrap_or_else(|| value.into())
}
pub fn descriptor(value: &str) -> Option<OutputDevice> {
    ensure();
    let data = catalog().lock().unwrap();
    unique(&data.entries, value).map(|e| e.info.clone())
}
