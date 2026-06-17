/**
 * Archivo: tabDnd.js
 * Proposito: capturar Alt + arrastre sobre pestanas y delegar el reordenado a Rust.
 * La UI solo muestra origen/destino; el orden real se valida y persiste en backend.
 */

import { invoke } from './api.js';
import { appAlert } from './appDialog.js';
import { t } from './i18n.js';

let _config = null;
let _onRefresh = null;
let _srcPaletaId = '';
let _wired = false;

/** Conecta el arrastre de pestanas una sola vez y conserva config actual. */
export function initTabDnd(config, onRefresh) {
    updateTabDnd(config, onRefresh);
    if (_wired) return;
    _wired = true;
    _wire();
}

/** Actualiza dependencias usadas por el gesto tras cada refresco de config. */
export function updateTabDnd(config, onRefresh) {
    _config = config;
    _onRefresh = onRefresh;
}

function _wire() {
    document.addEventListener('mousedown', e => {
        if (!e.altKey || e.button !== 0) return;
        const tab = _tabFrom(e.target);
        if (!tab) return;
        _srcPaletaId = tab.dataset.paletaId;
        tab.classList.add('tab-drag-source');
        e.preventDefault();
    });

    document.addEventListener('mousemove', e => {
        if (!_srcPaletaId) return;
        _markTarget(_tabFrom(e.target));
    });

    document.addEventListener('mouseup', e => {
        if (!_srcPaletaId) return;
        const fromPaletaId = _srcPaletaId;
        const toPaletaId = _tabFrom(e.target)?.dataset.paletaId ?? '';
        _srcPaletaId = '';
        _clearMarks();
        void _drop(fromPaletaId, toPaletaId);
    });
}

async function _drop(fromPaletaId, toPaletaId) {
    if (!toPaletaId || toPaletaId === fromPaletaId) return;
    try {
        await invoke('reorder_paletas', {
            profileId: _config.active_profile_id,
            fromPaletaId,
            toPaletaId,
        });
        _onRefresh?.();
    } catch (err) {
        const key = `errors.${err}`;
        const msg = t(key);
        await appAlert(msg === key ? String(err) : msg);
    }
}

function _markTarget(tab) {
    document.querySelectorAll('.tab.tab-drag-over')
        .forEach(el => el.classList.remove('tab-drag-over'));
    if (tab?.dataset.paletaId !== _srcPaletaId) {
        tab?.classList.add('tab-drag-over');
    }
}

function _clearMarks() {
    document.querySelectorAll('.tab.tab-drag-source, .tab.tab-drag-over')
        .forEach(el => el.classList.remove('tab-drag-source', 'tab-drag-over'));
}

function _tabFrom(target) {
    return target.closest?.('#tabs-list .tab[data-paleta-id]') ?? null;
}
