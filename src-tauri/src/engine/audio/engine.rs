use crate::engine::audio::button::{ButtonStateMap, PlaybackGroup};
use crate::engine::audio::command::AudioCommand;
use crate::engine::audio::last_pressed::LastPressedInfo;
use crate::engine::audio::routing::bus_for;
use crate::engine::audio::thread as audio_thread;
use crate::engine::cache::preload::PreloadCache;
use crate::engine::cache::preloader::Preloader;
use crate::engine::console::{BusId, ConsoleEngine, Routing};
use crate::model::fade::FadeConfig;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

/// Motor de efectos: botones de la botonera y del panel fijo.
///
/// Ya no posee tarjetas ni faders: le pide sus buses a la consola, que es su
/// dueña. El master y el vumetro de la barra inferior son el fader y el medidor
/// del bus `Programa`, y se le piden a ella (`console.fader` / `console.levels`).
pub struct AudioEngine {
    tx: Sender<AudioCommand>,
    button_states: Arc<Mutex<ButtonStateMap>>,
    last_pressed: Arc<Mutex<Option<LastPressedInfo>>>,
    preload_cache: Arc<Mutex<PreloadCache>>,
    preloader: Preloader,
    preload_enabled: Arc<AtomicBool>,
}

impl AudioEngine {
    pub fn new(console: Arc<ConsoleEngine>) -> Self {
        let (tx, rx) = channel::<AudioCommand>();
        let button_states: Arc<Mutex<ButtonStateMap>> = Arc::new(Mutex::new(HashMap::new()));
        let last_pressed = Arc::new(Mutex::new(None));
        let preload_cache = Arc::new(Mutex::new(PreloadCache::new(128)));
        let preload_enabled = Arc::new(AtomicBool::new(false));
        let preloader = Preloader::start(Arc::clone(&preload_cache), Arc::clone(&preload_enabled));
        audio_thread::spawn(
            rx,
            Arc::clone(&button_states),
            Arc::clone(&last_pressed),
            Arc::clone(&preload_cache),
            console,
        );
        Self {
            tx,
            button_states,
            last_pressed,
            preload_cache,
            preloader,
            preload_enabled,
        }
    }
    pub fn preload_cache_handle(&self) -> Arc<Mutex<PreloadCache>> {
        Arc::clone(&self.preload_cache)
    }
    pub fn enqueue_preload(&self, path: String) {
        self.preloader.enqueue(path);
    }
    pub fn set_preload_enabled(&self, enabled: bool) {
        self.preload_enabled.store(enabled, Ordering::Relaxed);
        self.preloader.set_enabled(enabled);
    }
    pub fn button_states_handle(&self) -> Arc<Mutex<ButtonStateMap>> {
        Arc::clone(&self.button_states)
    }
    pub fn last_pressed_handle(&self) -> Arc<Mutex<Option<LastPressedInfo>>> {
        Arc::clone(&self.last_pressed)
    }

    /// La salida principal: la tarjeta por la que sale el programa.
    pub fn set_device(&self, device_name: &str) -> Result<(), String> {
        self.send(AudioCommand::SetBusRouting {
            bus: BusId::Programa,
            routing: Routing::Device(device_name.to_string()),
        })
    }

    /// La salida de pre-escucha. Vacio = comparte tarjeta con el programa, pero
    /// **sigue siendo un bus aparte**: sin master y sin sumar a su vumetro.
    pub fn set_pre_device(&self, device_name: &str) -> Result<(), String> {
        let routing = if device_name.is_empty() {
            Routing::ProgramDevice
        } else {
            Routing::Device(device_name.to_string())
        };
        self.send(AudioCommand::SetBusRouting {
            bus: BusId::Cue,
            routing,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn play_file(
        &self,
        id: String,
        path: &str,
        volume: f32,
        duration: f64,
        loop_mode: bool,
        stop_other: bool,
        overlap: bool,
        restart: bool,
        cue_start_s: f64,
        cue_end_s: Option<f64>,
        file_gain: f32,
        to_pre: bool,
        fade: &FadeConfig,
        group: PlaybackGroup,
    ) -> Result<(), String> {
        if !to_pre && !id.starts_with("__") && group == PlaybackGroup::Main {
            *self.last_pressed.lock().unwrap() = Some(LastPressedInfo { id: id.clone() });
        }
        self.send(AudioCommand::Play {
            id,
            path: path.to_string(),
            volume,
            duration,
            loop_mode,
            stop_other,
            overlap,
            restart,
            cue_start_s,
            cue_end_s,
            file_gain,
            bus: bus_for(to_pre, group),
            fade_in_s: fade.fade_in_s,
            fade_out_stop_s: fade.fade_out_stop_s,
            fade_out_end_s: fade.fade_out_end_s,
            group,
        })
    }

    pub fn stop(&self, id: &str) {
        let _ = self.tx.send(AudioCommand::Stop { id: id.to_string() });
    }
    pub fn stop_fade(&self, id: &str) {
        let _ = self.tx.send(AudioCommand::StopFade { id: id.to_string() });
    }
    pub fn stop_all(&self) {
        *self.last_pressed.lock().unwrap() = None;
        let _ = self.tx.send(AudioCommand::StopAll);
    }
    pub fn stop_group_fade(&self, group: PlaybackGroup) {
        let _ = self.tx.send(AudioCommand::StopGroupFade { group });
    }
    pub fn stop_all_fade(&self) {
        *self.last_pressed.lock().unwrap() = None;
        let _ = self.tx.send(AudioCommand::StopAllFade);
    }

    pub fn set_volume(&self, id: &str, volume: f32) {
        let _ = self.tx.send(AudioCommand::SetVolume {
            id: id.to_string(),
            volume,
        });
    }

    pub fn seek_active(&self, delta_s: Option<f64>, position_s: Option<f64>) -> Result<(), String> {
        self.send(AudioCommand::SeekActive {
            delta_s,
            position_s,
        })
    }

    pub fn play_sequence(
        &self,
        id: String,
        paths: Vec<String>,
        volume: f32,
        duration: f64,
        group: PlaybackGroup,
    ) -> Result<(), String> {
        self.send(AudioCommand::PlaySequence {
            id,
            paths,
            volume,
            duration,
            // Una locucion suena por el bus de su grupo, como cualquier boton.
            bus: bus_for(false, group),
            group,
        })
    }

    fn send(&self, command: AudioCommand) -> Result<(), String> {
        self.tx
            .send(command)
            .map_err(|_| "Audio thread died".to_string())
    }
}
