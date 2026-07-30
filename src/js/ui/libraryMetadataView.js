/** Crea una sola instancia accesible del modal en la ventana que lo solicita. */
import { t } from '../util/i18n.js';

export function ensureMetadataView(handlers) {
    let modal = document.getElementById('library-metadata-modal');
    if (modal) return modal;
    modal = document.createElement('div');
    modal.id = 'library-metadata-modal';
    modal.className = 'modal-overlay hidden';
    modal.innerHTML = `<section class="modal-content lm-modal" role="dialog"
      aria-modal="true" aria-labelledby="lm-title">
      <header class="modal-header">
        <h3 id="lm-title">${t('metadata_editor.title')}</h3>
        <button id="lm-close" class="close-btn" type="button"
          aria-label="${t('metadata_editor.close')}">✕</button>
      </header>
      <div class="modal-body lm-body">
        <div class="lm-heading">
          <span id="lm-kind" class="lm-kind"></span>
          <span id="lm-count"></span>
        </div>
        <div id="lm-fields" class="lm-fields"></div>
        <section class="lm-tags" aria-labelledby="lm-tags-title">
          <h4 id="lm-tags-title">${t('metadata_editor.tags')}</h4>
          <p id="lm-tags-help" class="lm-help">${t('metadata_editor.tags_help')}</p>
          <div class="lm-tags-list" role="list"></div>
          <input class="lm-tag-input" type="text" maxlength="80" list="lm-tag-suggestions"
            placeholder="${t('metadata_editor.tags_placeholder')}"
            aria-label="${t('metadata_editor.tags_input')}">
          <datalist id="lm-tag-suggestions" class="lm-tag-suggestions"></datalist>
        </section>
        <section id="lm-rename-section" class="lm-rename hidden">
          <label class="checkbox-line">
            <input id="lm-rename-enabled" type="checkbox">
            <span>${t('metadata_editor.rename_physical')}</span>
          </label>
          <div id="lm-rename-controls" class="lm-rename-controls hidden">
            <label for="lm-new-file">${t('metadata_editor.new_filename')}</label>
            <div class="lm-rename-row">
              <input id="lm-new-file" type="text">
              <button id="lm-rename" type="button" class="btn-dark">
                ${t('metadata_editor.rename_now')}</button>
            </div>
            <p class="lm-warning">${t('metadata_editor.rename_warning')}</p>
          </div>
        </section>
        <section id="lm-write-section" class="lm-write hidden">
          <label class="checkbox-line">
            <input id="lm-write-file" type="checkbox">
            <span>${t('metadata_editor.write_file')}</span>
          </label>
          <p class="lm-help">${t('metadata_editor.write_file_help')}</p>
        </section>
        <p id="lm-status" class="lm-status" role="status" aria-live="polite"></p>
      </div>
      <footer class="modal-footer">
        <button id="lm-cancel" type="button" class="btn-dark">${t('metadata_editor.cancel')}</button>
        <button id="lm-save" type="button" class="btn-blue">${t('metadata_editor.save')}</button>
      </footer>
    </section>`;
    document.body.appendChild(modal);
    modal.querySelector('#lm-close').addEventListener('click', handlers.close);
    modal.querySelector('#lm-cancel').addEventListener('click', handlers.close);
    modal.querySelector('#lm-save').addEventListener('click', handlers.save);
    modal.querySelector('#lm-rename').addEventListener('click', handlers.rename);
    modal.querySelector('#lm-rename-enabled').addEventListener('change', event =>
        modal.querySelector('#lm-rename-controls')
            .classList.toggle('hidden', !event.target.checked));
    modal.addEventListener('mousedown', event => {
        if (event.target === modal) handlers.close();
    });
    modal.addEventListener('keydown', event => keyboard(event, modal, handlers.close));
    return modal;
}

export function setMetadataBusy(modal, busy, message = '') {
    modal.querySelectorAll('button,input,textarea').forEach(control => {
        control.disabled = busy || control.dataset.lmDisabled === 'true';
    });
    modal.querySelector('#lm-status').textContent = message;
    modal.setAttribute('aria-busy', String(busy));
}

function keyboard(event, modal, close) {
    if (event.key === 'Escape') {
        event.preventDefault();
        return close();
    }
    if (event.key !== 'Tab') return;
    const focusable = [...modal.querySelectorAll(
        'button:not([disabled]),input:not([disabled]),textarea:not([disabled])')];
    if (!focusable.length) return;
    const first = focusable[0];
    const last = focusable.at(-1);
    if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
    }
}
