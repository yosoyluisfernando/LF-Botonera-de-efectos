use super::{MidiPortInfo, MidiRuntimeMsg, Sender};
use crate::engine::input::midi_message::parse;
use windows_sys::Win32::Media::Audio::{
    midiInClose, midiInOpen, midiInReset, midiInStart, midiInStop, CALLBACK_FUNCTION, HMIDIIN,
};
use windows_sys::Win32::Media::MM_MIM_DATA;

pub struct MidiConnection {
    handle: HMIDIIN,
    context: *mut CallbackContext,
}

struct CallbackContext {
    id: String,
    name: String,
    tx: Sender<MidiRuntimeMsg>,
}

pub fn connect_port(port: &MidiPortInfo, tx: Sender<MidiRuntimeMsg>) -> Option<MidiConnection> {
    let context = Box::new(CallbackContext {
        id: port.id.clone(),
        name: port.name.clone(),
        tx,
    });
    let context = Box::into_raw(context);
    let mut handle = std::ptr::null_mut();
    let result = unsafe {
        midiInOpen(
            &mut handle,
            port.index,
            midi_callback as *const () as usize,
            context as usize,
            CALLBACK_FUNCTION,
        )
    };
    if result != 0 {
        unsafe { drop(Box::from_raw(context)) };
        return None;
    }
    if unsafe { midiInStart(handle) } != 0 {
        unsafe {
            midiInClose(handle);
            drop(Box::from_raw(context));
        }
        return None;
    }
    Some(MidiConnection { handle, context })
}

impl Drop for MidiConnection {
    fn drop(&mut self) {
        unsafe {
            let _ = midiInStop(self.handle);
            let _ = midiInReset(self.handle);
            let _ = midiInClose(self.handle);
            drop(Box::from_raw(self.context));
        }
    }
}

unsafe extern "system" fn midi_callback(
    _handle: HMIDIIN,
    message: u32,
    instance: usize,
    param1: usize,
    _param2: usize,
) {
    if message != MM_MIM_DATA || instance == 0 {
        return;
    }
    let context = &*(instance as *const CallbackContext);
    let packet = param1 as u32;
    let data = [
        (packet & 0xff) as u8,
        ((packet >> 8) & 0xff) as u8,
        ((packet >> 16) & 0xff) as u8,
    ];
    if let Some(event) = parse(&context.id, &context.name, &data) {
        let _ = context.tx.send(MidiRuntimeMsg::Incoming(event));
    }
}
