//! Modulo: engine/player/monitor.rs
//! Proposito: hilo que emite "player-tick" con el estado en vivo del reproductor.
//!
//! El reproductor tiene motor propio, asi que necesita su propio pulso. El tick
//! de los efectos ("audio-tick") NO se emite en reposo, y la musica de fondo
//! suele sonar sin ningun efecto disparado: colgar de aquel tick dejaba la lista
//! sin pintar (ni verde, ni naranja, ni tiempo). Mismo patron que
//! `engine/audio/monitor.rs`, pero independiente, como los dos motores.
use super::PlayerSnapshot;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Emitter;

const TICK: Duration = Duration::from_millis(100);

/// Emite el snapshot mientras suene algo (el tiempo avanza) o cuando cambie
/// (marcar siguiente, editar la cola, parar). En reposo no emite: sin musica ni
/// cambios no hay nada que repintar.
pub fn start(app: tauri::AppHandle, snapshot: Arc<Mutex<PlayerSnapshot>>) {
    thread::spawn(move || {
        let mut last: Option<PlayerSnapshot> = None;
        loop {
            let snap = snapshot.lock().unwrap().clone();
            if should_emit(&snap, last.as_ref()) {
                let _ = app.emit("player-tick", snap.clone());
                last = Some(snap);
            }
            thread::sleep(TICK);
        }
    });
}

fn should_emit(snap: &PlayerSnapshot, last: Option<&PlayerSnapshot>) -> bool {
    snap.playing || last != Some(snap)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(playing: bool, current: Option<u32>) -> PlayerSnapshot {
        PlayerSnapshot { playing, current_index: current, ..Default::default() }
    }

    #[test]
    fn emits_while_playing_so_the_time_advances() {
        let s = snap(true, Some(0));
        assert!(should_emit(&s, Some(&s)), "sonando siempre emite: el tiempo corre");
    }

    #[test]
    fn stays_quiet_when_idle_and_unchanged() {
        let s = snap(false, None);
        assert!(!should_emit(&s, Some(&s)), "en reposo y sin cambios no se emite");
    }

    #[test]
    fn emits_once_when_idle_state_changes() {
        // Marcar una siguiente con el reproductor parado debe repintar el naranja.
        let before = snap(false, None);
        let after = PlayerSnapshot { next_index: Some(2), ..before.clone() };
        assert!(should_emit(&after, Some(&before)));
    }

    #[test]
    fn emits_the_first_tick() {
        assert!(should_emit(&snap(false, None), None));
    }
}
