/**
 * Módulo: settingsFade.js
 * Propósito: sección de fundidos (fade) en el panel principal de ajustes.
 * Carga y guarda FadeConfig a través de IPC (Regla 4, Regla 6).
 */
import { invoke } from './api.js';

/** Carga los valores actuales de fade desde el AppConfig ya obtenido. */
export function loadFadePanel(config) {
    const fade = config?.fade || {};
    _set('fade-in', fade.fade_in_s ?? 0);
    _set('fade-out-stop', fade.fade_out_stop_s ?? 0);
    _set('fade-out-end', fade.fade_out_end_s ?? 0);
}

/** Persiste los valores de fade en Rust. */
export async function saveFade() {
    await invoke('set_fade_config', {
        configIn: {
            fade_in_s: _get('fade-in'),
            fade_out_stop_s: _get('fade-out-stop'),
            fade_out_end_s: _get('fade-out-end'),
        },
    });
}

function _set(id, value) {
    const el = document.getElementById(id);
    if (el) el.value = value;
}
function _get(id) {
    return parseFloat(document.getElementById(id)?.value || '0') || 0;
}
