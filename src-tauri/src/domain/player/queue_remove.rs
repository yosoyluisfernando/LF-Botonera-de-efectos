//! Eliminación pura de varias posiciones de la cola persistida.
//! Valida todo antes de mutar y elimina de mayor a menor para no desplazar
//! posiciones todavía pendientes.

pub fn remove_positions<T>(items: &mut Vec<T>, positions: &[u32]) -> Result<(), String> {
    if positions.is_empty() {
        return Err("no_player_tracks_selected".into());
    }
    let mut positions: Vec<usize> = positions.iter().map(|&index| index as usize).collect();
    positions.sort_unstable();
    positions.dedup();
    if positions.last().is_some_and(|&index| index >= items.len()) {
        return Err("button_not_found".into());
    }
    for index in positions.into_iter().rev() {
        items.remove(index);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::remove_positions;

    #[test]
    fn removes_multiple_positions_without_index_drift() {
        let mut items = vec!["a", "b", "c", "d", "e"];
        remove_positions(&mut items, &[1, 3]).unwrap();
        assert_eq!(items, ["a", "c", "e"]);
    }

    #[test]
    fn ignores_duplicate_positions() {
        let mut items = vec!["a", "b", "c"];
        remove_positions(&mut items, &[1, 1]).unwrap();
        assert_eq!(items, ["a", "c"]);
    }

    #[test]
    fn invalid_selection_does_not_partially_mutate() {
        let mut items = vec!["a", "b", "c"];
        assert!(remove_positions(&mut items, &[0, 8]).is_err());
        assert_eq!(items, ["a", "b", "c"]);
    }
}
