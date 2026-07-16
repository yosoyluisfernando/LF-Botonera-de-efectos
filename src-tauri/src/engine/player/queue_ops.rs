//! Modulo: engine/player/queue_ops.rs
//! Proposito: TRANSPORTE de la cola (`QueueState`): arrancar, detener, avanzar y
//! relevar entre decks. Devuelve acciones que el hilo ejecuta; no toca audio.
//! Configurar la cola (contenido, modo, interruptores) vive en `queue_edit.rs`;
//! elegir que pista viene y pre-cargarla, en `queue_select.rs`.
use super::queue::{DeckAction, QueueState};

impl QueueState {
    /// Empieza a reproducir el indice dado (accion explicita del operador).
    pub fn start_at(&mut self, index: usize) -> Vec<DeckAction> {
        if !self.playable(index) {
            return Vec::new();
        }
        self.active_deck = 0;
        self.current = Some(index);
        self.cursor = Some(index);
        self.marked = None;
        // Arranque explicito: corta cualquier deck en curso (evita solapar con el
        // deck contrario) y carga la pista pedida desde cero en el deck 0.
        let mut actions = vec![
            DeckAction::StopAll,
            DeckAction::Load { deck: 0, entry: self.entries[index].clone(), autoplay: true },
        ];
        self.preload(&mut actions);
        actions
    }

    /// Doble clic en una fila. Detenido: la reproduce. Sonando: la marca como
    /// siguiente, sin cortar la musica (marcar es ley, asi que sonara al acabar).
    /// `is_playing` lo aporta el hilo, que es quien conoce los decks: una pista
    /// huerfana puede sonar sin estar ya en la cola.
    pub fn activate(&mut self, index: usize, is_playing: bool) -> Vec<DeckAction> {
        if is_playing {
            self.mark_next(Some(index))
        } else {
            self.start_at(index)
        }
    }

    /// Reanuda tras una parada arrancando la siguiente (respeta lo marcado).
    pub fn resume_next(&mut self) -> Vec<DeckAction> {
        self.advance(true)
    }

    pub fn prev(&mut self) -> Vec<DeckAction> {
        let target = self.current.or(self.cursor).map_or(0, |c| c.saturating_sub(1));
        self.start_at(target)
    }

    /// Detiene todo. El naranja NO se pierde: lo que estaba precargado pasa a
    /// marcado, para que se siga viendo que vendra al pulsar Play.
    pub fn stop(&mut self) -> Vec<DeckAction> {
        self.marked = self.marked.or(self.loaded_other);
        self.current = None;
        self.loaded_other = None;
        self.ensure_upcoming_marked();
        vec![DeckAction::StopAll]
    }

    /// Avanza a la siguiente pista. `forced` = accion del operador (ignora el
    /// "detener al finalizar"); no forzado = fin natural de la pista.
    pub fn advance(&mut self, forced: bool) -> Vec<DeckAction> {
        if self.entries.is_empty() {
            self.current = None;
            return vec![DeckAction::StopAll];
        }
        // Loop: la pista actual vuelve a empezar en vez de avanzar. Va lo primero
        // porque manda sobre todo lo demas: mientras este puesto, la cancion no
        // "termina", asi que "detener al finalizar" no llega a actuar. Solo en fin
        // natural: el boton Siguiente (`forced`) es del operador y avanza igual.
        // Lo marcado sigue en naranja, esperando su turno: el Loop dice CUANDO
        // acaba esta, no QUE viene despues.
        if !forced && self.loop_current {
            if let Some(current) = self.current.filter(|&i| self.playable(i)) {
                return vec![DeckAction::Load {
                    deck: self.active_deck,
                    entry: self.entries[current].clone(),
                    autoplay: true,
                }];
            }
        }
        let preloaded = self.loaded_other.take();
        let target = match preloaded {
            Some(t) => Some(t),
            None => self.peek_next(),
        };
        let Some(target) = target else {
            // Fin de la lista: para, pero deja en naranja por donde se retomaria.
            self.current = None;
            self.ensure_upcoming_marked();
            return vec![DeckAction::StopAll];
        };
        if !forced && self.stop_after {
            // Detener al finalizar: no arranca sola; lo marcado sigue siendo ley.
            self.marked = Some(target);
            self.current = None;
            return vec![DeckAction::StopAll];
        }
        let other = 1 - self.active_deck;
        let mut actions = Vec::new();
        // Solo se reanuda lo que de verdad quedo precargado: la hora y el clima
        // se saltan la pre-carga a proposito y hay que resolverlos AHORA.
        let ready = preloaded == Some(target) && !self.entries[target].needs_late_resolve();
        if ready {
            actions.push(DeckAction::Resume { deck: other });
        } else {
            actions.push(DeckAction::Load { deck: other, entry: self.entries[target].clone(), autoplay: true });
        }
        // Corta el deck saliente: en un relevo forzado (boton Siguiente) aun suena;
        // en un fin natural ya esta vacio. Sin esto, dos pistas sonarian a la vez.
        actions.push(DeckAction::Stop { deck: self.active_deck });
        self.active_deck = other;
        self.current = Some(target);
        self.cursor = Some(target);
        self.marked = None;
        self.preload(&mut actions);
        actions
    }
}

#[cfg(test)]
#[path = "queue_ops_tests.rs"]
mod queue_ops_tests;

#[cfg(test)]
#[path = "queue_special_tests.rs"]
mod queue_special_tests;

#[cfg(test)]
#[path = "queue_loop_tests.rs"]
mod queue_loop_tests;

#[cfg(test)]
#[path = "queue_stop_after_tests.rs"]
mod queue_stop_after_tests;
