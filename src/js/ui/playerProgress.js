/**
 * Archivo: playerProgress.js
 * Propósito: la fila de progreso del reproductor: contador, barra con seek y
 * botón Loop. La UI solo pinta y pide: Rust decide si se puede saltar
 * (`can_seek`), recuerda si el contador va en transcurrido o restante, y suma el
 * total de la lista.
 */

import { invoke } from '../bridge/api.js';
import { mmss, hhmmss } from '../util/durationFormat.js';

const STEPS = 1000; // resolución de la barra; el tiempo real lo pone Rust

let _wired = false;
let _totalS = 0;          // total de la lista: se enseña con el reproductor parado
let _display = 'elapsed'; // "elapsed" | "remaining"
let _dragging = false;    // mientras se arrastra, el tick no repinta la barra

export function initPlayerProgress() {
    if (_wired) return;
    _wired = true;

    const bar = document.getElementById('player-progress');
    // Al arrastrar no se pinta encima: el tick llega cada 100 ms y daría saltos.
    bar.addEventListener('pointerdown', () => { _dragging = true; });
    bar.addEventListener('input', () => _paintTimeFromBar());
    bar.addEventListener('change', async () => {
        const dur = Number(bar.dataset.durationS) || 0;
        _dragging = false;
        if (dur <= 0) return;
        await invoke('player_seek', { positionS: (bar.value / STEPS) * dur })
            .catch(console.error);
    });

    document.getElementById('player-loop').addEventListener('click', async () => {
        const snap = await invoke('get_player_snapshot');
        await invoke('player_set_loop', { enabled: !snap.loop_current });
    });

    document.getElementById('player-time').addEventListener('click', async () => {
        _display = await invoke('player_toggle_time_display');
        _paintFromCache();
    });
}

/** El total lo suma Rust; aquí solo se recuerda para pintarlo al estar parado. */
export function setQueueTotal(totalS, timeDisplay) {
    _totalS = Number(totalS) || 0;
    if (timeDisplay) _display = timeDisplay;
}

export function paintPlayerProgress(snapshot) {
    const bar = document.getElementById('player-progress');
    if (!bar) return;
    const dur = Number(snapshot.duration_s) || 0;
    const pos = Math.min(Number(snapshot.position_s) || 0, dur);

    bar.dataset.durationS = dur;
    bar.dataset.positionS = pos;
    // Solo se puede arrastrar en una pista de un archivo con duración conocida:
    // una locución son varios encadenados y no se puede reposicionar.
    bar.disabled = !snapshot.can_seek;
    if (!_dragging) {
        bar.value = dur > 0 ? Math.round((pos / dur) * STEPS) : 0;
    }

    document.getElementById('player-loop')
        ?.classList.toggle('active', !!snapshot.loop_current);
    _paintTime(pos, dur);
}

/** Con pista cargada, el contador es de la CANCIÓN; parado, el total de la lista. */
function _paintTime(pos, dur) {
    const el = document.getElementById('player-time');
    if (!el) return;
    if (dur <= 0) {
        el.textContent = hhmmss(_totalS);
        el.classList.add('is-total');
        return;
    }
    el.classList.remove('is-total');
    el.textContent = _display === 'remaining'
        ? `-${mmss(Math.max(0, dur - pos))}`
        : mmss(pos);
}

/** Mientras se arrastra, el contador sigue al pulgar, no al audio. */
function _paintTimeFromBar() {
    const bar = document.getElementById('player-progress');
    const dur = Number(bar.dataset.durationS) || 0;
    _paintTime((bar.value / STEPS) * dur, dur);
}

function _paintFromCache() {
    const bar = document.getElementById('player-progress');
    _paintTime(Number(bar?.dataset.positionS) || 0, Number(bar?.dataset.durationS) || 0);
}
