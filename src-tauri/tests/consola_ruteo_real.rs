//! Pruebas de la consola contra las TARJETAS DE SONIDO REALES del equipo.
//!
//! Van marcadas `#[ignore]` porque necesitan hardware: en un equipo sin salidas
//! —o en CI— no pueden pasar, y una prueba que falla por el entorno no dice nada.
//! Se piden a mano:
//!
//! ```text
//! cargo test --test consola_ruteo_real -- --ignored --nocapture --test-threads=1
//! ```
//!
//! **Cierra la aplicacion antes**: abren las mismas tarjetas.
//!
//! Aqui va el RUTEO: por donde sale cada bus y que pasa al cambiarlo. Los
//! medidores estan en `consola_vumetros_real.rs`.
mod common;
use common::{dos_tarjetas, nivel, respirar, tono};
use std::thread::sleep;
use std::time::Duration;
use tauri_app_lib::domain::console::{BusId, Routing};
use tauri_app_lib::engine::console::ConsoleEngine;

/// El caso que el autor reporto: el reproductor en LA MISMA tarjeta que la
/// botonera debe verse en el vumetro. Elegir esa tarjeta por su nombre es pedir
/// el programa.
#[test]
#[ignore]
fn el_reproductor_en_la_tarjeta_del_programa_suma_en_el() {
    let Some((tarjeta, _)) = dos_tarjetas() else {
        return;
    };
    let consola = ConsoleEngine::new();
    consola.set_bus_routing_sync(BusId::Programa, Routing::Device(tarjeta.clone()));
    // Por su NOMBRE, no con "la misma que los efectos".
    consola.set_bus_routing_sync(BusId::Reproductor, Routing::Device(tarjeta.clone()));
    let bus = consola.bus(BusId::Reproductor).expect("el bus deberia existir");
    bus.add(tono(0.8));
    respirar();

    let (l, _r) = consola.levels(BusId::Programa);
    println!("[{tarjeta}] reproductor por nombre → programa: {}", nivel(&l));
    assert!(
        nivel(&l) > 0.5,
        "pedir la tarjeta del programa es pedir el programa: {}",
        nivel(&l)
    );
}

/// Sacar el reproductor a OTRA tarjeta lo saca del programa: deja de contar en su
/// vumetro, pero sigue sonando y midiendo en el suyo.
#[test]
#[ignore]
fn el_reproductor_en_otra_tarjeta_sale_del_programa() {
    let Some((una, otra)) = dos_tarjetas() else {
        eprintln!("hacen falta DOS tarjetas; se salta");
        return;
    };
    let consola = ConsoleEngine::new();
    consola.set_bus_routing_sync(BusId::Programa, Routing::Device(una.clone()));
    consola.set_bus_routing_sync(BusId::Reproductor, Routing::Device(otra.clone()));
    let bus = consola.bus(BusId::Reproductor).expect("el bus deberia existir");
    bus.add(tono(0.8));
    respirar();

    let (prog, _) = consola.levels(BusId::Programa);
    let (repro, _) = consola.levels(BusId::Reproductor);
    println!("[{una} | {otra}] programa: {} · reproductor: {}", nivel(&prog), nivel(&repro));
    assert!(
        nivel(&repro) > 0.5,
        "el reproductor deberia medir en su propio bus: {}",
        nivel(&repro)
    );
    assert!(
        nivel(&prog) < 0.01,
        "en otra tarjeta NO debe contar en el programa: {}",
        nivel(&prog)
    );
}

/// La prueba del bug de los buses fantasma, con hardware: rehacer el grafo SIN
/// cambiar la tarjeta del programa. Si el bus viejo siguiera vivo dentro de ella,
/// escribiria cero en el mismo medidor y el vumetro parpadearia.
#[test]
#[ignore]
fn rehacer_el_grafo_no_deja_buses_fantasma_midiendo() {
    let Some((una, otra)) = dos_tarjetas() else {
        return;
    };
    let consola = ConsoleEngine::new();
    consola.set_bus_routing_sync(BusId::Programa, Routing::Device(una.clone()));
    let bus = consola.bus(BusId::Efectos).unwrap();
    bus.add(tono(0.8));
    respirar();

    // Se rehace el grafo cinco veces moviendo OTRO bus: la tarjeta del programa
    // no cambia, asi que su endpoint no se cierra — que es cuando aparecian los
    // fantasmas.
    for i in 0..5 {
        let destino = if i % 2 == 0 {
            Routing::Device(otra.clone())
        } else {
            Routing::ProgramDevice
        };
        consola.set_bus_routing_sync(BusId::Cue, destino);
    }
    // El bus de efectos murio con cada rebuild: se le devuelve el tono, como hace
    // `reattach` en el motor de verdad.
    consola.bus(BusId::Efectos).unwrap().add(tono(0.8));
    respirar();

    // Se mira varias veces: un fantasma pisando el medidor da ceros a rachas, y
    // una sola lectura podria caer en un momento bueno por suerte.
    let (l, _r) = consola.levels(BusId::Programa);
    let mut lecturas = Vec::new();
    for _ in 0..10 {
        lecturas.push(nivel(&l));
        sleep(Duration::from_millis(50));
    }
    println!("[{una}] tras 5 reconstrucciones: {lecturas:?}");
    assert!(
        lecturas.iter().all(|&v| v > 0.5),
        "ningun tick debe venir a cero: un fantasma esta pisando el medidor: {lecturas:?}"
    );
}

/// Cambiar la tarjeta del programa de una a otra y volver. El medidor debe seguir
/// midiendo en la nueva, sin quedarse mudo ni colgado.
#[test]
#[ignore]
fn cambiar_de_tarjeta_deja_el_medidor_vivo_en_la_nueva() {
    let Some((una, otra)) = dos_tarjetas() else {
        eprintln!("hacen falta DOS tarjetas; se salta");
        return;
    };
    let consola = ConsoleEngine::new();
    let (l, _r) = consola.levels(BusId::Programa);

    for (i, tarjeta) in [&una, &otra, &una].iter().enumerate() {
        consola.set_bus_routing_sync(BusId::Programa, Routing::Device((*tarjeta).clone()));
        // Cada cambio mata las fuentes: el motor de verdad las rehace con
        // `reattach`, y aqui se hace a mano lo mismo.
        consola.bus(BusId::Efectos).unwrap().add(tono(0.8));
        respirar();
        let medido = nivel(&l);
        println!("paso {i} → [{tarjeta}]: {medido}");
        assert!(
            medido > 0.5,
            "tras cambiar a {tarjeta} el medidor deberia seguir vivo: {medido}"
        );
    }
}
