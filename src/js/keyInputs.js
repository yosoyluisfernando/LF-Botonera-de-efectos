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
    if (['Control', 'Alt', 'Shift', 'Meta', 'Enter'].includes(e.key)) return;
    let key = '';
    if (e.ctrlKey) key += 'Ctrl+';
    if (e.altKey) key += 'Alt+';
    if (e.shiftKey) key += 'Shift+';
    key += e.key.toUpperCase();
    e.currentTarget.value = key;
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
