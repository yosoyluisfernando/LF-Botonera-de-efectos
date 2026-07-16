//! Modulo: engine/console/thread.rs
//! Proposito: el hilo guardian de la consola. Unico dueno de las tarjetas
//! abiertas (`EndpointRegistry`), porque `OutputStream` no es Send y tiene que
//! quedarse quieto en algun sitio. Solo atiende cambios de ruteo: reproducir NO
//! pasa por aqui — los motores anaden fuentes al bus desde sus propios hilos.
use super::bus::Bus;
use super::endpoint::EndpointRegistry;
use super::{BusId, BusSlot, ConsoleCommand, ConsoleState};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub fn run(rx: Receiver<ConsoleCommand>, state: Arc<Mutex<ConsoleState>>) {
    let mut endpoints = EndpointRegistry::default();
    for cmd in rx {
        match cmd {
            ConsoleCommand::SetBusDevice { bus, device_name } => {
                set_bus_device(&mut endpoints, &state, bus, device_name);
            }
            ConsoleCommand::CloseBus { bus } => {
                {
                    let mut guard = state.lock().unwrap();
                    guard.live.remove(&bus);
                    if let Some(slot) = guard.slots.get_mut(&bus) {
                        slot.device_name = String::new();
                    }
                }
                close_unused(&mut endpoints, &state);
            }
        }
    }
}

/// Enchufa el bus a la tarjeta pedida. Si ya estaba ahi y sigue vivo, no se toca:
/// reabrir cortaria el audio que este sonando por ese bus.
fn set_bus_device(
    endpoints: &mut EndpointRegistry,
    state: &Arc<Mutex<ConsoleState>>,
    bus: BusId,
    device_name: String,
) {
    {
        let state = state.lock().unwrap();
        let same = state
            .slots
            .get(&bus)
            .is_some_and(|slot| slot.device_name == device_name);
        if same && state.live.contains_key(&bus) {
            return;
        }
    }
    // El bus viejo se suelta ANTES de abrir el nuevo: si los dos apuntaran a la
    // misma tarjeta, el mixer viejo seguiria sonando en paralelo al nuevo.
    state.lock().unwrap().live.remove(&bus);

    let opened = endpoints.ensure(&device_name).and_then(|endpoint| {
        let mut state = state.lock().unwrap();
        let slot = state.slots.get(&bus)?;
        let fresh = Bus::open(
            endpoint.handle(),
            Arc::clone(&slot.level_l),
            Arc::clone(&slot.level_r),
            Arc::clone(&slot.volume),
        )?;
        state.live.insert(bus, fresh);
        state.slots.get_mut(&bus)?.device_name = device_name.clone();
        Some(())
    });
    // La tarjeta no existe o no dejo abrirse: el bus se queda sin salida y el
    // slot sin nombre, para que un reintento con el mismo nombre no crea que ya
    // esta hecho y se salte el trabajo.
    if opened.is_none() {
        if let Some(slot) = state.lock().unwrap().slots.get_mut(&bus) {
            slot.device_name = String::new();
        }
    }
    close_unused(endpoints, state);
}

/// Cierra las tarjetas que ya no usa ningun bus.
fn close_unused(endpoints: &mut EndpointRegistry, state: &Arc<Mutex<ConsoleState>>) {
    let in_use = {
        let guard = state.lock().unwrap();
        let live: Vec<BusId> = guard.live.keys().copied().collect();
        devices_in_use(&guard.slots, &live)
    };
    endpoints.retain_only(&in_use);
}

/// Que tarjetas siguen haciendo falta: solo las de los buses que existen AHORA.
/// Que el slot de un bus recuerde un nombre no basta — si su bus no esta vivo,
/// esa tarjeta no la escucha nadie y hay que soltarla.
fn devices_in_use(slots: &HashMap<BusId, BusSlot>, live: &[BusId]) -> Vec<String> {
    live.iter()
        .filter_map(|bus| slots.get(bus))
        .map(|slot| slot.device_name.clone())
        .filter(|name| !name.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{devices_in_use, BusId, BusSlot};
    use std::collections::HashMap;

    /// El caso que justifica el registro: dos buses en la misma tarjeta la
    /// mantienen abierta UNA vez, y ninguno de los dos la cierra por su cuenta.
    #[test]
    fn dos_buses_en_la_misma_tarjeta_la_mantienen_abierta() {
        let slots = slots_with(&[(BusId::Main, "Altavoces"), (BusId::Pre, "Altavoces")]);
        let in_use = devices_in_use(&slots, &[BusId::Main, BusId::Pre]);
        assert!(in_use.contains(&"Altavoces".to_string()));
    }

    /// Un bus cuyo slot recuerda una tarjeta pero que no esta vivo no la retiene:
    /// si no, cambiar de salida dejaria la vieja abierta para siempre.
    #[test]
    fn un_bus_no_vivo_no_retiene_su_tarjeta() {
        let slots = slots_with(&[(BusId::Main, "Altavoces"), (BusId::Pre, "Auriculares")]);
        let in_use = devices_in_use(&slots, &[BusId::Main]);
        assert_eq!(in_use, vec!["Altavoces".to_string()]);
    }

    /// Un bus vivo sin nombre de tarjeta no debe retener la cadena vacia como si
    /// fuera un dispositivo.
    #[test]
    fn un_bus_sin_nombre_no_cuenta_como_tarjeta() {
        let slots = slots_with(&[(BusId::Main, "")]);
        assert!(devices_in_use(&slots, &[BusId::Main]).is_empty());
    }

    fn slots_with(pairs: &[(BusId, &str)]) -> HashMap<BusId, BusSlot> {
        pairs
            .iter()
            .map(|(bus, name)| {
                let mut slot = BusSlot::new(1.0);
                slot.device_name = name.to_string();
                (*bus, slot)
            })
            .collect()
    }
}
