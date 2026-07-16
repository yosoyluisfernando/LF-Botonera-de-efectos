/// Modulo: engine/audio/last_pressed.rs
/// Proposito: recordar el ultimo boton disparado. Lo usa el reloj de la barra
/// inferior para saber de quien enseña el tiempo restante.
/// Informacion del ultimo boton presionado.
#[derive(Clone)]
pub struct LastPressedInfo {
    pub id: String,
}
