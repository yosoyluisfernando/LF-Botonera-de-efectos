//! Audio en memoria: cambiar otro destino conserva los mismos buses y muestras.
use super::{bus::BusOutput, graph_update, Bus, BusId, BusSlot, ConsoleState, Routing};
use rodio::{
    buffer::SamplesBuffer,
    dynamic_mixer::{self, DynamicMixerController},
    source::Zero,
    Source,
};
use std::{collections::HashMap, sync::Arc};
struct Rig {
    state: ConsoleState,
    targets: HashMap<BusId, Routing>,
    roots: HashMap<String, Arc<DynamicMixerController<f32>>>,
    main: Box<dyn Source<Item = f32> + Send>,
}
impl Rig {
    fn new() -> Self {
        let mut roots = HashMap::new();
        let (ctrl, main) = dynamic_mixer::mixer::<f32>(2, 48000);
        ctrl.add(Zero::<f32>::new(2, 48000));
        roots.insert("main".into(), ctrl);
        for name in ["USB", "USB2"] {
            let (ctrl, mixer) = dynamic_mixer::mixer::<f32>(2, 48000);
            std::mem::drop(mixer);
            roots.insert(name.into(), ctrl);
        }
        let targets = BusId::every()
            .map(|b| {
                (
                    b,
                    match b {
                        BusId::Programa => Routing::Device("main".into()),
                        BusId::Cue => Routing::Device("main".into()),
                        BusId::Salida(_) => Routing::Device(String::new()),
                        _ => Routing::Program,
                    },
                )
            })
            .collect();
        let state = ConsoleState {
            slots: BusId::every()
                .map(|b| (b, BusSlot::new(b.default_routing())))
                .collect(),
            ..Default::default()
        };
        let mut rig = Self {
            state,
            targets,
            roots,
            main: Box::new(main),
        };
        rig.update(&[]);
        rig
    }
    fn update(&mut self, force: &[BusId]) -> bool {
        let roots = &self.roots;
        graph_update::apply(&mut self.state, &self.targets, force, |id, state| {
            let slot = &state.slots[&id];
            let target = slot.applied.as_ref().unwrap();
            if !id.wants_output(target) {
                return;
            }
            let parent = match target {
                Routing::Program => state.live[&BusId::Programa].controller().clone(),
                Routing::Device(name) => roots[name].clone(),
                _ => unreachable!(),
            };
            let bus = Bus::open(
                BusOutput::Bus(&parent),
                slot.level_l.clone(),
                slot.level_r.clone(),
                slot.volume.clone(),
            )
            .unwrap();
            state.live.insert(id, bus);
        })
    }
}
#[test]
fn adding_a_tab_output_preserves_player_bus_and_every_main_sample() {
    let mut rig = Rig::new();
    let player = rig.state.live[&BusId::Reproductor].clone();
    let effects = rig.state.live[&BusId::Efectos].clone();
    player.add(SamplesBuffer::new(
        2,
        48000,
        (0..20000).map(|i| i as f32 / 20000.0).collect::<Vec<_>>(),
    ));
    effects.add(SamplesBuffer::new(2, 48000, vec![0.1; 20000]));
    let first = rig.main.next().unwrap();
    for _ in 1..100 {
        rig.main.next();
    }
    let revision = rig.state.slots[&BusId::Reproductor].revision;
    rig.targets
        .insert(BusId::Salida(0), Routing::Device("USB".into()));
    rig.update(&[]);
    assert!(player.same_as(&rig.state.live[&BusId::Reproductor]));
    assert!(effects.same_as(&rig.state.live[&BusId::Efectos]));
    assert_eq!(revision, rig.state.slots[&BusId::Reproductor].revision);
    for i in 100..300 {
        assert!((rig.main.next().unwrap() - (first + i as f32 / 20000.0)).abs() < 0.00001);
    }
}
#[test]
fn editing_one_output_keeps_the_other_output_and_program() {
    let mut rig = Rig::new();
    rig.targets
        .insert(BusId::Salida(0), Routing::Device("USB".into()));
    rig.targets
        .insert(BusId::Salida(1), Routing::Device("USB".into()));
    rig.update(&[]);
    let untouched = rig.state.live[&BusId::Salida(1)].clone();
    let program = rig.state.live[&BusId::Programa].clone();
    let changed = rig.state.live[&BusId::Salida(0)].clone();
    rig.targets
        .insert(BusId::Salida(0), Routing::Device("USB2".into()));
    rig.update(&[]);
    assert!(untouched.same_as(&rig.state.live[&BusId::Salida(1)]));
    assert!(program.same_as(&rig.state.live[&BusId::Programa]));
    assert!(!changed.same_as(&rig.state.live[&BusId::Salida(0)]));
}
#[test]
fn stable_poll_and_retry_of_one_lane_do_not_recreate_player() {
    let mut rig = Rig::new();
    let player = rig.state.live[&BusId::Reproductor].clone();
    let generation = rig.state.generation;
    assert!(!rig.update(&[]));
    assert_eq!(generation, rig.state.generation);
    rig.targets
        .insert(BusId::Salida(0), Routing::Device("USB".into()));
    rig.update(&[]);
    rig.update(&[BusId::Salida(0)]);
    assert!(player.same_as(&rig.state.live[&BusId::Reproductor]));
}
#[test]
fn moving_program_only_rebuilds_its_children() {
    let mut rig = Rig::new();
    rig.targets
        .insert(BusId::Reproductor, Routing::Device("USB".into()));
    rig.update(&[]);
    let independent_player = rig.state.live[&BusId::Reproductor].clone();
    let effects = rig.state.live[&BusId::Efectos].clone();
    rig.targets
        .insert(BusId::Programa, Routing::Device("USB2".into()));
    rig.update(&[]);
    assert!(independent_player.same_as(&rig.state.live[&BusId::Reproductor]));
    assert!(!effects.same_as(&rig.state.live[&BusId::Efectos]));
}

#[path = "reattach_tests.rs"]
mod reattach;
