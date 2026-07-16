//! Modulo: engine/player/queue_edit.rs
//! Proposito: CONFIGURAR la cola: reemplazar su contenido, fijar el modo de
//! avance y los interruptores de transporte (marcar siguiente, detener al
//! finalizar, Loop). Nada de esto arranca ni detiene audio por si mismo: solo
//! cambia lo que pasara despues. El transporte vive en `queue_ops.rs`.
use super::queue::{DeckAction, QueueEntry, QueueState};
use crate::domain::player::PlayerMode;

impl QueueState {
    /// Reemplaza la cola conservando por `id` lo que siga presente.
    ///
    /// **Nunca corta el audio.** Si la pista que suena desaparece de la cola
    /// nueva (al limpiar o al abrir otra lista), sigue sonando hasta su fin: la
    /// musica de fondo no se detiene porque se edite la lista. Queda "huerfana"
    /// (`current = None`, nada en verde: ya no esta en la lista), y al terminar,
    /// `advance` arranca la lista nueva desde el principio. Es el criterio del
    /// Automatizador, cuyo `clearList` vacia las filas sin tocar la reproduccion.
    pub fn set_entries(&mut self, entries: Vec<QueueEntry>) -> Vec<DeckAction> {
        let current_id = self.current.and_then(|i| self.entries.get(i)).map(|e| e.id.clone());
        let cursor_id = self.cursor.and_then(|i| self.entries.get(i)).map(|e| e.id.clone());
        let marked_id = self.marked.and_then(|i| self.entries.get(i)).map(|e| e.id.clone());
        self.entries = entries;
        self.current = current_id.as_deref().and_then(|id| self.index_of(id));
        self.cursor = cursor_id.as_deref().and_then(|id| self.index_of(id));
        self.marked = marked_id.as_deref().and_then(|id| self.index_of(id)).filter(|&i| self.playable(i));
        self.loaded_other = None;
        let mut actions = Vec::new();
        if self.current.is_some() {
            self.preload(&mut actions);
        } else {
            self.ensure_upcoming_marked();
        }
        actions
    }

    pub fn set_mode(&mut self, mode: PlayerMode) -> Vec<DeckAction> {
        self.mode = mode;
        let mut actions = Vec::new();
        // Solo recalcula la siguiente si hay algo sonando; detenido no debe
        // aparecer una "siguiente" en naranja (misma guarda que set_entries).
        if self.current.is_some() {
            self.preload(&mut actions);
        }
        actions
    }

    pub fn set_stop_after(&mut self, value: bool) {
        self.stop_after = value;
    }

    /// Boton Loop. No toca lo que suena: solo cambia que pasa AL TERMINAR, asi
    /// que ponerlo o quitarlo a mitad de una cancion no corta el audio.
    pub fn set_loop_current(&mut self, value: bool) {
        self.loop_current = value;
    }

    pub fn mark_next(&mut self, index: Option<usize>) -> Vec<DeckAction> {
        self.marked = index.filter(|&i| self.playable(i));
        let mut actions = Vec::new();
        if self.current.is_some() {
            self.preload(&mut actions);
        } else {
            self.loaded_other = None;
        }
        actions
    }
}
