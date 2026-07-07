/**
 * Archivo: gridPlayback.js
 * Propósito: Pinta el estado visual de las celdas según el tick de audio recibido.
 * Solo pinta; todo el cálculo de posición ocurre en Rust (Regla 4).
 * No registra listeners — main.js es el único punto de escucha de audio-tick.
 */

import { t } from '../util/i18n.js';

let _buttons = {};

/** grid.js llama esto en cada drawGrid para refrescar duraciones y flags. */
export function setPlaybackButtons(buttons) {
    _buttons = {};
    (buttons ?? []).forEach(b => { _buttons[b.id] = b; });
}

/** Actualiza el estado visual de las celdas. Llamar desde el handler de audio-tick. */
export function paintAudioTick(payload) {
    _paint(payload.buttons ?? []);
}

function _paint(ticks) {
    const playing = new Map(ticks.map(t => [t.id, t]));

    document.querySelectorAll('.grid-item[data-id]').forEach(cell => {
        const id      = cell.dataset.id;
        const timerEl = cell.querySelector('.timer');
        const barEl   = cell.querySelector('.progress-bar');
        const btn     = _buttons[id];

        if (playing.has(id)) {
            cell.classList.add('playing');
            const tick = playing.get(id);
            const pos = Number(tick?.pos) || 0;
            const dur = Number(tick?.duration) || 0;
            if (dur > 0) {
                const remaining = Math.max(0, Number(tick?.remaining) || 0);
                if (timerEl) timerEl.textContent = `${remaining.toFixed(1)}s`;
                if (barEl)   barEl.style.width   = `${(remaining / dur) * 100}%`;
            } else if (timerEl) {
                timerEl.textContent = `${pos.toFixed(1)}s`;
            }
        } else {
            cell.classList.remove('playing');
            if (timerEl) timerEl.textContent = _staticTimer(btn);
            if (barEl)   barEl.style.width   = '100%';
        }
    });
}

function _staticTimer(btn) {
    if (btn?.timer_label_key) return t(btn.timer_label_key);
    return btn?.duration_str || '';
}
