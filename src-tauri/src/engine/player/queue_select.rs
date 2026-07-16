//! Modulo: engine/player/queue_select.rs
//! Proposito: ELEGIR que pista viene y dejarla lista. Envuelve la regla pura de
//! `domain::player` con lo que necesita el runtime: saltar lo no reproducible,
//! la pre-carga ping-pong en el deck ocioso, y mantener el naranja como guia
//! cuando el reproductor esta detenido. El transporte (play/stop/avanzar) vive en
//! `queue_ops.rs`; aqui no se ejecuta nada sobre los decks.
use super::queue::{DeckAction, QueueState};
use crate::domain::player::next_index;

impl QueueState {
    /// Salta a una posicion de lo que suena. No cambia de pista ni de deck: solo
    /// el deck activo sabe que archivo esta sonando de verdad, lo que importa en
    /// una carpeta aleatoria, donde la cola solo conoce la carpeta. Si la pista
    /// no es reposicionable (una locucion), el deck ignora la accion.
    pub fn seek(&mut self, position_s: f64) -> Vec<DeckAction> {
        vec![DeckAction::Seek { deck: self.active_deck, position_s: position_s.max(0.0) }]
    }

    /// Primer candidato (respeta lo marcado), saltando no reproducibles.
    pub(super) fn peek_next(&mut self) -> Option<usize> {
        let base = self.current.or(self.cursor);
        let first = next_index(self.mode, self.entries.len(), base, self.marked, self.rand());
        self.resolve_playable(first)
    }

    /// Avanza desde un candidato hasta hallar uno reproducible (o None).
    pub(super) fn resolve_playable(&mut self, mut cand: Option<usize>) -> Option<usize> {
        for _ in 0..self.entries.len().max(1) {
            match cand {
                Some(i) if self.playable(i) => return Some(i),
                Some(i) => {
                    cand = next_index(self.mode, self.entries.len(), Some(i), None, self.rand());
                }
                None => return None,
            }
        }
        None
    }

    /// Detenido, el naranja debe seguir diciendo QUE VIENE: es la guia con la que
    /// el operador sabe que sonara al pulsar Play. Si no hay nada marcado, marca
    /// lo que arrancaria. Recorre como Normal (Manual no avanza solo, pero Play si
    /// arranca) y vuelve al principio si la lista acabo o aun no ha empezado: por
    /// eso, al anadir a una lista vacia, la primera queda marcada sola.
    pub(super) fn ensure_upcoming_marked(&mut self) {
        if self.entries.is_empty() || self.current.is_some() || self.marked.is_some() {
            return;
        }
        let cand = next_index(self.mode, self.entries.len(), self.cursor, None, self.rand());
        self.marked = self
            .resolve_playable(cand)
            .or_else(|| self.resolve_playable(Some(0)));
    }

    /// Precarga la proxima en el deck ocioso (queda comprometida como siguiente).
    /// La hora y el clima NO se precargan: se resuelven en el relevo, o dirian la
    /// hora de cuando empezo la pista anterior. Siguen marcandose en naranja.
    pub(super) fn preload(&mut self, actions: &mut Vec<DeckAction>) {
        self.loaded_other = None;
        let other = 1 - self.active_deck;
        if let Some(f) = self.peek_next() {
            self.loaded_other = Some(f);
            if !self.entries[f].needs_late_resolve() {
                actions.push(DeckAction::Load {
                    deck: other,
                    entry: self.entries[f].clone(),
                    autoplay: false,
                });
            }
        }
    }
}
