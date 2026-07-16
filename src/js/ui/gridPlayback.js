/**
 * Archivo: gridPlayback.js
 * Propósito: Pinta el estado visual de las celdas según el tick de audio recibido.
 * Solo pinta; todo el cálculo de posición ocurre en Rust (Regla 4).
 * No registra listeners — main.js es el único punto de escucha de audio-tick.
 */

import { paintPlayback } from './playbackPainter.js';

let _buttons = {};

/** grid.js llama esto en cada drawGrid para refrescar duraciones y flags. */
export function setPlaybackButtons(buttons) {
    _buttons = {};
    (buttons ?? []).forEach(b => { _buttons[b.id] = b; });
}
/** Actualiza el estado visual de las celdas. Llamar desde el handler de audio-tick. */
export function paintAudioTick(payload) {
    paintPlayback('.grid-item[data-id]', _buttons,
        (payload.buttons ?? []).filter(tick => tick.group !== 'fixed'));
}
