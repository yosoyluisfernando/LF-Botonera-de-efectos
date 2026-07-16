/**
 * Archivo: gridDnd.js
 * Proposito: arrastre INTERNO con Alt: mover botones entre celdas, pestanas, el
 * panel fijo y la cola del reproductor. La UI solo marca origen/destino; Rust
 * decide y persiste el movimiento. Los archivos del explorador van en fileDrop.js.
 */

import { invoke } from '../bridge/api.js';
import { drawGrid } from './grid.js';
import { alertIpcError } from './ipcError.js';
import { dropButtonOnPlayer } from './playerDnd.js';

let _onRefresh = null;
let _srcIndex = null;
let _srcId = '';
let _srcPaletaId = '';
let _wired = false;
let _srcFixed = false;

/** Inicializa el arrastre interno. Llamar una sola vez. */
export function initGridDnd(onRefresh) {
    _onRefresh = onRefresh;
    if (_wired) return;
    _wired = true;
    _wireReorder();
}

function _wireReorder() {
    document.addEventListener('mousedown', e => {
        if (!e.altKey || e.button !== 0) return;
        const cell = e.target.closest('.grid-item[data-id], .fixed-panel-item[data-id]');
        if (!cell) return;
        _srcIndex = parseInt(cell.dataset.index);
        _srcId = cell.dataset.id ?? '';
        _srcFixed = cell.classList.contains('fixed-panel-item');
        _srcPaletaId = _activePaletaId();
        cell.classList.add('drag-source');
        if (!_srcFixed) _activeTab()?.classList.add('tab-drag-source');
        e.preventDefault();
    });

    document.addEventListener('mousemove', e => {
        if (_srcIndex === null) return;
        _markDragTarget(e.target);
    });

    document.addEventListener('mouseup', async e => {
        if (_srcIndex === null) return;
        const from = _srcIndex;
        const fromId = _srcId;
        const fromPaletaId = _srcPaletaId;
        const fromFixed = _srcFixed;
        _srcIndex = null;
        _srcId = '';
        _srcPaletaId = '';
        _srcFixed = false;
        _clearDragMarks();

        // Cualquier boton arrastrado al reproductor se copia a su cola.
        if (e.target.closest('#player-view')) {
            try { await dropButtonOnPlayer(e.target, fromId); } catch (err) { await alertIpcError(err); }
            return;
        }
        if (fromFixed) {
            const gridCell = e.target.closest('.grid-item[data-index]');
            if (gridCell) { await _moveFixedToGrid(from, gridCell); return; }
            const target = e.target.closest('.fixed-panel-item[data-id]');
            if (target) await _moveWithinFixed(from, parseInt(target.dataset.index));
            return;
        }
        if (e.target.closest('#fixed-panel')) {
            await _moveGridToFixed(fromPaletaId, from, e.target);
            return;
        }
        const tab = e.target.closest('#tabs-list .tab[data-paleta-id]');
        if (tab?.dataset.paletaId && tab.dataset.paletaId !== fromPaletaId) {
            await _moveToTab(fromPaletaId, from, tab.dataset.paletaId);
            return;
        }
        await _moveWithinGrid(e.target.closest('.grid-item'), from);
    });
}

async function _moveWithinFixed(from, to) {
    if (from === to) return;
    try { await invoke('reorder_fixed_buttons', { fromIndex: from, toIndex: to }); _onRefresh?.(); }
    catch (err) { console.error('Error al reorganizar botones fijos:', err); }
}

async function _moveGridToFixed(fromPaletaId, fromIndex, dropTarget) {
    const item = dropTarget.closest('.fixed-panel-item[data-index]');
    const toFixedIndex = item ? parseInt(item.dataset.index) : null;
    try {
        await invoke('move_button_to_fixed', { fromPaletaId, fromIndex, toFixedIndex });
        _onRefresh?.();
    } catch (err) { await alertIpcError(err); }
}

async function _moveFixedToGrid(fromFixedIndex, gridCell) {
    try {
        await invoke('move_fixed_to_button', {
            fromFixedIndex,
            toPaletaId: _activePaletaId(),
            toIndex: parseInt(gridCell.dataset.index),
        });
        _onRefresh?.();
    } catch (err) { await alertIpcError(err); }
}

async function _moveWithinGrid(target, from) {
    const to = target ? parseInt(target.dataset.index) : null;
    if (to === null || to === from) return;
    try {
        const s = await invoke('reorder_buttons', { fromIndex: from, toIndex: to });
        drawGrid(s, _onRefresh);
    } catch (err) {
        console.error('Error al reorganizar botones:', err);
    }
}

async function _moveToTab(fromPaletaId, fromIndex, toPaletaId) {
    try {
        await invoke('move_button_to_paleta', { fromPaletaId, fromIndex, toPaletaId });
        _onRefresh?.();
    } catch (err) { await alertIpcError(err); }
}

function _markDragTarget(target) {
    document.querySelectorAll('.grid-item.drag-over, .fixed-panel-item.drag-over, .tab.tab-drag-over')
        .forEach(el => el.classList.remove('drag-over', 'tab-drag-over'));
    target.closest('.grid-item, .fixed-panel-item')?.classList.add('drag-over');
    const tab = target.closest('#tabs-list .tab[data-paleta-id]');
    if (!_srcFixed && tab?.dataset.paletaId !== _srcPaletaId) tab?.classList.add('tab-drag-over');
}

function _clearDragMarks() {
    document.querySelectorAll('.grid-item.drag-over, .grid-item.drag-source, .fixed-panel-item.drag-over, .fixed-panel-item.drag-source')
        .forEach(el => el.classList.remove('drag-over', 'drag-source'));
    document.querySelectorAll('.tab.tab-drag-over, .tab.tab-drag-source')
        .forEach(el => el.classList.remove('tab-drag-over', 'tab-drag-source'));
}

function _activePaletaId() {
    return _activeTab()?.dataset.paletaId ?? '';
}

function _activeTab() {
    return document.querySelector('#tabs-list .tab.active[data-paleta-id]');
}
