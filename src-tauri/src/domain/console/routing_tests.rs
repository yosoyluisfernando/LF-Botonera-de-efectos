//! Pruebas de las reglas de ruteo de la consola. Van aparte por el limite de
//! 200 lineas por archivo (regla 3).
use super::{device_of, devices_in_use, sanitize, BusId, Routing};

/// El caso que justifica el registro de tarjetas: el programa y la pre-escucha
/// comparten conector, y hace falta UNA tarjeta, no dos.
#[test]
fn programa_y_cue_en_la_misma_tarjeta_solo_necesitan_esa() {
    let live = [
        (BusId::Programa, Routing::Device("Altavoces".into())),
        (BusId::Cue, Routing::ProgramDevice),
        (BusId::Efectos, Routing::Program),
    ];
    let in_use = devices_in_use(&live, "Altavoces");
    assert!(in_use.iter().all(|n| n == "Altavoces"));
}

/// Un bus sumado en el programa no retiene tarjeta propia: no tiene.
#[test]
fn un_bus_sumado_en_programa_no_retiene_tarjeta() {
    let live = [(BusId::Efectos, Routing::Program)];
    assert!(devices_in_use(&live, "Altavoces").is_empty());
}

/// Sacar la pre-escucha a sus auriculares obliga a mantener las dos tarjetas.
#[test]
fn cue_con_tarjeta_propia_retiene_las_dos() {
    let live = [
        (BusId::Programa, Routing::Device("Altavoces".into())),
        (BusId::Cue, Routing::Device("Auriculares".into())),
    ];
    let in_use = devices_in_use(&live, "Altavoces");
    assert!(in_use.contains(&"Altavoces".to_string()));
    assert!(in_use.contains(&"Auriculares".to_string()));
}

/// Un bus vivo sin nombre de tarjeta no debe retener la cadena vacia como si
/// fuera un dispositivo.
#[test]
fn un_bus_sin_nombre_no_cuenta_como_tarjeta() {
    let live = [(BusId::Programa, Routing::Device(String::new()))];
    assert!(devices_in_use(&live, "").is_empty());
}

/// La regla que da sentido a la consola: la pre-escucha no entra en el programa
/// ni pidiendolo. Si alguien lo pide, sale por la tarjeta del programa pero
/// aparte — sin master y sin sumar al vumetro.
#[test]
fn el_cue_nunca_suma_en_programa() {
    assert!(!BusId::Cue.can_sum_into_program());
    assert_eq!(
        sanitize(BusId::Cue, Routing::Program),
        Routing::ProgramDevice
    );
}

/// Por defecto la pre-escucha comparte tarjeta con el programa (out_pre viene
/// vacio), y aun asi sigue siendo un bus aparte. Ese es el caso de la mayoria de
/// equipos, y el que antes hacia que se colara en el master.
#[test]
fn el_cue_comparte_tarjeta_por_defecto_pero_no_bus() {
    assert_eq!(BusId::Cue.default_routing(), Routing::ProgramDevice);
    assert_eq!(
        device_of(&BusId::Cue.default_routing(), "Altavoces"),
        Some("Altavoces".to_string())
    );
}

/// El programa es el final del camino: no puede sumarse a si mismo.
#[test]
fn el_programa_no_puede_sumarse_a_si_mismo() {
    assert!(!BusId::Programa.can_sum_into_program());
    assert_eq!(
        sanitize(BusId::Programa, Routing::Program),
        Routing::Device(String::new())
    );
    assert_eq!(
        sanitize(BusId::Programa, Routing::ProgramDevice),
        Routing::Device(String::new())
    );
}

/// Los efectos y el panel van al programa por defecto: es lo que sale al aire, y
/// por eso los dos cuentan en el vumetro principal.
#[test]
fn los_efectos_y_el_panel_van_al_programa() {
    for bus in [BusId::Efectos, BusId::Panel] {
        assert!(bus.can_sum_into_program());
        assert_eq!(bus.default_routing(), Routing::Program);
        // Sumado en el programa no tiene tarjeta propia: la del programa es.
        assert_eq!(device_of(&Routing::Program, "Altavoces"), None);
    }
}

/// Sacar un bus a otra tarjeta lo saca del programa: deja de pasar por el master
/// y de contar en su vumetro. Es la regla que el autor pidio que se "rompiera" al
/// mover un apartado a otra salida.
#[test]
fn sacar_un_bus_a_otra_tarjeta_lo_saca_del_programa() {
    let routing = sanitize(BusId::Efectos, Routing::Device("Auriculares".into()));
    assert_eq!(routing, Routing::Device("Auriculares".into()));
    assert_eq!(
        device_of(&routing, "Altavoces"),
        Some("Auriculares".to_string())
    );
}
