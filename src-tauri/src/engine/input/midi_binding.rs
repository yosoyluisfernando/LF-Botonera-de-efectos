use crate::model::MidiBinding;

pub fn same(a: &MidiBinding, b: &MidiBinding) -> bool {
    !a.is_empty()
        && !b.is_empty()
        && a.device_id == b.device_id
        && a.message == b.message
        && a.channel == b.channel
        && a.number == b.number
}

pub fn name(name: &str, label: &str) -> String {
    if name.trim().is_empty() {
        label.into()
    } else {
        name.into()
    }
}

pub fn conflict(code: &str, binding: &MidiBinding, target: &str) -> String {
    let key = format!(
        "{} ch {} {} {}",
        binding.device_name,
        binding.channel.saturating_add(1),
        binding.message,
        binding.number
    );
    format!("{code}|{}|{}", key.trim(), target.replace('|', " "))
}
