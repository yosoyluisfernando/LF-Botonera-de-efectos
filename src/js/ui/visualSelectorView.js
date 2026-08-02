import { t } from '../util/i18n.js';
import { typeIcon } from '../util/typeIcons.js';
import { createSvgVisual } from './visualSvg.js';

export function createSelectorModal() {
    const modal = document.createElement('div');
    modal.className = 'modal-overlay hidden visual-selector-overlay';
    modal.innerHTML = `<div class="modal-content visual-selector">
      <div class="modal-header"><h3></h3><button class="close-btn">×</button></div>
      <div class="visual-tabs">
        <button data-kind="emoji"></button><button data-kind="basic"></button>
      </div>
      <div class="visual-search-row">
        <input class="visual-search" type="search"><select class="visual-group"></select>
      </div>
      <div class="visual-options">
        <label class="visual-skin-tones"><input type="checkbox"><span></span></label>
        <span class="visual-current-category"></span>
      </div>
      <div class="visual-results"></div>
      <div class="modal-footer">
        <span class="visual-count"></span><button class="visual-cancel btn-dark"></button>
      </div>
    </div>`;
    document.body.appendChild(modal);
    modal.querySelector('h3').textContent = t('visuals.title');
    modal.querySelector('[data-kind="emoji"]').textContent = t('visuals.emojis');
    modal.querySelector('[data-kind="basic"]').textContent = t('visuals.basics');
    modal.querySelector('.visual-search').placeholder = t('visuals.search');
    modal.querySelector('.visual-cancel').textContent = t('edit_modal.cancel');
    modal.querySelector('.close-btn').ariaLabel = t('visuals.close');
    modal.querySelector('.visual-skin-tones span').textContent =
        t('visuals.include_skin_tones');
    modal.querySelector('.visual-results').dataset.loadingLabel = t('visuals.loading');
    return modal;
}

export function createResult(item, kind, onSelect) {
    const button = document.createElement('button');
    button.className = 'visual-result';
    const svg = createSvgVisual(kind, item.value, 'visual-result-svg');
    if (svg) {
        button.appendChild(svg);
    } else {
        button.textContent = item.value;
        button.classList.add(kind === 'basic'
            ? 'material-symbols-rounded' : 'button-visual-emoji');
    }
    button.title = item.name;
    button.setAttribute('aria-label', item.name);
    button.onclick = () => onSelect(item.value);
    return button;
}

export function paintPreview(preview, visual, buttonType) {
    preview.className = 'edit-visual-preview';
    preview.replaceChildren();
    if (visual.mode === 'text') {
        preview.textContent = '—';
        preview.classList.add('edit-visual-preview-empty');
        return;
    }
    const svg = createSvgVisual(visual.kind, visual.value);
    if (svg) {
        preview.appendChild(svg);
    } else if (visual.kind === 'emoji') {
        preview.textContent = visual.value;
        preview.classList.add('button-visual-emoji');
    } else if (visual.kind === 'basic') {
        preview.textContent = visual.value;
        preview.classList.add('material-symbols-rounded');
    } else if (visual.kind === 'auto') {
        preview.innerHTML = typeIcon(buttonType);
        preview.classList.add('edit-visual-preview-auto');
    } else {
        preview.textContent = '—';
        preview.classList.add('edit-visual-preview-empty');
    }
}

export function labelForGroup(id) {
    const value = t(`visuals.groups.${id}`);
    return value.startsWith('visuals.groups.') ? id : value;
}

export function labelForKind(kind) {
    return t(`visuals.${kind}s`);
}
