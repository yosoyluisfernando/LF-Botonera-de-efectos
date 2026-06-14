/// Módulo: audio.rs
/// Propósito: Motor de audio con soporte de loop, overlap, restart y stop_other.
/// Corre en hilo dedicado para no bloquear el hilo principal de Tauri.
/// Toda la salida pasa por MasterBus (DynamicMixer → LevelSource → Sink único).

use crate::master_bus::{ButtonStateMap, MasterBus, SequenceSource};
use crate::vu_meter::LastPressedInfo;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub enum AudioCommand {
    Play { id: String, path: String, volume: f32, duration: f64,
           loop_mode: bool, stop_other: bool, overlap: bool, restart: bool },
    Stop         { id: String },
    StopAll,
    SetDevice    { device_name: String },
    SetVolume    { id: String, volume: f32 },
    PlaySequence { id: String, paths: Vec<String>, volume: f32 },
}

pub struct AudioEngine {
    tx:             Sender<AudioCommand>,
    button_states:  Arc<Mutex<ButtonStateMap>>,
    master_level_l: Arc<AtomicU32>,
    master_level_r: Arc<AtomicU32>,
    last_pressed:   Arc<Mutex<Option<LastPressedInfo>>>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let (tx, rx)       = channel::<AudioCommand>();
        let button_states: Arc<Mutex<ButtonStateMap>> = Arc::new(Mutex::new(HashMap::new()));
        let master_level_l = Arc::new(AtomicU32::new(0));
        let master_level_r = Arc::new(AtomicU32::new(0));
        let last_pressed   = Arc::new(Mutex::new(None));

        let states_t = Arc::clone(&button_states);
        let ll_t     = Arc::clone(&master_level_l);
        let lr_t     = Arc::clone(&master_level_r);

        thread::spawn(move || {
            // _stream_data debe vivir todo el ciclo del hilo: OutputStream activa el dispositivo
            let mut _stream_data: Option<(OutputStream, OutputStreamHandle)> = None;
            let mut bus:         Option<MasterBus>                          = None;
            let mut current_device = String::new();

            for cmd in rx {
                // Purgar botones terminados antes de cada comando
                {
                    let mut s = states_t.lock().unwrap();
                    s.retain(|_, v| { v.retain(|b| !b.is_done()); !v.is_empty() });
                }

                match cmd {
                    AudioCommand::SetDevice { device_name } => {
                        if device_name == current_device && bus.is_some() { continue; }
                        let host   = rodio::cpal::default_host();
                        let device = if device_name == "default" {
                            host.default_output_device()
                        } else {
                            host.output_devices().ok().and_then(|mut d|
                                d.find(|x| x.name().unwrap_or_default() == device_name))
                        };
                        if let Some(dev) = device {
                            if let Ok((stream, handle)) = OutputStream::try_from_device(&dev) {
                                states_t.lock().unwrap().clear();
                                bus = MasterBus::new(&handle, Arc::clone(&ll_t), Arc::clone(&lr_t));
                                _stream_data = Some((stream, handle));
                                current_device = device_name;
                            }
                        }
                    }

                    AudioCommand::Play { id, path, volume, duration, loop_mode, stop_other, overlap, restart } => {
                        let Some(ref b) = bus else { continue; };
                        let mut states  = states_t.lock().unwrap();

                        if stop_other {
                            states.iter().filter(|(k, _)| **k != id)
                                .flat_map(|(_, v)| v.iter())
                                .for_each(|s| s.stop());
                            states.retain(|k, _| *k == id);
                        }

                        let is_playing = states.get(&id).map_or(false, |v| !v.is_empty());
                        if is_playing && !overlap {
                            if let Some(g) = states.remove(&id) { for s in g { s.stop(); } }
                            if !restart { continue; }
                        }

                        let Ok(file)    = File::open(&path)                  else { continue; };
                        let Ok(decoder) = Decoder::new(BufReader::new(file)) else { continue; };
                        let source: Box<dyn Source<Item = f32> + Send + 'static> = if loop_mode {
                            Box::new(decoder.repeat_infinite().convert_samples::<f32>())
                        } else {
                            Box::new(decoder.convert_samples::<f32>())
                        };
                        let btn_state = b.add_source(source, volume, duration);
                        states.entry(id).or_default().push(btn_state);
                    }

                    AudioCommand::Stop { id } => {
                        let mut states = states_t.lock().unwrap();
                        if let Some(g) = states.remove(&id) { for s in g { s.stop(); } }
                    }

                    AudioCommand::StopAll => {
                        let mut states = states_t.lock().unwrap();
                        for (_, g) in states.drain() { for s in g { s.stop(); } }
                    }

                    AudioCommand::SetVolume { id, volume } => {
                        let states = states_t.lock().unwrap();
                        if let Some(g) = states.get(&id) { for s in g { s.set_volume(volume); } }
                    }

                    AudioCommand::PlaySequence { id, paths, volume } => {
                        let Some(ref b) = bus else { continue; };
                        let mut states  = states_t.lock().unwrap();
                        if let Some(g) = states.remove(&id) { for s in g { s.stop(); } }
                        if let Some(seq) = SequenceSource::from_paths(&paths) {
                            let btn_state = b.add_source(Box::new(seq), volume, 0.0);
                            states.entry(id).or_default().push(btn_state);
                        }
                    }
                }
            }
        });

        Self { tx, button_states, master_level_l, master_level_r, last_pressed }
    }

    pub fn button_states_handle(&self)  -> Arc<Mutex<ButtonStateMap>>          { Arc::clone(&self.button_states) }
    pub fn master_levels_handles(&self) -> (Arc<AtomicU32>, Arc<AtomicU32>)    { (Arc::clone(&self.master_level_l), Arc::clone(&self.master_level_r)) }
    pub fn last_pressed_handle(&self)   -> Arc<Mutex<Option<LastPressedInfo>>> { Arc::clone(&self.last_pressed) }

    pub fn get_available_devices(&self) -> Vec<String> {
        let host = rodio::cpal::default_host();
        let mut devices = vec!["default".to_string()];
        if let Ok(devs) = host.output_devices() {
            for d in devs { if let Ok(name) = d.name() { devices.push(name); } }
        }
        devices
    }

    pub fn set_device(&self, device_name: &str) -> Result<(), String> {
        self.tx.send(AudioCommand::SetDevice { device_name: device_name.to_string() })
            .map_err(|_| "Audio thread died".to_string())
    }

    pub fn play_file(
        &self, id: String, path: &str, volume: f32, duration: f64,
        loop_mode: bool, stop_other: bool, overlap: bool, restart: bool,
    ) -> Result<(), String> {
        *self.last_pressed.lock().unwrap() = Some(LastPressedInfo { id: id.clone() });
        self.tx.send(AudioCommand::Play {
            id, path: path.to_string(), volume, duration, loop_mode, stop_other, overlap, restart,
        }).map_err(|_| "Audio thread died".to_string())
    }

    pub fn stop(&self, id: &str) {
        let _ = self.tx.send(AudioCommand::Stop { id: id.to_string() });
    }

    pub fn stop_all(&self) {
        *self.last_pressed.lock().unwrap() = None;
        let _ = self.tx.send(AudioCommand::StopAll);
    }

    pub fn set_volume(&self, id: &str, volume: f32) {
        let _ = self.tx.send(AudioCommand::SetVolume { id: id.to_string(), volume });
    }

    pub fn play_sequence(&self, id: String, paths: Vec<String>, volume: f32) -> Result<(), String> {
        self.tx.send(AudioCommand::PlaySequence { id, paths, volume })
            .map_err(|_| "Audio thread died".to_string())
    }
}
