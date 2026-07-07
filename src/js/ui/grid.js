/**
 * Archivo: grid.js
 * Propósito: Dibuja las celdas de la cuadrícula activa y retransmite el clic
 * al motor Rust. UI = control remoto tonto (Regla 4).
 * El drag & drop vive en gridDnd.js y el progreso en gridPlayback.js (Regla 3).
 */

import { invoke } from '../bridge/api.js';
import { showContextMenu } from './contextMenu.js';
import { setPlaybackButtons } from './gridPlayback.js';
import { isMapping, captureButton } from './mapping.js';
import { paintAdaptive } from '../util/colorAdapter.js';
import { typeIcon } from '../util/typeIcons.js';
import { t } from '../util/i18n.js';

let _onRefresh = null;

/** Guarda el callback global de refresco que usan las celdas. */
export function initGrid(onRefresh) {
    _onRefresh = onRefresh;
}

/**
 * Renderiza las celdas de la cuadrícula a partir del GridState devuelto por Rust.
 * @param {object}   gridState  {columns, rows, buttons[]}
 * @param {Function} onRefresh  Callback para refrescar toda la UI.
 */
export function drawGrid(gridState, onRefresh) {
    if (onRefresh) _onRefresh = onRefresh;

    const gridEl = document.getElementById('main-grid');
    if (!gridEl) return;

    setPlaybackButtons(gridState.buttons);

    gridEl.innerHTML = '';
    gridEl.style.gridTemplateColumns = `repeat(${gridState.columns}, 1fr)`;
    gridEl.style.gridTemplateRows    = `repeat(${gridState.rows}, 1fr)`;

    const total = gridState.columns * gridState.rows;
    for (let i = 0; i < total; i++) {
        const index   = i + 1;
        const btnData = gridState.buttons.find(b => b.index === index) ?? null;
        gridEl.appendChild(_makeCell(index, btnData));
    }
}

function _makeCell(index, btnData) {
    const btn = document.createElement('div');
    btn.className     = 'grid-item';
    btn.dataset.index = index;

    if (btnData) {
        btn.dataset.id            = btnData.id;
        paintAdaptive(btn, btnData.color_bg, btnData.color_text, 'button');
        const shortcutHtml = btnData.shortcut
            ? `<span class="shortcut-badge">${btnData.shortcut}</span>`
            : '';
        const icon = btnData.type_icon ? typeIcon(btnData.type_icon) : '';
        const timerText = _staticTimer(btnData);
        btn.innerHTML = `
            <span class="index">${btnData.index}</span>
            ${shortcutHtml}
            <span class="label">${icon}${btnData.name || btnData.label}</span>
            <span class="timer">${timerText}</span>
            <div class="progress-container">
              <div class="progress-bar"></div>
            </div>`;

        btn.addEventListener('click', e => {
            if (e.altKey) return; // Alt+clic = reordenar (gridDnd.js)
            if (isMapping()) { captureButton(btnData); return; }
            invoke('play_button', { id: btnData.id }).catch(console.error);
        });
    } else {
        btn.innerHTML = `<span class="index">${index}</span>`;
        // Doble clic (no clic simple) para no entorpecer el flujo de trabajo
        btn.addEventListener('dblclick', async () => {
            if (isMapping()) return;
            try {
                const s = await invoke('assign_file_to_button', { index, path: null });
                if (s) drawGrid(s, _onRefresh);
            } catch (_) { /* Usuario canceló */ }
        });
    }

    btn.addEventListener('contextmenu', e => {
        e.preventDefault();
        // Refresco COMPLETO: shortcuts.js y tabs.js necesitan la config nueva
        showContextMenu(e.clientX, e.clientY, index, btnData, () => _onRefresh?.());
    });

    return btn;
}

function _staticTimer(btnData) {
    if (btnData.timer_label_key) return t(btnData.timer_label_key);
    return btnData.duration_str || '';
}
