/**
 * Archivo: appDialog.js
 * Proposito: dialogos internos para avisos y confirmaciones de la app.
 */

import { t } from '../util/i18n.js';

let _modal = null;
let _title = null;
let _body = null;
let _ok = null;
let _cancel = null;
let _resolve = null;

/** Muestra un aviso modal sin usar alert nativo del WebView. */
export function appAlert(message) {
    return _open(message, false);
}

/** Muestra una confirmacion modal sin usar confirm nativo del WebView. */
export function appConfirm(message) {
    return _open(message, true);
}

function _open(message, withCancel) {
    _ensureDialog();
    _title.textContent = t('app.title');
    _body.textContent = message;
    _cancel.classList.toggle('hidden', !withCancel);
    _modal.classList.remove('hidden');
    _ok.focus();
    return new Promise(resolve => { _resolve = resolve; });
}

function _ensureDialog() {
    if (_modal) return;

    _modal = document.createElement('div');
    _modal.id = 'app-dialog-modal';
    _modal.className = 'modal-overlay hidden';
    _modal.style.zIndex = '4900';
    _modal.innerHTML = `
      <div class="modal-content modal-xs">
        <div class="modal-header">
          <h3></h3>
          <button class="close-btn" type="button">x</button>
        </div>
        <div class="modal-body">
          <p class="confirm-text"></p>
        </div>
        <div class="modal-footer">
          <button class="btn-dark app-dialog-cancel" type="button"></button>
          <button class="btn-blue app-dialog-ok" type="button"></button>
        </div>
      </div>`;
    document.body.appendChild(_modal);

    _title = _modal.querySelector('h3');
    _body = _modal.querySelector('.confirm-text');
    _ok = _modal.querySelector('.app-dialog-ok');
    _cancel = _modal.querySelector('.app-dialog-cancel');
    _ok.textContent = t('edit_modal.ok');
    _cancel.textContent = t('edit_modal.cancel');

    _modal.querySelector('.close-btn').addEventListener('click', () => _finish(false));
    _cancel.addEventListener('click', () => _finish(false));
    _ok.addEventListener('click', () => _finish(true));
    _modal.addEventListener('click', e => {
        if (e.target === _modal) _finish(false);
    });
}

function _finish(value) {
    _modal?.classList.add('hidden');
    _resolve?.(value);
    _resolve = null;
}
