/**
 * Archivo: gridDnd.js
 * Proposito: arrastre de botones, entre celdas/pestanas, y archivos externos.
 * La UI solo marca origen/destino; Rust decide y persiste el movimiento real.
 */

import { invoke, listen } from './api.js';
import { drawGrid } from './grid.js';
import { appAlert } from './appDialog.js';
import { t } from './i18n.js';

let _onRefresh = null;
let _srcIndex = null;
let _srcPaletaId = '';
let _wired = false;

/** Inicializa ambos sistemas de arrastre. Llamar una sola vez. */
export function initGridDnd(onRefresh) {
    _onRefresh = onRefresh;
    if (_wired) return;
    _wired = true;
    _wireReorder();
    _wireFileDrop();
}

function _wireReorder() {
    document.addEventListener('mousedown', e => {
        if (!e.altKey || e.button !== 0) return;
        const cell = e.target.closest('.grid-item[data-id]');
        if (!cell) return;
        _srcIndex = parseInt(cell.dataset.index);
        _srcPaletaId = _activePaletaId();
        cell.classList.add('drag-source');
        _activeTab()?.classList.add('tab-drag-source');
        e.preventDefault();
    });

    document.addEventListener('mousemove', e => {
        if (_srcIndex === null) return;
        _markDragTarget(e.target);
    });

    document.addEventListener('mouseup', async e => {
        if (_srcIndex === null) return;
        const from = _srcIndex;
        const fromPaletaId = _srcPaletaId;
        _srcIndex = null;
        _srcPaletaId = '';
        _clearDragMarks();

        const tab = e.target.closest('#tabs-list .tab[data-paleta-id]');
        if (tab?.dataset.paletaId && tab.dataset.paletaId !== fromPaletaId) {
            await _moveToTab(fromPaletaId, from, tab.dataset.paletaId);
            return;
        }
        await _moveWithinGrid(e.target.closest('.grid-item'), from);
    });
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
    } catch (err) {
        const key = `errors.${err}`;
        const msg = t(key);
        await appAlert(msg === key ? String(err) : msg);
    }
}

function _wireFileDrop() {
    listen('tauri://drag-over', e => _markFileTarget(_cellAt(e.payload.position)));
    listen('tauri://drag-leave', () => _markFileTarget(null));

    listen('tauri://drag-drop', async e => {
        const cell = _cellAt(e.payload.position);
        _markFileTarget(null);
        const paths = e.payload.paths;
        if (!cell || !paths?.length) return;
        try {
            const s = await invoke('assign_file_to_button', {
                index: parseInt(cell.dataset.index),
                path: paths[0],
            });
            drawGrid(s, _onRefresh);
        } catch (err) {
            console.error('Error al asignar archivo soltado:', err);
        }
    });
}

function _cellAt(pos) {
    if (!pos) return null;
    const scale = window.devicePixelRatio || 1;
    return document.elementFromPoint(pos.x / scale, pos.y / scale)
        ?.closest('.grid-item') ?? null;
}

function _markDragTarget(target) {
    document.querySelectorAll('.grid-item.drag-over, .tab.tab-drag-over')
        .forEach(el => el.classList.remove('drag-over', 'tab-drag-over'));
    target.closest('.grid-item')?.classList.add('drag-over');
    const tab = target.closest('#tabs-list .tab[data-paleta-id]');
    if (tab?.dataset.paletaId !== _srcPaletaId) tab?.classList.add('tab-drag-over');
}

function _markFileTarget(el) {
    document.querySelectorAll('.grid-item.drag-over')
        .forEach(c => c.classList.remove('drag-over'));
    el?.classList.add('drag-over');
}

function _clearDragMarks() {
    document.querySelectorAll('.grid-item.drag-over, .grid-item.drag-source')
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
