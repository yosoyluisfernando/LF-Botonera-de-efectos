/// Modulo: model/console.rs
/// Proposito: lo que la consola de audio recuerda entre sesiones.
use serde::{Deserialize, Serialize};

/// Los faders de la consola **que no tenian donde vivir**.
///
/// Aqui NO estan ni el master ni el volumen del reproductor, y no es un olvido:
/// el master es `AudioConfig.master_volume` (por perfil, y con su "recordar" y su
/// modo boost) y el del reproductor es `PlayerConfig.volume` (global). Los dos se
/// guardaban desde antes de que existiera la consola, y tenerlos tambien aqui
/// seria un segundo censo de lo mismo.
///
/// Es global, como el reproductor: la consola es del equipo, no del perfil.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ConsoleConfig {
    #[serde(default = "unidad")]
    pub efectos: f32,
    /// El bus de los botones fijos.
    #[serde(default = "unidad")]
    pub panel: f32,
    /// La pre-escucha. Su fader no toca al programa: es la escucha del operador.
    #[serde(default = "unidad")]
    pub cue: f32,
}

/// Un fader nuevo nace **abierto**: la consola aparece sin tocar nada de lo que
/// ya sonaba.
fn unidad() -> f32 {
    1.0
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            efectos: unidad(),
            panel: unidad(),
            cue: unidad(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConsoleConfig;

    /// Una configuracion vieja no trae nada de esto: todos los faders deben nacer
    /// abiertos para que abrir la consola no cambie como suena la aplicacion.
    #[test]
    fn una_config_sin_consola_nace_con_los_faders_abiertos() {
        let cfg: ConsoleConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(cfg, ConsoleConfig::default());
        assert_eq!(cfg.efectos, 1.0);
        assert_eq!(cfg.panel, 1.0);
        assert_eq!(cfg.cue, 1.0);
    }

    /// Y un fader guardado se respeta.
    #[test]
    fn un_fader_guardado_se_lee() {
        let cfg: ConsoleConfig = serde_json::from_str(r#"{"panel":0.5}"#).unwrap();
        assert_eq!(cfg.panel, 0.5);
        assert_eq!(cfg.efectos, 1.0);
    }
}
