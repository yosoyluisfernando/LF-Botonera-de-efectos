/// Modulo: engine/audio/routing.rs
/// Proposito: a que bus de la consola manda el motor de efectos cada fuente.
/// Es todo lo que queda del booleano `to_pre`, que elegia entre dos salidas
/// fijas. Ahora la pre-escucha tiene bus propio y los botones van al de su
/// grupo, asi que cada uno puede tener su fader y su vumetro.
use crate::engine::audio::button::PlaybackGroup;
use crate::engine::console::BusId;

pub fn bus_for(to_pre: bool, group: PlaybackGroup) -> BusId {
    if to_pre {
        return BusId::Cue;
    }
    match group {
        PlaybackGroup::Main => BusId::Efectos,
        PlaybackGroup::Fixed => BusId::Panel,
        PlaybackGroup::Cue => BusId::Cue,
    }
}

#[cfg(test)]
mod tests {
    use super::bus_for;
    use crate::engine::audio::button::PlaybackGroup;
    use crate::engine::console::BusId;

    /// La pre-escucha va a su bus **sea cual sea el grupo**: que la dispare un
    /// boton del panel no la convierte en programa.
    #[test]
    fn la_pre_escucha_siempre_va_al_cue() {
        for group in [PlaybackGroup::Main, PlaybackGroup::Fixed] {
            assert_eq!(bus_for(true, group), BusId::Cue);
        }
    }

    #[test]
    fn cada_grupo_va_a_su_bus() {
        assert_eq!(bus_for(false, PlaybackGroup::Main), BusId::Efectos);
        assert_eq!(bus_for(false, PlaybackGroup::Fixed), BusId::Panel);
        assert_eq!(bus_for(false, PlaybackGroup::Cue), BusId::Cue);
    }

    /// Los dos buses de botones suman en programa: el vumetro principal cuenta
    /// la botonera Y el panel fijo, que es lo que el operador espera ver.
    #[test]
    fn los_dos_buses_de_botones_suenan_en_programa() {
        assert!(bus_for(false, PlaybackGroup::Main).can_sum_into_program());
        assert!(bus_for(false, PlaybackGroup::Fixed).can_sum_into_program());
        assert!(!bus_for(true, PlaybackGroup::Main).can_sum_into_program());
    }
}
