//! Modulo: engine/console/thread.rs
//! Proposito: el hilo guardian de la consola. Unico dueno de las tarjetas
//! abiertas (`EndpointRegistry`), porque `OutputStream` no es Send y tiene que
//! quedarse quieto en algun sitio. Solo atiende cambios de ruteo: reproducir NO
//! pasa por aqui — los motores anaden fuentes al bus desde sus propios hilos.
use super::endpoint::EndpointRegistry;
use super::graph;
use super::{ConsoleCommand, ConsoleState};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub fn run(rx: Receiver<ConsoleCommand>, state: Arc<Mutex<ConsoleState>>) {
    let mut endpoints = EndpointRegistry::default();
    for cmd in rx {
        match cmd {
            ConsoleCommand::SetBusRouting { bus, routing, done } => {
                let same = state
                    .lock()
                    .unwrap()
                    .slots
                    .get(&bus)
                    .is_some_and(|slot| slot.routing == routing);
                // Rehacer el grafo mata las fuentes que estuvieran sonando, asi
                // que no se hace por gusto: reaplicar el mismo ruteo (al
                // arrancar, al reconectar) no debe tocar nada. Que el bus no
                // exista si obliga: hay que abrirlo.
                let live = state.lock().unwrap().live.contains_key(&bus);
                if !(same && live) {
                    if let Some(slot) = state.lock().unwrap().slots.get_mut(&bus) {
                        slot.routing = routing;
                    }
                    graph::rebuild(&mut endpoints, &state);
                }
                // Se avisa siempre, tambien cuando no hubo nada que hacer: quien
                // espera necesita seguir, no quedarse colgado hasta el tope.
                if let Some(done) = done {
                    let _ = done.send(());
                }
            }
        }
    }
}
