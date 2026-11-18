//! ALSA: nombres PCM con tarjeta simbólica, independientes del índice de arranque.
pub fn identity(name: &str) -> String {
    identity_with(name, |number| std::fs::read_to_string(format!("/proc/asound/card{number}/id")).ok())
}
fn identity_with(name: &str, card_id: impl Fn(&str) -> Option<String>) -> String {
    let parts = name.split_once("CARD=").map(|(prefix, rest)| (format!("{prefix}CARD="),rest))
        .or_else(|| ["hw:","plughw:"].into_iter().find_map(|prefix| name.strip_prefix(prefix).map(|rest| (prefix.into(),rest))));
    if let Some((prefix,rest))=parts {
        let number=rest.split(',').next().unwrap_or(rest);
        if number.parse::<u32>().is_ok() {
            if let Some(id)=card_id(number).filter(|s| !s.trim().is_empty()) {
                return format!("alsa:{prefix}{}{}",id.trim(),&rest[number.len()..]);
            }
        }
    }
    format!("alsa:{name}")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn a_card_keeps_its_identity_when_alsa_indices_change() {
        let read=|_: &str| Some("USB\n".into());
        assert_eq!(identity_with("hw:2,0",read),identity_with("hw:5,0",read));
        assert_eq!(identity_with("front:CARD=2,DEV=0",read),"alsa:front:CARD=USB,DEV=0");
    }
    #[test] fn named_pcm_keeps_its_backend_reference() {
        assert_eq!(identity("plughw:CARD=USB,DEV=0"),"alsa:plughw:CARD=USB,DEV=0");
    }
}
