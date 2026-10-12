/**
 * Archivo: fileDrop.js
 * Proposito: recibir archivos nativos del explorador o de Biblioteca sobre la
 * rejilla, pestañas, panel fijo o reproductor. Es el UNICO oyente de los eventos
 * `tauri://drag-*`: decide el destino y delega. El movimiento interno de botones
 * vive en gridDnd.js.
 */

import { invoke, listen } from '../bridge/api.js';
import { drawGrid } from './grid.js';
import { appConfirm, appConfirm3 } from './appDialog.js';
import { alertIpcError } from './ipcError.js';
import { dropOnPlayer } from './playerDrop.js';
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
        const paths = e.payload.paths ?? [];
        if (!target || !paths.length) return;
        try {
            if (target.closest('#player-view')) await dropOnPlayer(target, paths);
            else if (target.closest('#tabs-list .tab[data-paleta-id]')) {
                await _dropFilesOnTab(target, paths);
            } else if (paths.length !== 1) {
                await appAlert(t('library.single_track_for_button'));
            } else if (target.closest('#fixed-panel[data-view="buttons"]')) {
                await dropFileOnFixed(target, paths[0]);
            }
            else await dropFileOnGrid(target, paths[0]);
        } catch (err) {
            console.error('Error al asignar archivo soltado:', err);
            await alertIpcError(err);
        }
    });
}

// Soltar sobre un boton fijo existente pregunta reemplazar/anadir/cancelar; en
// vacio anade al final. Rust asigna el indice final cuando no se envia.
export async function dropFileOnFixed(target, path) {
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

async function _dropFilesOnTab(target, paths) {
    const tab = target.closest('#tabs-list .tab[data-paleta-id]');
    await invoke('library_assign_to_paleta', {
        paths,
        paletaId: tab.dataset.paletaId,
    });
    await _onRefresh?.();
}

// Sobre una celda ocupada (con data-id) pregunta reemplazar/no; en vacio asigna.
export async function dropFileOnGrid(cell, path) {
    if (cell.dataset.id && !await appConfirm(t('app.button_has_content'),
        { ok: t('app.replace'), cancel: t('app.dont_add') }, { ok: 1, cancel: 2 })) return;
    const s = await invoke('assign_file_to_button', { index: parseInt(cell.dataset.index), path });
    drawGrid(s, _onRefresh);
}

function _dropTargetAt(pos) {
    if (!pos) return null;
    const scale = window.devicePixelRatio || 1;
    return document.elementFromPoint(pos.x / scale, pos.y / scale)
        ?.closest('.grid-item, .fixed-panel-item, #fixed-panel[data-view="buttons"], .player-row, #player-view, #tabs-list .tab[data-paleta-id]') ?? null;
}

function _markFileTarget(el) {
    document.querySelectorAll('.grid-item.drag-over, .fixed-panel-item.drag-over, #fixed-panel.drag-over, .player-row.drag-over, #player-view.drag-over, .tab.tab-drag-over')
        .forEach(c => c.classList.remove('drag-over', 'tab-drag-over'));
    if (el?.matches('#tabs-list .tab[data-paleta-id]')) el.classList.add('tab-drag-over');
    else el?.classList.add('drag-over');
}
