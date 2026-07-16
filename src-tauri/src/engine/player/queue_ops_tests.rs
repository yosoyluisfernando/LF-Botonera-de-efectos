//! Pruebas del runtime de la cola (`QueueState`). Sin audio: se comprueban las
//! acciones que el hilo ejecutaria sobre los decks.
use super::*;
use crate::engine::player::queue::{QueueEntry, QueueState};

/// Pista reproducible (ruta no vacia) con id estable.
fn entry(id: &str) -> QueueEntry {
    QueueEntry { id: id.into(), path: format!("C:/{id}.mp3"), duration_s: 10.0, ..Default::default() }
}

fn queue_with(ids: &[&str]) -> QueueState {
    let mut q = QueueState::new();
    q.set_entries(ids.iter().map(|i| entry(i)).collect());
    q
}

fn stops_all(actions: &[DeckAction]) -> bool {
    actions.iter().any(|a| matches!(a, DeckAction::StopAll))
}

/// El requisito: limpiar la lista mientras suena algo NO corta la musica.
#[test]
fn clearing_the_queue_while_playing_does_not_stop_the_music() {
    let mut q = queue_with(&["a", "b"]);
    q.start_at(0);
    assert_eq!(q.current(), Some(0));

    let actions = q.set_entries(Vec::new());

    assert!(!stops_all(&actions), "limpiar la lista no debe detener la musica");
    assert_eq!(q.current(), None, "la pista que suena ya no esta en la lista");
    assert_eq!(q.len(), 0);
}

/// El requisito: abrir otra lista mientras suena algo tampoco corta la musica.
#[test]
fn opening_another_playlist_while_playing_does_not_stop_the_music() {
    let mut q = queue_with(&["a", "b"]);
    q.start_at(0);

    let actions = q.set_entries(vec![entry("x"), entry("y")]);

    assert!(!stops_all(&actions), "abrir otra lista no debe detener la musica");
    assert_eq!(q.current(), None);
    assert_eq!(q.len(), 2);
}

/// Al acabar la pista huerfana, la lista nueva sigue desde el principio.
#[test]
fn orphan_track_hands_over_to_the_new_list_when_it_ends() {
    let mut q = queue_with(&["a", "b"]);
    q.start_at(0);
    q.set_entries(vec![entry("x"), entry("y")]);

    let actions = q.advance(false);

    assert!(!stops_all(&actions), "hay lista nueva: debe continuar, no parar");
    assert_eq!(q.current(), Some(0), "arranca la primera de la lista nueva");
    assert!(actions
        .iter()
        .any(|a| matches!(a, DeckAction::Load { autoplay: true, .. })));
}

/// Marcar siguiente sigue siendo ley aunque la que suena sea huerfana.
#[test]
fn marked_next_is_honoured_while_an_orphan_track_plays() {
    let mut q = queue_with(&["a", "b"]);
    q.start_at(0);
    q.set_entries(vec![entry("x"), entry("y")]);
    q.mark_next(Some(1));

    assert_eq!(q.next(), Some(1), "lo marcado se pinta en naranja");
    q.advance(false);
    assert_eq!(q.current(), Some(1), "lo marcado es ley");
}

/// Lo marcado como siguiente es LEY: al reorganizar la lista, la marca sigue a
/// SU cancion (por id), no se queda en la posicion que ocupaba.
#[test]
fn marked_next_follows_its_track_when_the_queue_is_reordered() {
    let mut q = queue_with(&["a", "b", "c"]);
    q.start_at(0);
    q.mark_next(Some(2)); // marca "c", que esta la tercera
    assert_eq!(q.next(), Some(2));

    // El operador arrastra "c" a la primera posicion.
    q.set_entries(vec![entry("c"), entry("a"), entry("b")]);

    assert_eq!(q.next(), Some(0), "la marca sigue a 'c', no a la posicion 2");
    q.advance(false);
    assert_eq!(q.current(), Some(0), "y al avanzar suena 'c', que era lo marcado");
}

/// Si la cancion marcada se elimina de la lista, la marca se cae (no salta a otra).
#[test]
fn marked_next_is_dropped_when_its_track_is_removed() {
    let mut q = queue_with(&["a", "b", "c"]);
    q.start_at(0);
    q.mark_next(Some(2)); // marca "c"

    q.set_entries(vec![entry("a"), entry("b")]); // se quita "c"

    assert_eq!(q.marked, None, "la marca desaparece con su cancion");
}

/// Editar la cola (reordenar, quitar otras filas) conserva lo que suena.
#[test]
fn editing_the_queue_keeps_the_surviving_current_track() {
    let mut q = queue_with(&["a", "b", "c"]);
    q.start_at(1);
    assert_eq!(q.current(), Some(1));

    let actions = q.set_entries(vec![entry("c"), entry("b"), entry("a")]);

    assert!(!stops_all(&actions));
    assert_eq!(q.current(), Some(1), "'b' sigue presente: se conserva por id");
}

/// El naranja es la guia de "que viene": al pulsar Stop NO debe desaparecer.
#[test]
fn stop_keeps_the_orange_marker_visible() {
    let mut q = queue_with(&["a", "b", "c"]);
    q.start_at(0);
    assert_eq!(q.next(), Some(1), "precargada la 'b'");

    q.stop();

    assert_eq!(q.current(), None, "nada suena");
    assert_eq!(q.next(), Some(1), "pero se sigue viendo que vendra la 'b'");
}

/// Parado, el naranja debe indicar lo que sonaria al pulsar Play.
#[test]
fn stopping_at_the_end_of_the_list_still_shows_what_comes_next() {
    let mut q = queue_with(&["a", "b"]);
    q.start_at(1); // la ultima
    q.advance(false); // Normal: se acaba la lista

    assert_eq!(q.current(), None, "Normal se detiene al final");
    assert!(q.next().is_some(), "pero sigue habiendo guia de por donde retomar");
}

/// Al llenar una lista vacia, la primera queda marcada sola: sin ella el
/// operador no sabria que va a sonar al pulsar Play.
#[test]
fn filling_an_empty_queue_marks_the_first_track_automatically() {
    let mut q = QueueState::new();
    assert_eq!(q.next(), None, "lista vacia: nada que marcar");

    q.set_entries(vec![entry("a"), entry("b")]);

    assert_eq!(q.next(), Some(0), "lo primero queda marcado como siguiente");
}

/// Doble clic con el reproductor detenido: reproduce esa cancion.
#[test]
fn double_click_while_stopped_plays_the_track() {
    let mut q = queue_with(&["a", "b", "c"]);

    let actions = q.activate(2, false);

    assert_eq!(q.current(), Some(2), "detenido: doble clic reproduce");
    assert!(actions
        .iter()
        .any(|a| matches!(a, DeckAction::Load { autoplay: true, .. })));
}

/// Doble clic con algo sonando: marca como siguiente y NO corta la musica.
#[test]
fn double_click_while_playing_marks_next_without_cutting_the_music() {
    let mut q = queue_with(&["a", "b", "c"]);
    q.start_at(0);

    let actions = q.activate(2, true);

    assert_eq!(q.current(), Some(0), "lo que sonaba sigue sonando");
    assert_eq!(q.next(), Some(2), "y la elegida queda en naranja");
    assert!(!stops_all(&actions), "marcar no interrumpe la musica");
}

/// Con la cola vacia y nada que relevar, el fin de pista si detiene.
#[test]
fn advance_on_an_empty_queue_stops() {
    let mut q = queue_with(&["a"]);
    q.start_at(0);
    q.set_entries(Vec::new());

    let actions = q.advance(false);

    assert!(stops_all(&actions), "sin lista no hay nada que arrancar");
    assert_eq!(q.current(), None);
}
