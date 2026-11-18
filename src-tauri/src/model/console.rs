/// Modulo: model/console.rs
/// Proposito: lo que la consola de audio recuerda entre sesiones.
use serde::{Deserialize, Serialize};

/// Ajustes globales de la consola y faders propios.
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
    #[serde(default = "unidad", deserialize_with = "deserialize_unit_volume")]
    pub efectos: f32,
    /// El bus de los botones fijos.
    #[serde(default = "unidad", deserialize_with = "deserialize_unit_volume")]
    pub panel: f32,
    /// La pre-escucha. Su fader no toca al programa: es la escucha del operador.
    #[serde(default = "unidad", deserialize_with = "deserialize_unit_volume")]
    pub cue: f32,
    #[serde(default = "default_dock_width")]
    pub dock_width: u16,
    #[serde(default, deserialize_with = "deserialize_output_faders")]
    pub output_faders: std::collections::BTreeMap<String, f32>,
}

fn default_dock_width() -> u16 {
    320
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
            dock_width: default_dock_width(),
            output_faders: Default::default(),
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

#[cfg(test)]
mod output_tests {
    #[test]
    fn destination_faders_round_trip_by_device_and_old_configs_default_empty() {
        let mut cfg: super::ConsoleConfig = serde_json::from_str("{}").unwrap();
        assert!(cfg.output_faders.is_empty());
        cfg.output_faders.insert("USB".into(), 0.4);
        let saved = serde_json::to_string(&cfg).unwrap();
        let restored: super::ConsoleConfig = serde_json::from_str(&saved).unwrap();
        assert_eq!(restored.output_faders["USB"], 0.4);
    }
}

/// Los faders de canal solo atenúan; 1.0 conserva la señal original.
pub fn unit_volume(value: f32) -> f32 { if value.is_finite() { value.clamp(0.0,1.0) } else { 1.0 } }
pub fn deserialize_unit_volume<'de, D: serde::Deserializer<'de>>(d: D) -> Result<f32,D::Error> {
    f32::deserialize(d).map(unit_volume)
}
fn deserialize_output_faders<'de, D: serde::Deserializer<'de>>(d: D) -> Result<std::collections::BTreeMap<String,f32>,D::Error> {
    let mut values=std::collections::BTreeMap::<String,f32>::deserialize(d)?;
    for value in values.values_mut() { *value=unit_volume(*value); }
    Ok(values)
}
