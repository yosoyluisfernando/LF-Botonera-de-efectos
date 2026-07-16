/**
 * Archivo: appDialog.js
 * Proposito: dialogos internos para avisos y confirmaciones de la app.
 */

import { t } from '../util/i18n.js';

let _modal = null;
let _title = null;
let _body = null;
let _ok = null;
let _alt = null;
let _cancel = null;
let _resolve = null;
let _remember = null;
let _values = null;
let _wantsRemember = false;

/** Muestra un aviso modal sin usar alert nativo del WebView. */
export function appAlert(message) {
    return _open(message, { buttons: { ok: true }, values: { ok: true, cancel: false } });
}

/**
 * Confirmacion modal de dos botones. Resuelve true al aceptar, false al cancelar.
 * `labels` (opcional) reetiqueta {ok, cancel}; por defecto Aceptar/Cancelar.
 */
export function appConfirm(message, labels, order) {
    return _open(message, { buttons: { ok: true, cancel: true }, labels, order, values: { ok: true, cancel: false } });
}

/**
 * Confirmacion de tres vias en orden ok / alt / cancel (izquierda a derecha).
 * `labels` reetiqueta {ok, alt, cancel}. Resuelve 'yes', 'no' o 'cancel';
 * cerrar (X o fondo) equivale a 'cancel'.
 */
export function appConfirm3(message, labels) {
    return _open(message, {
        buttons: { ok: true, alt: true, cancel: true },
        labels,
        order: { ok: 1, alt: 2, cancel: 3 },
        values: { ok: 'yes', alt: 'no', cancel: 'cancel' },
    });
}

/**
 * Confirmación con un check de "recordar siempre".
 * Devuelve `{ choice: true|false, remember: bool }`, para que quien pregunta
 * decida qué guardar. El resto de diálogos no cambian.
 */
export function appConfirmRemember(message, rememberLabel, labels) {
    return _open(message, {
        buttons: { ok: true, cancel: true },
        labels,
        values: { ok: true, cancel: false },
        remember: rememberLabel,
    });
}

function _open(message, opts) {
    _ensureDialog();
    _title.textContent = t('app.title');
    _body.textContent = message;
    const labels = opts.labels ?? {};
    const order = opts.order ?? { cancel: 1, ok: 2 };
    _ok.textContent = labels.ok ?? t('edit_modal.ok');
    _alt.textContent = labels.alt ?? '';
    _cancel.textContent = labels.cancel ?? t('edit_modal.cancel');
    _ok.style.order = order.ok ?? 2;
    _alt.style.order = order.alt ?? 0;
    _cancel.style.order = order.cancel ?? 1;
    _alt.classList.toggle('hidden', !opts.buttons.alt);
    _cancel.classList.toggle('hidden', !opts.buttons.cancel);
    _modal.classList.toggle('dialog-wide', !!opts.buttons.alt);
    _values = opts.values;
    _wantsRemember = !!opts.remember;
    // Check opcional "recordar siempre": se pide por `opts.remember` y su estado
    // viaja con la respuesta, para que quien pregunta guarde la decision.
    _remember.classList.toggle('hidden', !opts.remember);
    _remember.querySelector('span').textContent = opts.remember ?? '';
    _remember.querySelector('input').checked = false;
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
          <label class="checkbox-line app-dialog-remember hidden">
            <input type="checkbox"><span></span>
          </label>
        </div>
        <div class="modal-footer">
          <button class="btn-dark app-dialog-cancel" type="button"></button>
          <button class="btn-dark app-dialog-alt" type="button"></button>
          <button class="btn-blue app-dialog-ok" type="button"></button>
        </div>
      </div>`;
    document.body.appendChild(_modal);

    _title = _modal.querySelector('h3');
    _body = _modal.querySelector('.confirm-text');
    _ok = _modal.querySelector('.app-dialog-ok');
    _alt = _modal.querySelector('.app-dialog-alt');
    _cancel = _modal.querySelector('.app-dialog-cancel');
    _remember = _modal.querySelector('.app-dialog-remember');

    _modal.querySelector('.close-btn').addEventListener('click', () => _finish('cancel'));
    _cancel.addEventListener('click', () => _finish('cancel'));
    _alt.addEventListener('click', () => _finish('alt'));
    _ok.addEventListener('click', () => _finish('ok'));
    _modal.addEventListener('click', e => {
        if (e.target === _modal) _finish('cancel');
    });
}

function _finish(kind) {
    _modal?.classList.add('hidden');
    const remember = !!_remember?.querySelector('input')?.checked;
    // Con check: {choice, remember}. Sin el: el valor de siempre, para no
    // romper a quien ya llamaba a appConfirm/appConfirm3.
    _resolve?.(_wantsRemember ? { choice: _values?.[kind], remember } : _values?.[kind]);
    _resolve = null;
}
