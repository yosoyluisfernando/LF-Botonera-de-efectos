//! Adaptadas de Danilo Wenner (7c1f1ae). Pruebas del reparto de carriles de salida por pestaña.
use super::{is_main_output, paleta_of_button, plan, TabRoute};
use crate::domain::console::{BusId, Routing, MAX_SALIDAS};
use std::collections::HashMap;

fn tab<'a>(id: &'a str, out: &'a str) -> TabRoute<'a> {
    TabRoute {
        paleta_id: id,
        nombre: id,
        audio_out: out,
    }
}

fn devices() -> Vec<String> {
    ["default", "Altavoces", "Auriculares", "Interfaz USB"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[test]
fn sin_salida_propia_va_por_efectos() {
    let p = plan(
        &[tab("a", ""), tab("b", "Altavoces")],
        "Altavoces",
        &devices(),
        &HashMap::new(),
    );
    assert!(p.paleta_bus.is_empty());
    assert!(p.issues.is_empty());
    assert!(p
        .routings
        .iter()
        .all(|(_, r)| *r == Routing::Device(String::new())));
}

#[test]
fn principal_vacia_equivale_a_default() {
    assert!(is_main_output("default", ""));
    assert!(is_main_output("", "Altavoces"));
    assert!(!is_main_output("Auriculares", "Altavoces"));
}

#[test]
fn la_misma_tarjeta_comparte_carril() {
    let tabs = [
        tab("a", "Auriculares"),
        tab("b", "Auriculares"),
        tab("c", "Interfaz USB"),
    ];
    let p = plan(&tabs, "Altavoces", &devices(), &HashMap::new());
    assert_eq!(p.paleta_bus["a"], p.paleta_bus["b"]);
    assert_ne!(p.paleta_bus["a"], p.paleta_bus["c"]);
    assert_eq!(p.routings.len(), MAX_SALIDAS as usize);
    assert!(p
        .routings
        .contains(&(p.paleta_bus["c"], Routing::Device("Interfaz USB".into()))));
}

/// Cambiar una pestaña no mueve a las demas: moverlas cortaria su audio.
#[test]
fn las_asignaciones_son_estables() {
    let first = plan(
        &[tab("a", "Auriculares"), tab("b", "Interfaz USB")],
        "Altavoces",
        &devices(),
        &HashMap::new(),
    );
    let second = plan(
        &[tab("b", "Interfaz USB")],
        "Altavoces",
        &devices(),
        &first.lanes,
    );
    assert_eq!(first.paleta_bus["b"], second.paleta_bus["b"]);
    let third = plan(
        &[tab("c", "Auriculares"), tab("b", "Interfaz USB")],
        "Altavoces",
        &devices(),
        &second.lanes,
    );
    assert_eq!(third.paleta_bus["b"], first.paleta_bus["b"]);
    assert_ne!(third.paleta_bus["c"], third.paleta_bus["b"]);
}

/// Una tarjeta desconectada informa sin enviar contenido a otro destino.
#[test]
fn tarjeta_desconectada_conserva_ruta_y_no_cae_a_la_principal() {
    let p = plan(
        &[tab("a", "Mesa vieja")],
        "Altavoces",
        &devices(),
        &HashMap::new(),
    );
    assert!(matches!(p.paleta_bus["a"], BusId::Salida(_)));
    assert_eq!(p.issues.len(), 1);
    assert_eq!(p.issues[0].reason, "missing");
    assert_eq!(p.issues[0].device, "Mesa vieja");
}

#[test]
fn sin_carriles_libres_se_informa() {
    let names: Vec<String> = (0..=MAX_SALIDAS).map(|i| format!("T{i}")).collect();
    let ids: Vec<String> = (0..=MAX_SALIDAS).map(|i| format!("p{i}")).collect();
    let tabs: Vec<TabRoute> = ids.iter().zip(&names).map(|(i, n)| tab(i, n)).collect();
    let p = plan(&tabs, "default", &names, &HashMap::new());
    assert_eq!(p.paleta_bus.len(), MAX_SALIDAS as usize + 1);
    assert_eq!(
        p.paleta_bus[ids.last().unwrap()],
        BusId::Salida(MAX_SALIDAS)
    );
    assert_eq!(p.issues.len(), 1);
    assert_eq!(p.issues[0].reason, "too_many");
}

#[test]
fn la_pestana_sale_del_id_del_boton() {
    assert_eq!(
        paleta_of_button("perfil_1_paleta_2_btn_14"),
        Some("perfil_1_paleta_2")
    );
    assert_eq!(paleta_of_button("__prelisten__"), None);
}

#[test]
fn los_carriles_libres_no_piden_salida() {
    assert!(!BusId::Salida(0).wants_output(&Routing::Device(String::new())));
    assert!(BusId::Salida(0).wants_output(&Routing::Device("X".into())));
    assert!(BusId::Programa.wants_output(&Routing::Device(String::new())));
    assert!(!BusId::Salida(1).can_sum_into_program());
}
