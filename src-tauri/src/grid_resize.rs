/// Modulo: grid_resize.rs
/// Proposito: redimensionar una pestana conservando posiciones visuales.
use crate::types::{ButtonData, PaletaData};

#[derive(Debug)]
pub struct ResizeResult {
    pub changed: bool,
    pub mappings: Vec<(String, String)>,
}

/// Cambia filas/columnas sin compactar botones. Al crecer, las nuevas celdas
/// quedan vacias; al reducir, se rechaza si algun boton quedaria fuera.
pub fn resize_paleta(
    paleta: &mut PaletaData,
    next_rows: u32,
    next_cols: u32,
) -> Result<ResizeResult, String> {
    if paleta.rows == next_rows && paleta.cols == next_cols {
        return Ok(ResizeResult {
            changed: false,
            mappings: Vec::new(),
        });
    }
    validate_resize(paleta, next_rows, next_cols)?;
    let old_cols = paleta.cols;
    let paleta_id = paleta.id.clone();
    let mut mappings = Vec::new();

    for button in paleta.botones.iter_mut() {
        let next_index = remap_index(button.index, old_cols, next_cols);
        move_button(button, &paleta_id, next_index, &mut mappings);
    }
    paleta.rows = next_rows;
    paleta.cols = next_cols;
    Ok(ResizeResult {
        changed: true,
        mappings,
    })
}

pub fn validate_resize(paleta: &PaletaData, next_rows: u32, next_cols: u32) -> Result<(), String> {
    if next_rows == 0 || next_cols == 0 {
        return Err("invalid_grid_size".to_string());
    }
    for button in &paleta.botones {
        let row = (button.index - 1) / paleta.cols;
        let col = (button.index - 1) % paleta.cols;
        if row >= next_rows || col >= next_cols {
            return Err("grid_resize_would_drop_buttons".to_string());
        }
    }
    Ok(())
}

fn remap_index(index: u32, old_cols: u32, next_cols: u32) -> u32 {
    let row = (index - 1) / old_cols;
    let col = (index - 1) % old_cols;
    row * next_cols + col + 1
}

fn move_button(
    button: &mut ButtonData,
    paleta_id: &str,
    next_index: u32,
    mappings: &mut Vec<(String, String)>,
) {
    let next_id = button_id(paleta_id, next_index);
    if button.id != next_id {
        mappings.push((button.id.clone(), next_id.clone()));
    }
    button.index = next_index;
    button.id = next_id;
}

fn button_id(paleta_id: &str, index: u32) -> String {
    format!("{}_btn_{}", paleta_id, index)
}

#[cfg(test)]
mod tests {
    use super::resize_paleta;
    use crate::button_defaults::new_button;
    use crate::types::PaletaData;

    #[test]
    fn adding_column_preserves_visual_positions() {
        let mut paleta = fixture(5, 5, &[1, 5, 6, 11, 21, 25]);
        resize_paleta(&mut paleta, 5, 6).unwrap();
        assert_eq!(indexes(&paleta), vec![1, 5, 7, 13, 25, 29]);
    }

    #[test]
    fn adding_row_keeps_existing_indexes() {
        let mut paleta = fixture(5, 5, &[1, 13, 25]);
        resize_paleta(&mut paleta, 6, 5).unwrap();
        assert_eq!(indexes(&paleta), vec![1, 13, 25]);
    }

    #[test]
    fn reducing_columns_rejects_buttons_outside_new_grid() {
        let mut paleta = fixture(5, 6, &[1, 6, 7]);
        let err = resize_paleta(&mut paleta, 5, 5).unwrap_err();
        assert_eq!(err, "grid_resize_would_drop_buttons");
        assert_eq!(indexes(&paleta), vec![1, 6, 7]);
    }

    #[test]
    fn reducing_rows_rejects_buttons_outside_new_grid() {
        let mut paleta = fixture(5, 5, &[1, 21]);
        let err = resize_paleta(&mut paleta, 4, 5).unwrap_err();
        assert_eq!(err, "grid_resize_would_drop_buttons");
        assert_eq!(indexes(&paleta), vec![1, 21]);
    }

    fn fixture(rows: u32, cols: u32, indexes: &[u32]) -> PaletaData {
        PaletaData {
            id: "paleta_1".to_string(),
            nombre: "Test".to_string(),
            rows,
            cols,
            audio_out: String::new(),
            shortcut: String::new(),
            tab_bg: String::new(),
            tab_text: String::new(),
            botones: indexes
                .iter()
                .map(|i| new_button("paleta_1", *i, &format!("B{i}"), "#111", "#fff"))
                .collect(),
        }
    }

    fn indexes(paleta: &PaletaData) -> Vec<u32> {
        let mut out: Vec<u32> = paleta.botones.iter().map(|b| b.index).collect();
        out.sort_unstable();
        out
    }
}
