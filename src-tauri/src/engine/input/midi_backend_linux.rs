use super::{MidiPortInfo, MidiRuntimeMsg, Sender};
use crate::engine::input::midi_message::parse;
use midir::{Ignore, MidiInput, MidiInputConnection};

const APP_CLIENT_NAME: &str = "LF Botonera de Efectos";
const CONNECTION_NAME: &str = "LF Botonera MIDI";

pub struct MidiConnection {
    _connection: MidiInputConnection<()>,
}

pub fn connect_port(port: &MidiPortInfo, tx: Sender<MidiRuntimeMsg>) -> Option<MidiConnection> {
    let mut input = MidiInput::new(APP_CLIENT_NAME).ok()?;
    input.ignore(Ignore::None);
    let native_port = input.find_port_by_id(&port.native_id)?;
    let id = port.id.clone();
    let name = port.name.clone();
    let connection = input
        .connect(
            &native_port,
            CONNECTION_NAME,
            move |_timestamp, data, _context| {
                if let Some(event) = parse(&id, &name, data) {
                    let _ = tx.send(MidiRuntimeMsg::Incoming(event));
                }
            },
            (),
        )
        .ok()?;
    Some(MidiConnection {
        _connection: connection,
    })
}
