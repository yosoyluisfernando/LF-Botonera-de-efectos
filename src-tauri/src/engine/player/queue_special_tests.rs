//! Pruebas de los tipos especiales en la cola (carpeta aleatoria, hora, clima).
//! Lo que se comprueba aqui es CUANDO se resuelven: la hora y el clima dependen
//! del momento en que suenan, asi que no pueden precargarse por adelantado.
use super::*;
use crate::engine::player::queue::{QueueEntry, QueueState};

fn entry(id: &str) -> QueueEntry {
    QueueEntry { id: id.into(), kind: "audio".into(), path: format!("C:/{id}.mp3"),
        duration_s: 10.0, ..Default::default() }
}

fn special(id: &str, kind: &str) -> QueueEntry {
    QueueEntry { id: id.into(), kind: kind.into(), folder: "C:/carpeta".into(), ..Default::default() }
}

fn loads(actions: &[DeckAction]) -> bool {
    actions.iter().any(|a| matches!(a, DeckAction::Load { .. }))
}

/// Una locucion horaria NO se precarga: si se cargara mientras suena la pista
/// anterior, diria la hora de hace varios minutos. Pero si se marca en naranja.
#[test]
fn a_time_locution_is_marked_next_but_not_preloaded() {
    let mut q = QueueState::new();
    q.set_entries(vec![entry("a"), special("hora", "time")]);

    let actions = q.start_at(0);

    assert_eq!(q.next(), Some(1), "se ve en naranja que viene la hora");
    let preloads: Vec<_> = actions
        .iter()
        .filter(|a| matches!(a, DeckAction::Load { autoplay: false, .. }))
        .collect();
    assert!(preloads.is_empty(), "la hora no debe precargarse");
}

/// Y al relevarla se CARGA de verdad (no se reanuda un deck vacio), que es lo
/// que la resuelve con la hora del momento.
#[test]
fn a_time_locution_is_loaded_fresh_when_its_turn_comes() {
    let mut q = QueueState::new();
    q.set_entries(vec![entry("a"), special("hora", "time")]);
    q.start_at(0);

    let actions = q.advance(false);

    assert_eq!(q.current(), Some(1));
    assert!(loads(&actions), "debe cargarse ahora, con la hora de este momento");
    assert!(
        !actions.iter().any(|a| matches!(a, DeckAction::Resume { .. })),
        "reanudar un deck vacio dejaria la locucion muda"
    );
}

/// Una carpeta aleatoria si se precarga: elegir la cancion antes no la estropea.
#[test]
fn a_random_folder_is_preloaded_normally() {
    let mut q = QueueState::new();
    q.set_entries(vec![entry("a"), special("rnd", "random_folder")]);

    let actions = q.start_at(0);

    assert!(actions
        .iter()
        .any(|a| matches!(a, DeckAction::Load { autoplay: false, .. })));
}

