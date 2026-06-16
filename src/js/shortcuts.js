/**
 * Archivo: shortcuts.js
 * Proposito: captura teclas con foco en la ventana y delega la accion a Rust.
 * JS solo maneja interaccion inmediata de UI; la resolucion vive en Rust.
 */

import { invoke } from './api.js';

let _onRefresh = null;
let _wired = false;

export function initShortcuts(config, onRefresh) {
    _onRefresh = onRefresh;
    if (_wired) return;
    _wired = true;
    document.addEventListener('keydown', _handleKey);
}

export function updateShortcuts(config, onRefresh) {
    _onRefresh = onRefresh;
}

async function _handleKey(e) {
    if (_handleEscape(e)) return;
    if (_handleEnter(e)) return;
    if (_isTextInput(e.target)) return;
    if (await _handleHistoryShortcut(e)) return;

    const key = _buildKey(e);
    if (!key) return;

    try {
        const result = await invoke('handle_local_shortcut', { key });
        if (!result?.handled) return;
        e.preventDefault();
        if (result.refresh) _onRefresh?.();
    } catch (err) {
        console.error('Error al resolver atajo local:', err);
    }
}

async function _handleHistoryShortcut(e) {
    if (!e.ctrlKey || e.shiftKey || e.key.toLowerCase() !== 'z') return false;
    e.preventDefault();
    try {
        await invoke(e.altKey ? 'redo_config' : 'undo_config');
        _onRefresh?.();
    } catch (err) {
        if (!String(err).startsWith('nothing_to_')) {
            console.error('Error al restaurar historial:', err);
        }
    }
    return true;
}

function _handleEscape(e) {
    if (e.key !== 'Escape') return false;
    document.querySelectorAll('.modal-overlay:not(.hidden)')
        .forEach(m => m.classList.add('hidden'));
    const pre = document.getElementById('prelisten-player');
    if (pre && !pre.classList.contains('hidden')) {
        document.getElementById('btn-stop-prelisten')?.click();
    }
    return true;
}

function _handleEnter(e) {
    if (e.key !== 'Enter') return false;
    const modal = document.querySelector('.modal-overlay:not(.hidden)');
    if (!modal) return false;
    const btn = modal.querySelector(
        '#btn-save-settings, #btn-save-capture, #btn-save-edit, #btn-save-tab, #btn-save-profile'
    );
    if (btn && !btn.disabled) {
        e.preventDefault();
        btn.click();
    }
    return true;
}

function _isTextInput(target) {
    const tag = target?.tagName;
    return tag === 'INPUT' || tag === 'TEXTAREA';
}

/** Construye la cadena de atajo normalizada, ej: "Ctrl+Shift+A". */
function _buildKey(e) {
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) return '';
    let k = '';
    if (e.ctrlKey) k += 'Ctrl+';
    if (e.altKey) k += 'Alt+';
    if (e.shiftKey) k += 'Shift+';
    k += e.key.length === 1 ? e.key.toUpperCase() : e.key;
    return k;
}
