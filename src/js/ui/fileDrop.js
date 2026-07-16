/**
 * Archivo: fileDrop.js
 * Proposito: soltar archivos del explorador (Windows/Linux) sobre la rejilla, el
 * panel fijo o la cola del reproductor. Es el UNICO oyente de los eventos
 * `tauri://drag-*`: decide el destino y delega. El arrastre interno entre botones
 * vive en gridDnd.js.
 */

import { invoke, listen } from '../bridge/api.js';
import { drawGrid } from './grid.js';
import { appConfirm, appConfirm3 } from './appDialog.js';
import { alertIpcError } from './ipcError.js';
import { dropFileOnPlayer } from './playerDnd.js';
import { t } from '../util/i18n.js';

let _onRefresh = null;
let _wired = false;

export function initFileDrop(onRefresh) {
    _onRefresh = onRefresh;
    if (_wired) return;
    _wired = true;
    listen('tauri://drag-over', e => _markFileTarget(_dropTargetAt(e.payload.position)));
    listen('tauri://drag-leave', () => _markFileTarget(null));
    listen('tauri://drag-drop', async e => {
        const target = _dropTargetAt(e.payload.position);
        _markFileTarget(null);
        const path = e.payload.paths?.[0]; // Un solo archivo por soltado (regla de producto).
        if (!target || !path) return;
        try {
            if (target.closest('#player-view')) await dropFileOnPlayer(target, path);
            else if (target.closest('#fixed-panel')) await _dropFileOnFixed(target, path);
            else await _dropFileOnGrid(target, path);
        } catch (err) {
            console.error('Error al asignar archivo soltado:', err);
            await alertIpcError(err);
        }
    });
}

// Soltar sobre un boton fijo existente pregunta reemplazar/anadir/cancelar; en
// vacio anade al final. Rust asigna el indice final cuando no se envia.
async function _dropFileOnFixed(target, path) {
    const button = target.closest('.fixed-panel-item[data-index]');
    let args = { path };
    if (button) {
        const choice = await appConfirm3(t('app.button_has_content'),
            { ok: t('app.replace'), alt: t('app.append_end'), cancel: t('edit_modal.cancel') });
        if (choice === 'cancel') return;
        if (choice === 'yes') args = { index: parseInt(button.dataset.index), path };
    }
    await invoke('assign_file_to_fixed_button', args);
    _onRefresh?.();
}

// Sobre una celda ocupada (con data-id) pregunta reemplazar/no; en vacio asigna.
async function _dropFileOnGrid(cell, path) {
    if (cell.dataset.id && !await appConfirm(t('app.button_has_content'),
        { ok: t('app.replace'), cancel: t('app.dont_add') }, { ok: 1, cancel: 2 })) return;
    const s = await invoke('assign_file_to_button', { index: parseInt(cell.dataset.index), path });
    drawGrid(s, _onRefresh);
}

function _dropTargetAt(pos) {
    if (!pos) return null;
    const scale = window.devicePixelRatio || 1;
    return document.elementFromPoint(pos.x / scale, pos.y / scale)
        ?.closest('.grid-item, .fixed-panel-item, #fixed-panel, .player-row, #player-view') ?? null;
}

function _markFileTarget(el) {
    document.querySelectorAll('.grid-item.drag-over, .fixed-panel-item.drag-over, #fixed-panel.drag-over, .player-row.drag-over, #player-view.drag-over')
        .forEach(c => c.classList.remove('drag-over'));
    el?.classList.add('drag-over');
}
