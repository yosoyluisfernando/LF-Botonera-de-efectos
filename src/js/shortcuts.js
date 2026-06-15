/**
 * Archivo: shortcuts.js
 * Propósito: Escucha atajos de teclado globales y delega toda acción a Rust.
 * No contiene lógica de negocio (Regla 4).
 */

import { invoke } from './api.js';

let _config    = null;
let _onRefresh = null;
let _wired     = false;

export function initShortcuts(config, onRefresh) {
    _config    = config;
    _onRefresh = onRefresh;
    if (_wired) return;
    _wired = true;
    document.addEventListener('keydown', _handleKey);
}

export function updateShortcuts(config, onRefresh) {
    _config    = config;
    _onRefresh = onRefresh;
}

async function _handleKey(e) {
    // ESC cierra cualquier modal abierto, incluso con el foco en un input
    if (e.key === 'Escape') {
        document.querySelectorAll('.modal-overlay:not(.hidden)')
            .forEach(m => m.classList.add('hidden'));
        const pre = document.getElementById('prelisten-player');
        if (pre && !pre.classList.contains('hidden')) {
            document.getElementById('btn-stop-prelisten')?.click();
        }
        return;
    }

    // ENTER acepta el modal activo (funciona aunque el foco esté en un input)
    if (e.key === 'Enter') {
        const modal = document.querySelector('.modal-overlay:not(.hidden)');
        if (modal) {
            const btn = modal.querySelector(
                '#btn-save-settings, #btn-save-capture, #btn-save-edit, #btn-save-tab, #btn-save-profile'
            );
            if (btn && !btn.disabled) { e.preventDefault(); btn.click(); }
            return;
        }
    }

    const tag = e.target.tagName;
    // No capturar atajos si el foco está en un campo de texto
    if (tag === 'INPUT' || tag === 'TEXTAREA') return;
    if (!_config) return;

    const key = _buildKey(e);
    if (!key) return;

    const profile = _config.profiles?.find(p => p.id === _config.active_profile_id);
    if (!profile) return;

    // Atajos globales del perfil (detener todo / pestaña siguiente / anterior)
    const keys = profile.audio ?? {};
    if (key === keys.key_stop && keys.key_stop) {
        e.preventDefault();
        await invoke('stop_all_audio');
        return;
    }
    if ((key === keys.key_next && keys.key_next) ||
        (key === keys.key_prev && keys.key_prev)) {
        e.preventDefault();
        await invoke('cycle_paleta', { offset: key === keys.key_next ? 1 : -1 });
        _onRefresh?.();
        return;
    }

    // Atajo de pestaña → cambiar paleta activa
    const tab = profile.paletas?.find(p => p.shortcut === key);
    if (tab) {
        e.preventDefault();
        await invoke('set_active_paleta', { profileId: profile.id, paletaId: tab.id });
        _onRefresh?.();
        return;
    }

    // Atajo de botón → reproducir (busca en todas las paletas del perfil)
    for (const paleta of (profile.paletas ?? [])) {
        const btn = paleta.botones?.find(b => b.shortcut === key && b.path);
        if (btn) {
            e.preventDefault();
            await invoke('play_audio', {
                id: btn.id, path: btn.path, volume: btn.vol ?? 1.0,
                loopMode:  btn.loop_mode  ?? false,
                stopOther: btn.stop_other ?? false,
                overlap:   btn.overlap    ?? false,
                restart:   btn.restart    ?? false,
            });
            return;
        }
    }
}

/** Construye la cadena de atajo normalizada, ej: "Ctrl+Shift+A". */
function _buildKey(e) {
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) return '';
    let k = '';
    if (e.ctrlKey)  k += 'Ctrl+';
    if (e.altKey)   k += 'Alt+';
    if (e.shiftKey) k += 'Shift+';
    k += e.key.length === 1 ? e.key.toUpperCase() : e.key;
    return k;
}
