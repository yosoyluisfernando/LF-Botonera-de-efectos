/**
 * Archivo: playbackModes.js
 * Propósito: Gestiona los botones de modo de reproducción global de la barra inferior.
 * El modo activo se persiste por perfil en Rust (cmd_playback.rs, Regla 2).
 * Los demás módulos (ej. gridPlayback.js) consultan getCurrentMode() para adaptar el pintado.
 */

import { invoke } from './api.js';

const MODES = ['normal', 'loop', 'overlap', 'restart', 'stop_others'];

let _currentMode = 'normal';
let _wired       = false;

/** Devuelve el modo activo actual (consulta local; Rust es fuente de verdad en disco). */
export function getCurrentMode() { return _currentMode; }

export function initPlaybackModes() {
    // Cargar modo guardado del perfil activo
    invoke('get_playback_mode')
        .then(mode => { _currentMode = mode || 'normal'; _updateUI(); })
        .catch(console.error);

    if (_wired) return;
    _wired = true;

    // Botones de modo (radio exclusivo)
    MODES.forEach(mode => {
        document.getElementById(`pb-btn-${mode}`)?.addEventListener('click', () => {
            _currentMode = mode;
            _updateUI();
            invoke('set_playback_mode', { mode }).catch(console.error);
        });
    });

    // Botón Detener Todo
    document.getElementById('pb-btn-stop-all')?.addEventListener('click', () => {
        invoke('stop_all_audio').catch(console.error);
    });
}

function _updateUI() {
    MODES.forEach(mode => {
        document.getElementById(`pb-btn-${mode}`)
            ?.classList.toggle('active', mode === _currentMode);
    });
}
