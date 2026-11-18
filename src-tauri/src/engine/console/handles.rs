//! Lectura de handles y control de faders sin pasar por el hilo guardián.
use super::{duck::DuckControl, Bus, BusId, ConsoleEngine};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
impl ConsoleEngine {
    pub fn generation(&self) -> u64 {
        self.state.lock().unwrap().generation
    }
    pub fn bus_revision(&self, bus: BusId) -> u64 {
        self.state.lock().unwrap().slots[&bus].revision
    }
    pub fn bus(&self, bus: BusId) -> Option<Bus> {
        self.state.lock().unwrap().live.get(&bus).cloned()
    }
    pub fn levels(&self, bus: BusId) -> (Arc<AtomicU32>, Arc<AtomicU32>) {
        let state = self.state.lock().unwrap();
        let slot = &state.slots[&bus];
        (Arc::clone(&slot.level_l), Arc::clone(&slot.level_r))
    }
    pub fn fader(&self, bus: BusId) -> f32 {
        f32::from_bits(self.volume_handle(bus).load(Ordering::Relaxed))
    }
    pub fn volume_handle(&self, bus: BusId) -> Arc<AtomicU32> {
        Arc::clone(&self.state.lock().unwrap().slots[&bus].volume)
    }
    pub fn duck_handle(&self, bus: BusId) -> Arc<DuckControl> {
        Arc::clone(&self.state.lock().unwrap().slots[&bus].duck)
    }
    pub fn set_fader(&self, bus: BusId, value: f32) {
        let value = if bus == BusId::Programa {
            if value.is_finite() { value.clamp(0.0,1.5) } else { 1.0 }
        } else { crate::model::console::unit_volume(value) };
        let state = self.state.lock().unwrap();
        state.slots[&bus].volume.store(value.to_bits(), Ordering::Relaxed);
        if bus == BusId::Programa { state.master_ramp.cancel(); }
    }
    /// Solo limita valores por encima de unidad; conserva cualquier volumen inferior.
    pub fn limit_master_to_unity(&self) -> f32 {
        let state = self.state.lock().unwrap();
        let volume = &state.slots[&BusId::Programa].volume;
        let target = f32::from_bits(volume.load(Ordering::Relaxed));
        if target > 1.0 {
            state.master_ramp.reduce(state.master_ramp.gain(target));
            volume.store(1.0f32.to_bits(), Ordering::Relaxed);
            1.0
        } else { target }
    }

}
