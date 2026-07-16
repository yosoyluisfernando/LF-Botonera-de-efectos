//! Pruebas del interruptor "detener al finalizar". Hereda el papel del antiguo
//! modo `manual`, pero **combinandose con el modo activo** en vez de forzar el
//! orden normal, que es lo que aquel hacia y lo que le impedia convivir con
//! aleatorio.
use super::*;
use crate::domain::player::PlayerMode;
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

/// "Detener al finalizar" para al acabar la actual y deja marcada la que tocaba:
/// es lo que hacia el desaparecido modo `manual`.
#[test]
fn stop_after_holds_at_the_end_of_each_track() {
    let mut q = queue_with(&["a", "b", "c"]);
    q.start_at(0);
    q.set_stop_after(true);

    let actions = q.advance(false);

    assert!(stops_all(&actions), "no arranca la siguiente sola");
    assert_eq!(q.current(), None);
    assert_eq!(q.next(), Some(1), "y deja marcada la que tocaba");
}

/// Lo que el modo `manual` NO permitia: pararse en cada pista **respetando el
/// modo**. `manual` forzaba el orden normal para elegir la siguiente, asi que
/// "manual + aleatorio" era imposible. El interruptor se combina con cualquiera.
#[test]
fn stop_after_combines_with_the_active_mode() {
    let mut q = queue_with(&["a", "b", "c", "d"]);
    q.set_mode(PlayerMode::Random);
    q.start_at(0);
    q.set_stop_after(true);

    q.advance(false);

    assert_eq!(q.current(), None, "se para, como hacia manual");
    let marked = q.next().expect("debe dejar marcada la siguiente");
    assert_ne!(marked, 0, "aleatorio no repite la actual");
    // Con `normal` la siguiente seria SIEMPRE la 1; aqui manda el modo elegido.
    assert!(marked < 4);
}

/// Y con el interruptor apagado, cada modo avanza solo como siempre.
#[test]
fn without_stop_after_the_mode_advances_on_its_own() {
    let mut q = queue_with(&["a", "b", "c"]);
    q.set_mode(PlayerMode::Normal);
    q.start_at(0);

    q.advance(false);

    assert_eq!(q.current(), Some(1), "avanza sin intervenir");
}
