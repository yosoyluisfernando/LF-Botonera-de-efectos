/**
 * Archivo: playerDnd.js
 * Proposito: arrastre de la cola del reproductor. Reordenar canciones con un
 * arrastre normal (sin Alt: en la lista el clic suelto no hace nada y el doble
 * clic activa la fila), recibir lo que se suelta desde el explorador o desde un
 * boton de la botonera, y soltar una pista SOBRE la botonera para copiarla a un
 * boton. Rust decide y persiste; la UI solo marca origen/destino.
 */

import { invoke } from '../bridge/api.js';
import { drawPlayerView } from './playerView.js';
import { drawGrid } from './grid.js';
import { appConfirm } from './appDialog.js';
import { t } from '../util/i18n.js';
import { alertIpcError } from './ipcError.js';

let _src = null;
let _wired = false;
let _onRefresh = null;

export function initPlayerDnd(onRefresh) {
    _onRefresh = onRefresh;
    if (_wired) return;
    _wired = true;
    document.addEventListener('mousedown', e => {
        if (e.button !== 0 || e.altKey) return;
        const row = _rowOf(e.target);
        if (!row) return;
        _src = Number(row.dataset.index);
        row.classList.add('drag-source');
        e.preventDefault(); // evita seleccionar texto al arrastrar
    });

    document.addEventListener('mousemove', e => {
        if (_src === null) return;
        _clearMarks('drag-over');
        _clearGridMarks();
        const row = _rowOf(e.target);
        if (row) { row.classList.add('drag-over'); return; }
        // Sobre la botonera se marca la celda destino, como con un archivo.
        e.target.closest?.('.grid-item[data-index]')?.classList.add('drag-over');
    });

    document.addEventListener('mouseup', async e => {
        if (_src === null) return;
        const from = _src;
        _src = null;
        _clearMarks('drag-over');
        _clearMarks('drag-source');
        _clearGridMarks();

        // Soltado sobre la botonera principal: copia la pista a ese boton.
        const cell = e.target.closest?.('.grid-item[data-index]');
        if (cell) { await _dropTrackOnGrid(from, cell); return; }

        // Soltado sobre otra fila: reordena la cola.
        const row = _rowOf(e.target);
        if (!row) return;
        const to = Number(row.dataset.index);
        if (to === from) return;
        try {
            await invoke('player_reorder_tracks', { fromIndex: from, toIndex: to });
            await drawPlayerView();
        } catch (err) {
            await alertIpcError(err);
        }
    });
}

/** Archivo del explorador: sobre una fila inserta ahi; en vacio anade al final. */
export async function dropFileOnPlayer(target, path) {
    await invoke('player_add_track', _args(target, { path }));
    await drawPlayerView();
}

/** Boton arrastrado desde la botonera o el panel fijo: se copia a la cola. */
export async function dropButtonOnPlayer(target, buttonId) {
    await invoke('player_add_button', _args(target, { buttonId }));
    await drawPlayerView();
}

/** Pista soltada sobre la botonera: se copia al boton (celda ocupada pregunta). */
async function _dropTrackOnGrid(trackIndex, cell) {
    if (cell.dataset.id && !await appConfirm(t('app.button_has_content'),
        { ok: t('app.replace'), cancel: t('app.dont_add') }, { ok: 1, cancel: 2 })) return;
    try {
        const grid = await invoke('assign_player_track_to_button', {
            trackIndex, toIndex: Number(cell.dataset.index),
        });
        drawGrid(grid, _onRefresh);
    } catch (err) {
        await alertIpcError(err);
    }
}

/** Anade la posicion de destino si se solto sobre una fila concreta. */
function _args(target, base) {
    const row = _rowOf(target);
    return row ? { ...base, index: Number(row.dataset.index) } : base;
}

function _rowOf(node) {
    return node?.closest?.('#player-rows .player-row') ?? null;
}

function _clearMarks(className) {
    document.querySelectorAll(`#player-rows .player-row.${className}`)
        .forEach(el => el.classList.remove(className));
}

function _clearGridMarks() {
    document.querySelectorAll('.grid-item.drag-over').forEach(el => el.classList.remove('drag-over'));
}
