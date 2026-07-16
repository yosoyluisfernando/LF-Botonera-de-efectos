//! Modulo: domain/palette.rs
//! Proposito: la lista de colores que se ofrece al usuario. Es un dato, no
//! logica: la adaptacion a cada tema y el contraste viven en `colors.rs`.

/// Paleta que se ofrece al usuario. **Varia solo en MATIZ, y es a proposito.**
///
/// `adapt_color` recorta para garantizar contraste en los dos temas (regla 8):
/// en oscuro la luminosidad no pasa de 0.30, y en claro la saturacion no baja de
/// 0.90. Eso deja el matiz como **lo unico que sobrevive** a la adaptacion: dos
/// colores con el mismo matiz se ven IGUALES por mucho que su base difiera.
///
/// La paleta anterior eran 16 matices de Material en dos intensidades (600 y
/// 800), asi que aparentaba 32 colores pero el recorte igualaba cada pareja: se
/// veian 16, con 6 azules y 6 rojos y un solo verde. Medido: 26 parejas por
/// debajo de 12° de matiz, el punto donde el ojo deja de separarlas.
///
/// Estos 24 se reparten por el circulo de color con **separacion perceptual**, no
/// a intervalos iguales: el ojo distingue mal entre verdes y entre azules (pasos
/// de hasta 25°) y muy bien entre naranjas y amarillos (pasos de 12°). Todos
/// nacen con L=0.50 y S=0.72, el centro de lo que el recorte respeta, para que
/// ni el tema claro ni el oscuro los deformen.
pub const SAFE_COLORS: [&str; 24] = [
    // rojo → naranja → amarillo (el ojo separa bien: pasos cortos)
    "#DB2424", "#DB4F24", "#DB7924", "#DB9E24", "#DBC324", "#CFDB24",
    // verdes (separa mal: pasos largos)
    "#9EDB24", "#67DB24", "#24DB24", "#24DB70",
    // turquesa → cian
    "#24DBAD", "#24DBDB", "#24B4DB",
    // azules (separa mal: pasos largos)
    "#248FDB", "#2461DB", "#243CDB", "#3324DB",
    // violetas → magentas → rosas
    "#5B24DB", "#8C24DB", "#B724DB", "#DB24D5", "#DB24A4", "#DB2479", "#DB2452",
];
