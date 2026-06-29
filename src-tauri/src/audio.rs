use crate::audio_command::AudioCommand;
use crate::audio_thread;
use crate::master_bus::ButtonStateMap;
use crate::preload_cache::PreloadCache;
use crate::preloader::Preloader;
use crate::types_fade::FadeConfig;
use crate::vu_meter::LastPressedInfo;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

pub struct AudioEngine {
    tx: Sender<AudioCommand>,
    button_states: Arc<Mutex<ButtonStateMap>>,
    master_level_l: Arc<AtomicU32>,
    master_level_r: Arc<AtomicU32>,
    master_volume: Arc<AtomicU32>,
    last_pressed: Arc<Mutex<Option<LastPressedInfo>>>,
    preload_cache: Arc<Mutex<PreloadCache>>,
    preloader: Preloader,
    preload_enabled: Arc<AtomicBool>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let (tx, rx) = channel::<AudioCommand>();
        let button_states: Arc<Mutex<ButtonStateMap>> = Arc::new(Mutex::new(HashMap::new()));
        let master_level_l = Arc::new(AtomicU32::new(0));
        let master_level_r = Arc::new(AtomicU32::new(0));
        let master_volume = Arc::new(AtomicU32::new(1.0f32.to_bits()));
        let last_pressed = Arc::new(Mutex::new(None));
        let preload_cache = Arc::new(Mutex::new(PreloadCache::new(128)));
        let preload_enabled = Arc::new(AtomicBool::new(false));
        let preloader = Preloader::start(Arc::clone(&preload_cache), Arc::clone(&preload_enabled));
        audio_thread::spawn(
            rx,
            Arc::clone(&button_states),
            Arc::clone(&master_level_l),
            Arc::clone(&master_level_r),
            Arc::clone(&master_volume),
            Arc::clone(&last_pressed),
            Arc::clone(&preload_cache),
        );
        Self {
            tx,
            button_states,
            master_level_l,
            master_level_r,
            master_volume,
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

    pub fn master_levels_handles(&self) -> (Arc<AtomicU32>, Arc<AtomicU32>) {
        (
            Arc::clone(&self.master_level_l),
            Arc::clone(&self.master_level_r),
        )
    }

    pub fn last_pressed_handle(&self) -> Arc<Mutex<Option<LastPressedInfo>>> {
        Arc::clone(&self.last_pressed)
    }

    pub fn get_available_devices(&self) -> Vec<String> {
        crate::audio_device_list::available_devices()
    }

    pub fn master_volume(&self) -> f32 {
        f32::from_bits(self.master_volume.load(Ordering::Relaxed))
    }

    pub fn set_master_volume(&self, volume: f32) {
        self.master_volume
            .store(volume.clamp(0.0, 1.5).to_bits(), Ordering::Relaxed);
    }

    pub fn set_device(&self, device_name: &str) -> Result<(), String> {
        self.send(AudioCommand::SetDevice {
            device_name: device_name.to_string(),
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
    ) -> Result<(), String> {
        if !to_pre && !id.starts_with("__") {
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
            to_pre,
            fade_in_s: fade.fade_in_s,
            fade_out_stop_s: fade.fade_out_stop_s,
            fade_out_end_s: fade.fade_out_end_s,
        })
    }

    pub fn set_pre_device(&self, device_name: &str) -> Result<(), String> {
        self.send(AudioCommand::SetPreDevice {
            device_name: device_name.to_string(),
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
    ) -> Result<(), String> {
        self.send(AudioCommand::PlaySequence {
            id,
            paths,
            volume,
            duration,
        })
    }

    fn send(&self, command: AudioCommand) -> Result<(), String> {
        self.tx
            .send(command)
            .map_err(|_| "Audio thread died".to_string())
    }
}
