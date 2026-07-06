/**
 * Archivo: keyInputs.js
 * Propósito: Captura y limpieza visible de campos de atajo de teclado.
 */

import { t } from './i18n.js';

/** Inicializa todos los .key-input actuales una sola vez por campo. */
export function initKeyInputs(root = document) {
    root.querySelectorAll('.key-input').forEach(input => {
        if (!input.dataset.keyInputReady) {
            input.addEventListener('keydown', _captureKey);
            input.dataset.keyInputReady = '1';
        }
        _ensureClearButton(input);
    });
}

function _captureKey(e) {
    e.preventDefault();
    if (['Escape', 'Backspace', 'Delete'].includes(e.key)) {
        e.currentTarget.value = '';
        return;
    }
    const key = shortcutFromEvent(e);
    if (!key) return;
    e.currentTarget.value = key;
}

/** Normaliza KeyboardEvent a la forma persistida: Ctrl+Shift+F1, Space, etc. */
export function shortcutFromEvent(e) {
    if (['Control', 'Alt', 'Shift', 'Meta', 'Enter'].includes(e.key)) return '';
    const base = _baseKey(e);
    if (!base) return '';
    let key = '';
    if (e.ctrlKey) key += 'Ctrl+';
    if (e.altKey) key += 'Alt+';
    if (e.shiftKey) key += 'Shift+';
    return `${key}${base}`;
}

function _baseKey(e) {
    if (e.code === 'Space' || e.key === ' ') return 'Space';
    if (/^F([1-9]|1[0-9]|2[0-4])$/.test(e.key)) return e.key;
    if (e.key.length === 1) return e.key.toUpperCase();
    return e.key;
}

function _ensureClearButton(input) {
    const next = input.nextElementSibling;
    if (next?.classList.contains('key-clear-btn')) return;

    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'key-clear-btn';
    btn.textContent = '×';
    btn.title = t('settings.clear_shortcut');
    btn.addEventListener('click', () => {
        input.value = '';
        input.focus();
    });

    const wrap = document.createElement('div');
    wrap.className = 'key-input-wrap';
    input.parentNode.insertBefore(wrap, input);
    wrap.appendChild(input);
    wrap.appendChild(btn);
}
