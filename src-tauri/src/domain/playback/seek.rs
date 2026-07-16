use crate::domain::playback::source as playback_source;
use crate::engine::audio::attach::{attach_button, AttachArgs};
use crate::engine::audio::button::{ButtonStateMap, PlaybackGroup};
use crate::engine::audio::last_pressed::LastPressedInfo;
/// Modulo: playback_seek.rs
/// Proposito: estado de reconstruccion y seek para botones principales.
use crate::engine::audio::ops as audio_ops;
use crate::engine::cache::preload::PreloadCache;
use crate::engine::console::Bus;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ReplayInfo {
    pub id: String,
    pub path: String,
    pub volume: f32,
    pub duration: f64,
    pub loop_mode: bool,
    pub cue_start_s: f64,
    pub cue_end_s: Option<f64>,
    pub file_gain: f32,
    pub fade_in_s: f64,
    pub fade_out_stop_s: f64,
    pub fade_out_end_s: f64,
}

#[allow(clippy::too_many_arguments)]
pub fn seek_active(
    states: &Arc<Mutex<ButtonStateMap>>,
    bus: Option<&Bus>,
    cache: &Arc<Mutex<PreloadCache>>,
    last_pressed: &Mutex<Option<LastPressedInfo>>,
    replays: &HashMap<String, ReplayInfo>,
    delta_s: Option<f64>,
    position_s: Option<f64>,
) {
    let Some(bus) = bus else {
        return;
    };
    let Some(id) = last_pressed.lock().unwrap().as_ref().map(|i| i.id.clone()) else {
        return;
    };
    let Some(info) = replays.get(&id).cloned() else {
        return;
    };
    let Some(target) = target_position(
        states,
        &id,
        info.duration,
        info.loop_mode,
        delta_s,
        position_s,
    ) else {
        return;
    };
    let Some(source) = playback_source::build_seek(
        cache,
        &info.path,
        info.loop_mode,
        info.cue_start_s,
        target,
        info.cue_end_s,
    ) else {
        return;
    };
    audio_ops::stop_id(states, &id);
    let state = attach_button(
        bus,
        source,
        AttachArgs {
            volume: info.volume,
            duration: info.duration,
            loop_mode: info.loop_mode,
            file_gain: info.file_gain,
            fade_in_s: info.fade_in_s,
            fade_out_stop_s: info.fade_out_stop_s,
            fade_out_end_s: info.fade_out_end_s,
            position_offset_s: target,
            group: PlaybackGroup::Main,
        },
    );
    states.lock().unwrap().entry(id).or_default().push(state);
}

fn target_position(
    states: &Arc<Mutex<ButtonStateMap>>,
    id: &str,
    duration: f64,
    loop_mode: bool,
    delta_s: Option<f64>,
    position_s: Option<f64>,
) -> Option<f64> {
    if duration <= 0.0 {
        return None;
    }
    let current = states
        .lock()
        .unwrap()
        .get(id)?
        .iter()
        .rev()
        .find(|s| !s.is_done())?
        .position();
    let raw = position_s.unwrap_or(current + delta_s.unwrap_or(0.0));
    if loop_mode {
        Some(raw.rem_euclid(duration))
    } else {
        Some(raw.clamp(0.0, (duration - 0.02).max(0.0)))
    }
}
