/**
 * Archivo: buttonSelection.js
 * Propósito: seleccionar varios botones con **Ctrl + clic** para pintarlos de un
 * color de una vez. Funciona en la rejilla y en el panel fijo.
 *
 * Sustituye a la idea de una "política de colores" configurable: se descartó por
 * complicada. Aquí no hay que configurar nada por adelantado — seleccionas,
 * pintas, y ves el resultado.
 */

import { invoke } from '../bridge/api.js';
import { pickColor } from './colorPalette.js';
import { alertIpcError } from './ipcError.js';

// Un botón por índice, y el grupo del que salieron: no tiene sentido mezclar la
// rejilla con el panel fijo, porque el índice significa cosas distintas.
let _selected = new Set();
let _group = null;
let _wired = false;

export function initButtonSelection() {
    if (_wired) return;
    _wired = true;
    // Fase de captura: hay que ganarle al manejador que dispara el sonido.
    // Ctrl+clic NO debe sonar: seleccionar cinco efectos y oírlos todos a la vez
    // sería un desastre en directo.
    document.addEventListener('click', e => {
        if (!e.ctrlKey) return _maybeClear(e);
        const cell = _cellOf(e.target);
        if (!cell) return;
        e.preventDefault();
        e.stopPropagation();
        _toggle(cell);
    }, true);

    // Escape suelta la selección: sin esto quedarían botones marcados sin saber
    // cómo deshacerlo.
    document.addEventListener('keydown', e => {
        if (e.key === 'Escape' && _selected.size) clearSelection();
    });
}

export function hasSelection() {
    return _selected.size > 0;
}

export function selectionGroup() {
    return _group;
}

export function clearSelection() {
    _selected.clear();
    _group = null;
    document.querySelectorAll('.color-selected')
        .forEach(el => el.classList.remove('color-selected'));
}

/** Abre el selector y pinta todos los elegidos. Rust calcula el color del texto. */
export async function paintSelection(onRefresh) {
    const indexes = [..._selected];
    const group = _group;
    if (!indexes.length) return;
    // Reutiliza el mismo diálogo de 24 colores del editor, en modo "devuélveme
    // el color": aquí no hay formulario donde escribirlo.
    const color = await pickColor('button');
    if (!color) return;
    try {
        await invoke('set_buttons_color', { indexes, colorBg: color, group });
    } catch (err) {
        await alertIpcError(err);
    }
    clearSelection();
    onRefresh?.();
}

/** Solo botones CON contenido: una celda vacía no tiene color que cambiar. */
function _cellOf(node) {
    const cell = node?.closest?.('.grid-item[data-id], .fixed-panel-item[data-id]');
    return cell?.dataset.id ? cell : null;
}

function _toggle(cell) {
    const group = cell.classList.contains('fixed-panel-item') ? 'fixed' : 'grid';
    // Cambiar de grupo empieza una selección nueva: los índices de la rejilla y
    // los del panel no son comparables.
    if (_group && _group !== group) clearSelection();
    _group = group;

    const index = Number(cell.dataset.index);
    if (_selected.has(index)) {
        _selected.delete(index);
        cell.classList.remove('color-selected');
        if (!_selected.size) _group = null;
    } else {
        _selected.add(index);
        cell.classList.add('color-selected');
    }
}

/** Un clic normal fuera del menú suelta la selección. */
function _maybeClear(e) {
    if (!_selected.size || e.button !== 0) return;
    if (e.target.closest('#context-menu')) return;
    clearSelection();
}
