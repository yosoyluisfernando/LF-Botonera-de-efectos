/**
 * Archivo: gridPlayback.js
 * Propósito: Pinta el estado visual de las celdas según el tick de audio recibido.
 * Solo pinta; todo el cálculo de posición ocurre en Rust (Regla 4).
 * No registra listeners — main.js es el único punto de escucha de audio-tick.
 */

import { getCurrentMode } from './playbackModes.js';

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
    const playing    = new Map(ticks.map(t => [t.id, t.pos]));
    const globalMode = getCurrentMode();

    document.querySelectorAll('.grid-item[data-id]').forEach(cell => {
        const id      = cell.dataset.id;
        const timerEl = cell.querySelector('.timer');
        const barEl   = cell.querySelector('.progress-bar');
        const btn     = _buttons[id];

        if (playing.has(id)) {
            cell.classList.add('playing');
            const pos = playing.get(id);
            const dur = btn?.duration > 0 ? btn.duration : 0;
            if (dur > 0) {
                const isLooping = globalMode === 'loop' || (globalMode === 'normal' && btn?.loop_mode);
                const local     = isLooping ? pos % dur : Math.min(pos, dur);
                const remaining = Math.max(0, dur - local);
                if (timerEl) timerEl.textContent = `${remaining.toFixed(1)}s`;
                if (barEl)   barEl.style.width   = `${(remaining / dur) * 100}%`;
            } else if (timerEl) {
                timerEl.textContent = `${pos.toFixed(1)}s`;
            }
        } else {
            cell.classList.remove('playing');
            if (timerEl) timerEl.textContent = btn?.duration_str || '';
            if (barEl)   barEl.style.width   = '100%';
        }
    });
}
