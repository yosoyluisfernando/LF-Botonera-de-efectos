//! Pruebas de los VUMETROS contra las tarjetas reales del equipo.
//!
//! Van marcadas `#[ignore]` porque necesitan hardware. Se piden a mano:
//!
//! ```text
//! cargo test --test consola_vumetros_real -- --ignored --nocapture --test-threads=1
//! ```
//!
//! **Cierra la aplicacion antes**: abren las mismas tarjetas.
//!
//! Comprueban lo que sostiene una tira de canal por bus: que cada medidor cuenta
//! LO SUYO, que el del programa cuenta la suma, y que mover un fader mueve solo
//! las agujas que debe. Si se pisaran, una tira enseñaria el audio de otra.
mod common;
use common::{dos_tarjetas, nivel, respirar, tono};
use tauri_app_lib::domain::console::{BusId, Routing};
use tauri_app_lib::engine::console::ConsoleEngine;

/// Lo que sostiene los vumetros por bus: cada medidor cuenta LO SUYO, y el del
/// programa la suma. Si se pisaran, una tira de canal enseñaria el audio de otra.
#[test]
#[ignore]
fn cada_bus_mide_lo_suyo_y_el_programa_la_suma() {
    let Some((tarjeta, _)) = dos_tarjetas() else {
        return;
    };
    let consola = ConsoleEngine::new();
    consola.set_bus_routing_sync(BusId::Programa, Routing::Device(tarjeta.clone()));
    // Valores que no saturan al sumarse: el medidor recorta a 1.0 y no se veria.
    consola.bus(BusId::Efectos).unwrap().add(tono(0.5));
    consola.bus(BusId::Panel).unwrap().add(tono(0.25));
    respirar();

    let (efe, _) = consola.levels(BusId::Efectos);
    let (pan, _) = consola.levels(BusId::Panel);
    let (rep, _) = consola.levels(BusId::Reproductor);
    let (prog, _) = consola.levels(BusId::Programa);
    println!(
        "efectos {} · panel {} · reproductor {} · programa {}",
        nivel(&efe),
        nivel(&pan),
        nivel(&rep),
        nivel(&prog)
    );
    assert!((nivel(&efe) - 0.5).abs() < 0.01, "efectos: {}", nivel(&efe));
    assert!((nivel(&pan) - 0.25).abs() < 0.01, "panel: {}", nivel(&pan));
    assert!(
        nivel(&rep) < 0.01,
        "el reproductor esta callado y su aguja debe estar quieta: {}",
        nivel(&rep)
    );
    assert!(
        (nivel(&prog) - 0.75).abs() < 0.01,
        "el programa debe medir la suma de los dos: {}",
        nivel(&prog)
    );
}

/// El fader de un bus mueve su aguja y la del programa, pero no la de sus
/// vecinos. Es lo que hace util una consola: bajar la musica sin tocar nada mas.
#[test]
#[ignore]
fn bajar_un_bus_mueve_su_aguja_y_la_del_programa_solamente() {
    let Some((tarjeta, _)) = dos_tarjetas() else {
        return;
    };
    let consola = ConsoleEngine::new();
    consola.set_bus_routing_sync(BusId::Programa, Routing::Device(tarjeta));
    consola.bus(BusId::Efectos).unwrap().add(tono(0.5));
    consola.bus(BusId::Reproductor).unwrap().add(tono(0.25));
    respirar();

    let (efe, _) = consola.levels(BusId::Efectos);
    let (rep, _) = consola.levels(BusId::Reproductor);
    let (prog, _) = consola.levels(BusId::Programa);
    assert!((nivel(&prog) - 0.75).abs() < 0.01, "de partida: {}", nivel(&prog));

    // Se baja la musica a la mitad, como para hablar encima.
    consola.set_fader(BusId::Reproductor, 0.5);
    respirar();
    println!(
        "tras bajar el reproductor → efectos {} · reproductor {} · programa {}",
        nivel(&efe),
        nivel(&rep),
        nivel(&prog)
    );
    assert!(
        (nivel(&efe) - 0.5).abs() < 0.01,
        "los efectos no se enteran: {}",
        nivel(&efe)
    );
    assert!(
        (nivel(&rep) - 0.125).abs() < 0.01,
        "el reproductor baja a la mitad: {}",
        nivel(&rep)
    );
    assert!(
        (nivel(&prog) - 0.625).abs() < 0.01,
        "el programa refleja la nueva suma: {}",
        nivel(&prog)
    );
}

/// El master es el fader del programa: mueve SU aguja, no la de los buses que
/// suman en el. Cada tira enseña lo que ella aporta, no lo que sale al aire.
#[test]
#[ignore]
fn el_master_mueve_la_aguja_del_programa_pero_no_las_de_sus_buses() {
    let Some((tarjeta, _)) = dos_tarjetas() else {
        return;
    };
    let consola = ConsoleEngine::new();
    consola.set_bus_routing_sync(BusId::Programa, Routing::Device(tarjeta));
    consola.bus(BusId::Efectos).unwrap().add(tono(0.8));
    respirar();

    let (efe, _) = consola.levels(BusId::Efectos);
    let (prog, _) = consola.levels(BusId::Programa);
    consola.set_fader(BusId::Programa, 0.5);
    respirar();
    println!("con el master a la mitad → efectos {} · programa {}", nivel(&efe), nivel(&prog));
    assert!(
        (nivel(&efe) - 0.8).abs() < 0.01,
        "el bus de efectos aporta lo mismo: {}",
        nivel(&efe)
    );
    assert!(
        (nivel(&prog) - 0.4).abs() < 0.01,
        "el programa sale a la mitad: {}",
        nivel(&prog)
    );
}

