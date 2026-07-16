//! Modulo: domain/console/routing.rs
//! Proposito: las reglas de ruteo de la consola. Puras: no abren tarjetas, no
//! tocan rodio y se prueban sin hardware. Aqui vive la decision de QUE va DONDE;
//! `engine/console/` solo la obedece.

/// Los buses de la consola.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BusId {
    /// Botones de la botonera principal (`PlaybackGroup::Main`).
    Efectos,
    /// Botones del panel fijo (`PlaybackGroup::Fixed`).
    Panel,
    /// Pre-escucha y previa del editor. **Nunca suma en programa.**
    Cue,
    /// El programa (PGM): la suma de los buses que van al aire. Su fader es el
    /// volumen master y su medidor es el vumetro de la barra inferior.
    Programa,
}

/// A donde entrega un bus su señal.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Routing {
    /// Suma en el bus de programa: le pega el master y lo cuenta su vumetro.
    Program,
    /// Sale por la MISMA tarjeta que el programa, pero **sin sumar en el**: no le
    /// pega el master ni lo mide su vumetro.
    ///
    /// Esta variante es la idea entera de la consola. Que dos cosas salgan por el
    /// mismo altavoz no las convierte en la misma señal: se suman en el conector,
    /// no en el bus. Es lo que mantiene privada la pre-escucha cuando solo hay
    /// una tarjeta de sonido.
    ProgramDevice,
    /// Sale por su propia tarjeta, ajeno al programa y al master.
    Device(String),
}

impl BusId {
    /// Los buses que entran en la suma del programa, en orden estable.
    pub const PROGRAM_INPUTS: [BusId; 2] = [BusId::Efectos, BusId::Panel];

    /// Todos los buses que la consola construye.
    pub const ALL: [BusId; 4] = [BusId::Programa, BusId::Efectos, BusId::Panel, BusId::Cue];

    /// El ruteo por defecto de cada bus al arrancar.
    pub fn default_routing(self) -> Routing {
        match self {
            // El programa siempre sale por una tarjeta: es el final del camino.
            // Su nombre real lo pone el perfil activo al arrancar.
            BusId::Programa => Routing::Device(String::new()),
            // La pre-escucha comparte tarjeta con el programa mientras no tenga
            // una propia, pero JAMAS se suma a el.
            BusId::Cue => Routing::ProgramDevice,
            _ => Routing::Program,
        }
    }

    /// Si este bus puede sumar en el programa.
    ///
    /// El CUE no puede, y esa es la regla que justifica toda la consola: una
    /// escucha privada que se cuela en el aire no es una escucha privada. El
    /// programa tampoco, porque no puede sumarse a si mismo.
    pub fn can_sum_into_program(self) -> bool {
        Self::PROGRAM_INPUTS.contains(&self)
    }
}

/// Corrige un ruteo imposible antes de que llegue al motor. Devuelve el que se
/// va a aplicar de verdad.
///
/// No es programacion defensiva: es la regla de negocio. Pedir que la
/// pre-escucha suene "en el programa" no es un error de un llamante que haya que
/// silenciar, es una peticion que la consola traduce a lo unico que significa —
/// que suene por la tarjeta del programa, pero aparte.
pub fn sanitize(bus: BusId, requested: Routing) -> Routing {
    match (bus, &requested) {
        // El programa es el final del camino: no puede sumarse a si mismo ni
        // "seguir" a nadie.
        (BusId::Programa, Routing::Program | Routing::ProgramDevice) => {
            Routing::Device(String::new())
        }
        (bus, Routing::Program) if !bus.can_sum_into_program() => Routing::ProgramDevice,
        _ => requested,
    }
}

/// La tarjeta por la que sale de verdad un bus, o None si no tiene salida propia
/// porque va sumado dentro del programa.
pub fn device_of(routing: &Routing, program_device: &str) -> Option<String> {
    match routing {
        Routing::Program => None,
        Routing::ProgramDevice => Some(program_device.to_string()),
        Routing::Device(name) => Some(name.clone()),
    }
}

/// Que tarjetas hacen falta para los buses vivos que se le pasen. Un bus sumado
/// en el programa no retiene ninguna: la del programa es. Las tarjetas que no
/// salgan aqui se cierran — una tarjeta abierta que nadie escucha puede dejar sin
/// ella a otro programa (WASAPI en exclusivo) y no aporta nada.
pub fn devices_in_use(live: &[(BusId, Routing)], program_device: &str) -> Vec<String> {
    live.iter()
        .filter_map(|(_, routing)| device_of(routing, program_device))
        .filter(|name| !name.is_empty())
        .collect()
}


#[cfg(test)]
#[path = "routing_tests.rs"]
mod routing_tests;
