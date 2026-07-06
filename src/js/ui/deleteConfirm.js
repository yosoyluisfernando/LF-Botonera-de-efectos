/**
 * Archivo: deleteConfirm.js
 * Proposito: muestra una confirmacion visual antes de borrar perfiles o pestañas.
 * La decision destructiva se ejecuta en Rust; este modulo solo recoge la opcion.
 */

import { t } from '../util/i18n.js';

let _wired = false;
let _resolve = null;

/** Abre el modal de confirmacion y resuelve con la accion elegida. */
export function confirmDelete(kind) {
    _wire();
    const modal = document.getElementById('delete-confirm-modal');
    document.getElementById('delete-confirm-title').textContent =
        t(`delete_confirm.title_${kind}`);
    document.getElementById('delete-confirm-body').textContent =
        t(`delete_confirm.body_${kind}`);
    modal.classList.remove('hidden');
    return new Promise(resolve => { _resolve = resolve; });
}

function _wire() {
    if (_wired) return;
    _wired = true;
    _bind('btn-delete-save', 'save_delete');
    _bind('btn-delete-only', 'delete');
    _bind('btn-delete-cancel', 'cancel');
    document.getElementById('delete-confirm-close')
        .addEventListener('click', () => _finish('cancel'));
    document.getElementById('delete-confirm-modal')
        .addEventListener('click', e => {
            if (e.target.id === 'delete-confirm-modal') _finish('cancel');
        });
}

function _bind(id, action) {
    document.getElementById(id).addEventListener('click', () => _finish(action));
}

function _finish(action) {
    document.getElementById('delete-confirm-modal')?.classList.add('hidden');
    _resolve?.(action);
    _resolve = null;
}
