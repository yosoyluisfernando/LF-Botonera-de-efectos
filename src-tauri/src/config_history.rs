/// Modulo: config_history.rs
/// Proposito: historial limitado para deshacer/rehacer cambios destructivos.
use crate::types::AppConfig;

const LIMIT: usize = 10;

#[derive(Default)]
pub struct ConfigHistory {
    undo: Vec<AppConfig>,
    redo: Vec<AppConfig>,
}

impl ConfigHistory {
    pub fn remember(&mut self, current: &AppConfig) {
        self.undo.push(current.clone());
        if self.undo.len() > LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn undo(&mut self, current: &AppConfig) -> Option<AppConfig> {
        let prev = self.undo.pop()?;
        self.redo.push(current.clone());
        if self.redo.len() > LIMIT {
            self.redo.remove(0);
        }
        Some(prev)
    }

    pub fn redo(&mut self, current: &AppConfig) -> Option<AppConfig> {
        let next = self.redo.pop()?;
        self.undo.push(current.clone());
        if self.undo.len() > LIMIT {
            self.undo.remove(0);
        }
        Some(next)
    }
}
