/// Modulo: engine/audio/attach.rs
/// Proposito: enganchar un boton a un bus de la consola. Construye el
/// ButtonSource con sus fades y devuelve el ButtonState con el que se controla.
///
/// Vive en el motor de efectos y no en la consola a proposito: un bus solo sabe
/// sumar fuentes. Que un boton tenga fades, trim, estado y grupo es asunto de
/// quien sabe de botones, que es este motor.
use crate::engine::audio::button::{ButtonSource, ButtonState, PlaybackGroup};
use crate::engine::console::Bus;
use crate::engine::dsp::fade::FadeRamp;
use rodio::Source;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Arc;
use std::time::Instant;

pub struct AttachArgs {
    pub volume: f32,
    pub duration: f64,
    pub loop_mode: bool,
    pub file_gain: f32,
    pub fade_in_s: f64,
    pub fade_out_stop_s: f64,
    pub fade_out_end_s: f64,
    pub position_offset_s: f64,
    pub group: PlaybackGroup,
}

/// Mete la fuente en el bus y devuelve el estado para controlarla desde fuera.
pub fn attach_button(
    bus: &Bus,
    source: Box<dyn Source<Item = f32> + Send + 'static>,
    args: AttachArgs,
) -> ButtonState {
    let done_flag = Arc::new(AtomicBool::new(false));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let vol_atomic = Arc::new(AtomicU32::new(args.volume.to_bits()));
    let sr = source.sample_rate();
    let ch = source.channels();
    // En loop el fundido de salida se mide sobre la vuelta entera; sin loop,
    // sobre lo que queda desde donde arranca (un salto no reinicia el fundido).
    let fade_duration = if args.loop_mode {
        args.duration
    } else {
        (args.duration - args.position_offset_s).max(0.0)
    };
    let total_samples = if fade_duration > 0.0 {
        (fade_duration * sr as f64 * ch as f64).round() as usize
    } else {
        0
    };
    let (fade, fade_out_flag) = FadeRamp::new(
        args.fade_in_s,
        args.fade_out_stop_s,
        args.fade_out_end_s,
        sr,
        ch,
        total_samples,
        args.loop_mode,
    );
    bus.add(ButtonSource {
        inner: source,
        stop_flag: Arc::clone(&stop_flag),
        done_flag: Arc::clone(&done_flag),
        file_gain: args.file_gain,
        volume: Arc::clone(&vol_atomic),
        fade,
    });
    ButtonState {
        group: args.group,
        done_flag,
        stop_flag,
        fade_out_flag,
        volume: vol_atomic,
        start_time: Instant::now(),
        position_offset_s: args.position_offset_s,
        duration: args.duration,
        loop_mode: args.loop_mode,
    }
}
