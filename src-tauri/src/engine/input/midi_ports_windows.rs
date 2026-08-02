use super::MidiPortInfo;
use windows_sys::Win32::Media::Audio::{midiInGetDevCapsW, midiInGetNumDevs, MIDIINCAPSW};

pub fn available_ports() -> Vec<MidiPortInfo> {
    let count = unsafe { midiInGetNumDevs() };
    (0..count)
        .filter_map(|index| unsafe {
            let mut caps = std::mem::zeroed::<MIDIINCAPSW>();
            let size = std::mem::size_of::<MIDIINCAPSW>() as u32;
            if midiInGetDevCapsW(index as usize, &mut caps, size) != 0 {
                return None;
            }
            let mid = std::ptr::addr_of!(caps.wMid).read_unaligned();
            let pid = std::ptr::addr_of!(caps.wPid).read_unaligned();
            let name_data = std::ptr::addr_of!(caps.szPname).read_unaligned();
            Some(MidiPortInfo {
                id: format!("winmm:{}:{}:{}", mid, pid, index),
                name: utf16_name(&name_data),
                index,
            })
        })
        .collect()
}

fn utf16_name(data: &[u16]) -> String {
    let len = data
        .iter()
        .position(|item| *item == 0)
        .unwrap_or(data.len());
    String::from_utf16_lossy(&data[..len])
}
