use super::*;
#[test]
fn remapping_tab_two_preserves_tab_one_locution_panel_and_cue_states() {
    use crate::engine::{
        audio::button::{ButtonState, PlaybackGroup},
        cache::preload::PreloadCache,
    };
    use std::sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc::channel,
        Mutex,
    };
    use std::time::Instant;
    let mut rig = Rig::new();
    rig.targets
        .insert(BusId::Salida(0), Routing::Device("USB".into()));
    rig.update(&[]);
    let make = |group, output: Bus| ButtonState {
        group,
        output: Some(output),
        done_flag: Arc::new(AtomicBool::new(false)),
        stop_flag: Arc::new(AtomicBool::new(false)),
        fade_out_flag: None,
        volume: Arc::new(AtomicU32::new(1.0f32.to_bits())),
        start_time: Instant::now(),
        position_offset_s: 0.0,
        duration: 20.0,
        loop_mode: false,
        replay: None,
    };
    let one = make(PlaybackGroup::Main, rig.state.live[&BusId::Efectos].clone());
    let two = make(PlaybackGroup::Main, rig.state.live[&BusId::Efectos].clone());
    let fixed = make(PlaybackGroup::Fixed, rig.state.live[&BusId::Panel].clone());
    let cue = make(PlaybackGroup::Cue, rig.state.live[&BusId::Cue].clone());
    let one_flag = one.stop_flag.clone();
    let two_flag = two.stop_flag.clone();
    let start = one.start_time;
    let states = Arc::new(Mutex::new(HashMap::from([
        ("p_one_btn_1".into(), vec![one]),
        ("p_two_btn_1".into(), vec![two]),
        ("fixed".into(), vec![fixed]),
        ("preview".into(), vec![cue]),
    ])));
    rig.state.tabs.insert("p_two".into(), BusId::Salida(0));
    let (tx, _rx) = channel();
    let console = crate::engine::console::ConsoleEngine {
        tx,
        state: Arc::new(Mutex::new(rig.state)),
    };
    crate::domain::playback::reattach::reattach_all(
        &states,
        &console,
        &Arc::new(Mutex::new(PreloadCache::new(128))),
    );
    let map = states.lock().unwrap();
    assert_eq!(map.len(), 3);
    assert!(!map.contains_key("p_two_btn_1"));
    assert_eq!(map["p_one_btn_1"][0].start_time, start);
    assert!(!one_flag.load(Ordering::Relaxed));
    assert!(two_flag.load(Ordering::Relaxed));
    assert!(!map["fixed"][0].stop_flag.load(Ordering::Relaxed));
    assert!(!map["preview"][0].stop_flag.load(Ordering::Relaxed));
}
