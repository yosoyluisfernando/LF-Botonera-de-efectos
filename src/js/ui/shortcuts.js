/**
 * Archivo: shortcuts.js
 * Proposito: captura teclas con foco en la ventana y delega la accion a Rust.
 * JS solo maneja interaccion inmediata de UI; la resolucion vive en Rust.
 */

import { invoke } from '../bridge/api.js';
import { shortcutFromEvent } from '../util/keyInputs.js';

let _onRefresh = null;
let _wired = false;

export function initShortcuts(config, onRefresh) {
    _onRefresh = onRefresh;
    if (_wired) return;
    _wired = true;
    document.addEventListener('keydown', _preventBrowserFunctionKeys, true);
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
    e.preventDefault();

    try {
        const result = await invoke('handle_local_shortcut', { key });
        if (!result?.handled) return;
        if (result.refresh) _onRefresh?.();
    } catch (err) {
        console.error('Error al resolver atajo local:', err);
    }
}

function _preventBrowserFunctionKeys(e) {
    if (e.key === 'F3' || e.key === 'F5') e.preventDefault();
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
    const modal = _topModal();
    if (modal) {
        e.preventDefault();
        _closeModal(modal);
        return true;
    }
    const pre = document.getElementById('prelisten-player');
    if (pre && !pre.classList.contains('hidden')) {
        document.getElementById('btn-stop-prelisten')?.click();
    }
    return true;
}

function _handleEnter(e) {
    if (e.key !== 'Enter') return false;
    const modal = _topModal();
    if (!modal) return false;
    const btn = modal.querySelector(
        '.app-dialog-ok, #btn-save-settings, #btn-save-capture, #btn-save-edit, #btn-save-tab, #btn-save-profile'
    );
    if (btn && !btn.disabled) {
        e.preventDefault();
        btn.click();
    }
    return true;
}

function _topModal() {
    const modals = [...document.querySelectorAll('.modal-overlay:not(.hidden)')];
    return modals.sort((a, b) => _zIndex(a) - _zIndex(b)).at(-1) ?? null;
}

function _zIndex(el) {
    const value = Number(getComputedStyle(el).zIndex);
    return Number.isFinite(value) ? value : 0;
}

function _closeModal(modal) {
    if (modal.id === 'app-dialog-modal') {
        modal.querySelector('.close-btn')?.click();
        return;
    }
    if (modal.id === 'delete-confirm-modal') {
        document.getElementById('delete-confirm-close')?.click();
        return;
    }
    modal.classList.add('hidden');
}

function _isTextInput(target) {
    const tag = target?.tagName;
    return tag === 'INPUT' || tag === 'TEXTAREA';
}

/** Construye la cadena de atajo normalizada, ej: "Ctrl+Shift+A". */
function _buildKey(e) {
    return shortcutFromEvent(e);
}
