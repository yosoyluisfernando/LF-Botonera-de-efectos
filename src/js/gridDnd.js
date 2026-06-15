/**
 * Archivo: gridDnd.js
 * Propósito: Dos tipos de arrastre sobre la cuadrícula:
 *  1. Alt+arrastrar con el ratón para intercambiar botones (reorder_buttons).
 *     Se usa mousedown/mousemove/mouseup porque el webview de Tauri captura
 *     el drag & drop nativo y el DnD de HTML5 nunca llega al DOM.
 *  2. Soltar archivos desde el explorador: Tauri v2 emite tauri://drag-over y
 *     tauri://drag-drop con la posición física del cursor.
 */

import { invoke, listen } from './api.js';
import { drawGrid } from './grid.js';

let _onRefresh = null;
let _srcIndex  = null;
let _wired     = false;

/** Inicializa ambos sistemas de arrastre. Llamar una sola vez. */
export function initGridDnd(onRefresh) {
    _onRefresh = onRefresh;
    if (_wired) return;
    _wired = true;
    _wireReorder();
    _wireFileDrop();
}

// ─── Alt+arrastrar: intercambiar botones ─────────────────────────────────────

function _wireReorder() {
    document.addEventListener('mousedown', e => {
        if (!e.altKey || e.button !== 0) return;
        const cell = e.target.closest('.grid-item[data-id]');
        if (cell) {
            _srcIndex = parseInt(cell.dataset.index);
            e.preventDefault(); // evita selección de texto durante el arrastre
        }
    });

    document.addEventListener('mousemove', e => {
        if (_srcIndex === null) return;
        _mark(e.target.closest('.grid-item'));
    });

    document.addEventListener('mouseup', async e => {
        if (_srcIndex === null) return;
        const from = _srcIndex;
        _srcIndex  = null;
        _mark(null);
        const target = e.target.closest('.grid-item');
        const to     = target ? parseInt(target.dataset.index) : null;
        if (to === null || to === from) return;
        try {
            const s = await invoke('reorder_buttons', { fromIndex: from, toIndex: to });
            drawGrid(s, _onRefresh);
        } catch (err) { console.error('Error al reorganizar botones:', err); }
    });
}

// ─── Archivos externos (eventos nativos de Tauri v2) ─────────────────────────

function _wireFileDrop() {
    listen('tauri://drag-over', e => _mark(_cellAt(e.payload.position)));
    listen('tauri://drag-leave', () => _mark(null));

    listen('tauri://drag-drop', async e => {
        const cell = _cellAt(e.payload.position);
        _mark(null);
        const paths = e.payload.paths;
        if (!cell || !paths?.length) return;
        try {
            const s = await invoke('assign_file_to_button', {
                index: parseInt(cell.dataset.index),
                path:  paths[0],
            });
            drawGrid(s, _onRefresh);
        } catch (err) { console.error('Error al asignar archivo soltado:', err); }
    });
}

/** Convierte la posición física del cursor (Tauri) en la celda bajo él. */
function _cellAt(pos) {
    if (!pos) return null;
    const scale = window.devicePixelRatio || 1;
    return document.elementFromPoint(pos.x / scale, pos.y / scale)
        ?.closest('.grid-item') ?? null;
}

/** Resalta la celda destino (o limpia el resaltado si el elemento es null). */
function _mark(el) {
    document.querySelectorAll('.grid-item.drag-over')
        .forEach(c => c.classList.remove('drag-over'));
    el?.classList.add('drag-over');
}
