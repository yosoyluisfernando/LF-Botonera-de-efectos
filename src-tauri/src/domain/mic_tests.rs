use super::*;

#[test]
fn absolute_target_preserves_normal_levels_and_never_boosts() {
    for base in [0.0, 0.1, 0.4, 0.6, 1.0, 1.5] {
        let ducked = base * attenuation(base, 1.0, 0.2);
        assert!((ducked - base.min(0.2)).abs() < 0.00001);
        assert_eq!(base * 1.0, base);
    }
    assert!((0.6 * attenuation(1.0, 0.6, 0.2) - 0.2).abs() < 0.00001);
    assert!((0.4 * 0.6 * attenuation(0.4, 0.6, 0.2) - 0.2).abs() < 0.00001);
}

#[test]
fn hold_tracks_independent_sources_and_local_cancellation() {
    let mut s = MicState::default();
    s.input(MicMode::Hold, "pointer".into(), true);
    s.input(MicMode::Hold, "global:1".into(), true);
    s.release_local();
    assert!(s.active(MicMode::Hold));
    s.input(MicMode::Hold, "global:1".into(), false);
    assert!(!s.active(MicMode::Hold));
}

#[test]
fn toggling_ignores_repeat_and_release_does_not_unlatch() {
    let mut s = MicState::default();
    s.input(MicMode::Toggle, "local:KeyA".into(), true);
    s.input(MicMode::Toggle, "local:KeyA".into(), true);
    s.input(MicMode::Toggle, "local:KeyA".into(), false);
    assert!(s.active(MicMode::Toggle));
    s.input(MicMode::Toggle, "local:KeyA".into(), true);
    assert!(!s.active(MicMode::Toggle));
}

#[test]
fn detector_waits_for_voice_and_silence_with_hysteresis() {
    let cfg = MicConfig::default();
    let mut s = MicState::default();
    s.input(MicMode::Auto, "pointer".into(), true);
    s.detect(-20.0, 50, &cfg);
    assert!(!s.active(MicMode::Auto));
    s.detect(-20.0, 50, &cfg);
    assert!(s.active(MicMode::Auto));
    s.detect(-31.0, 2000, &cfg);
    assert!(s.voice);
    s.detect(-80.0, 999, &cfg);
    assert!(s.voice);
    s.detect(-20.0, 10, &cfg);
    s.detect(-80.0, 999, &cfg);
    assert!(s.voice);
    s.detect(-80.0, 1, &cfg);
    assert!(!s.voice);
}

#[test]
fn automatic_suspension_restores_and_clears_detection() {
    let cfg = MicConfig::default();
    let mut s = MicState::default();
    s.input(MicMode::Auto, "pointer".into(), true);
    s.detect(-10.0, 500, &cfg);
    s.input(MicMode::Auto, "pointer".into(), false);
    s.input(MicMode::Auto, "pointer".into(), true);
    s.detect(-10.0, 500, &cfg);
    assert!(!s.active(MicMode::Auto));
    assert!(!s.voice);
}

#[test]
fn measuring_input_does_not_arm_automatic_attenuation() {
    let cfg = MicConfig::default();
    let mut s = MicState::default();
    for _ in 0..100 {
        s.detect(-10.0, 50, &cfg);
    }
    assert!(!s.listening);
    assert!(!s.voice);
    assert!(!s.active(MicMode::Auto));
    s.input(MicMode::Auto, "pointer".into(), true);
    s.detect(-10.0, 50, &cfg);
    assert!(!s.active(MicMode::Auto));
    s.detect(-10.0, 50, &cfg);
    assert!(s.active(MicMode::Auto));
}

#[test]
fn old_configs_load_with_mic_inactive_and_no_shortcut() {
    let cfg: crate::model::AppConfig = serde_json::from_str("{}").unwrap();
    assert_eq!(cfg.mic, MicConfig::default());
    assert!(!MicState::default().active(cfg.mic.mode));
    let audio: crate::model::AudioConfig = serde_json::from_str("{}").unwrap();
    assert!(audio.key_mic.is_empty());
    let mut mic = cfg.mic;
    mic.level = f32::NAN;
    assert!(mic.validate().is_err());
    mic.level = 0.2;
    mic.main = false;
    mic.panel = false;
    mic.player = false;
    assert!(mic.validate().is_err());
}
