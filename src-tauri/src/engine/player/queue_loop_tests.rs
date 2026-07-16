//! Pruebas del boton Loop (repetir la cancion actual). Lo que se comprueba aqui
//! es su relacion con el resto de reglas: manda sobre "detener al finalizar",
//! cede ante el boton Siguiente, y no toca lo marcado como siguiente.
use super::*;
use crate::engine::player::queue::{QueueEntry, QueueState};

fn entry(id: &str) -> QueueEntry {
    QueueEntry { id: id.into(), kind: "audio".into(), path: format!("C:/{id}.mp3"),
        duration_s: 10.0, ..Default::default() }
}

fn queue_with(ids: &[&str]) -> QueueState {
    let mut q = QueueState::new();
    q.set_entries(ids.iter().map(|i| entry(i)).collect());
    q
}

fn stops_all(actions: &[DeckAction]) -> bool {
    actions.iter().any(|a| matches!(a, DeckAction::StopAll))
}

/// Loop: al terminar, la misma cancion vuelve a empezar en vez de avanzar.
#[test]
fn loop_replays_the_same_track_instead_of_advancing() {
    let mut q = queue_with(&["a", "b", "c"]);
    q.start_at(0);
    q.set_loop_current(true);

    let actions = q.advance(false);

    assert_eq!(q.current(), Some(0), "sigue sonando la misma");
    assert!(actions
        .iter()
        .any(|a| matches!(a, DeckAction::Load { autoplay: true, .. })));
    assert!(!stops_all(&actions));
}

/// El boton Siguiente es del operador y manda sobre el Loop.
#[test]
fn the_next_button_wins_over_loop() {
    let mut q = queue_with(&["a", "b", "c"]);
    q.start_at(0);
    q.set_loop_current(true);

    q.advance(true); // forzado = boton Siguiente

    assert_eq!(q.current(), Some(1), "avanza aunque el Loop este puesto");
}

/// "Detener al finalizar" no llega a actuar con el Loop puesto: la cancion no
/// termina nunca. Es lo acordado; al quitar el Loop, el stop vuelve a mandar.
#[test]
fn loop_wins_over_stop_after() {
    let mut q = queue_with(&["a", "b"]);
    q.start_at(0);
    q.set_stop_after(true);
    q.set_loop_current(true);

    let actions = q.advance(false);

    assert_eq!(q.current(), Some(0), "se repite: el stop no actua");
    assert!(!stops_all(&actions));
}

/// El Loop dice CUANDO acaba la actual, no QUE viene despues: lo marcado sigue
/// en naranja esperando, y suena en cuanto se quita el Loop.
#[test]
fn loop_keeps_the_marked_track_waiting_in_orange() {
    let mut q = queue_with(&["a", "b", "c"]);
    q.start_at(0);
    q.mark_next(Some(2));
    q.set_loop_current(true);

    q.advance(false);
    assert_eq!(q.current(), Some(0), "se repite");
    assert_eq!(q.next(), Some(2), "lo marcado sigue esperando");

    q.set_loop_current(false);
    q.advance(false);
    assert_eq!(q.current(), Some(2), "al quitar el Loop, suena lo marcado");
}
