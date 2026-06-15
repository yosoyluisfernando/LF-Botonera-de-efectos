/**
 * Archivo: prelisten.js
 * Propósito: Panel flotante de pre-escucha (Fase 3). Reproduce mediante el
 * motor Rust con el id reservado "__prelisten__" y pinta el progreso usando
 * el evento "audio-tick" de audio_monitor.rs. Solo pinta (Regla 4).
 */

import { invoke } from './api.js';

const PRELISTEN_ID = '__prelisten__';

let _duration  = 0;
let _wasActive = false;
let _wired     = false;

/**
 * Abre el panel y reproduce el archivo en modo pre-escucha.
 * @param {string} path     Ruta del archivo de audio.
 * @param {string} name     Nombre a mostrar en el panel.
 * @param {number} vol      Volumen inicial (0..1).
 * @param {number} duration Duración total en segundos (0 = desconocida).
 */
export async function openPrelisten(path, name, vol, duration) {
    if (!path) return;
    _duration = duration > 0 ? duration : 0;
    _wireOnce();

    document.getElementById('prelisten-name').textContent = name || '';
    document.getElementById('prelisten-volume').value     = vol;
    document.getElementById('prelisten-player').classList.remove('hidden');

    try {
        await invoke('play_audio', {
            id: PRELISTEN_ID, path, volume: vol,
            loopMode: false, stopOther: false, overlap: false, restart: true,
        });
        _wasActive = true;
    } catch (e) { console.error('Error en pre-escucha:', e); }
}

/** Detiene la reproducción y oculta el panel. */
export function stopPrelisten() {
    invoke('stop_audio', { id: PRELISTEN_ID });
    _wasActive = false;
    document.getElementById('prelisten-player').classList.add('hidden');
}

function _wireOnce() {
    if (_wired) return;
    _wired = true;

    document.getElementById('close-prelisten').addEventListener('click', stopPrelisten);
    document.getElementById('btn-stop-prelisten').addEventListener('click', stopPrelisten);
    document.getElementById('prelisten-volume').addEventListener('input', e => {
        invoke('set_audio_volume', { id: PRELISTEN_ID, volume: parseFloat(e.target.value) });
    });

    window.addEventListener('lf-audio-tick', e => updatePrelistenTick(e.detail));
}

/** Recibe audio-tick desde startup.js y actualiza solo si el panel está activo. */
export function updatePrelistenTick(payload) {
    const ticks = Array.isArray(payload) ? payload : (payload?.buttons ?? []);
    const tick = ticks.find(t => t.id === PRELISTEN_ID);

    if (!tick) {
        // El audio terminó por sí solo: cerrar el panel como hacía la maqueta
        if (_wasActive) stopPrelisten();
        return;
    }

    const bar  = document.getElementById('prelisten-progress');
    const time = document.getElementById('prelisten-time');
    if (_duration > 0) {
        bar.style.width  = `${Math.min(100, (tick.pos / _duration) * 100)}%`;
        time.textContent = `${_fmt(tick.pos)} / ${_fmt(_duration)}`;
    } else {
        time.textContent = _fmt(tick.pos);
    }
}

/** Formatea segundos como mm:ss. */
function _fmt(secs) {
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}
