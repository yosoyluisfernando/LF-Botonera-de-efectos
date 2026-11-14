/** Presentación de Principal y selector de diseño para Windows y Linux. */
import { desktopAppearance, selectDesktopAppearance } from './desktopAppearance.js';
import { t } from '../util/i18n.js';
import { appAlert } from './appDialog.js';

export function initSettingsAppearance() {
    if (!__LF_DESKTOP_THEMES__ || !desktopAppearance().available) return;
    const main = document.getElementById('s-main');
    const appearance = section('themes.appearance');
    const fields = document.createElement('div');
    fields.className = 'settings-main-fields';
    const picker = document.createElement('div');
    picker.innerHTML = `
        <label for="config-desktop-style">${t('themes.choose')}</label>
        <select id="config-desktop-style">
          <option value="default">${t('themes.default')}</option>
          <option value="modern">${t('themes.modern')}</option>
        </select>`;
    const mode = field('config-theme');
    mode.querySelector('label').textContent = t('themes.color_mode');
    fields.append(picker, mode, field('config-language'), field('config-button-text-size'));
    const note = document.createElement('p');
    note.className = 'hint'; note.textContent = t('themes.immediate');
    const status = document.createElement('p');
    status.className = 'hint settings-appearance-status';
    status.setAttribute('role', 'status');
    status.setAttribute('aria-live', 'polite');
    appearance.append(fields, note, status);

    const toolbar = section('settings.toolbar_buttons');
    const buttons = document.getElementById('config-show-console').closest('.row');
    buttons.querySelector('[data-i18n="settings.toolbar_buttons"]').remove();
    toolbar.classList.add('settings-main-toolbar');
    toolbar.append(buttons, document.getElementById('config-show-bottom-bar').closest('.checkbox-line'));

    const outputs = section('themes.outputs');
    outputs.append(field('config-out-main'), field('config-out-pre'), field('player-device'));
    main.classList.add('settings-main-organized');
    main.replaceChildren(appearance, toolbar, outputs);

    const select = picker.querySelector('select');
    const refresh = () => { select.value = desktopAppearance().style; };
    document.addEventListener('lf-desktop-style-change', refresh);
    refresh();
    select.addEventListener('change', async () => {
        select.disabled = true;
        status.textContent = t('themes.applying');
        try {
            await selectDesktopAppearance(select.value);
            status.textContent = t('themes.applied');
        } catch (error) {
            refresh();
            status.textContent = t('themes.failed');
            console.error('Desktop appearance:', error);
            await appAlert(t('themes.failed'));
        } finally {
            select.disabled = false;
        }
    });
}

function section(title) {
    const fieldset = document.createElement('fieldset');
    fieldset.className = 'settings-main-group';
    const legend = document.createElement('legend');
    legend.textContent = t(title);
    fieldset.append(legend);
    return fieldset;
}

function field(id) {
    const select = document.getElementById(id);
    const label = select.previousElementSibling;
    const wrapper = document.createElement('div');
    label.htmlFor = id;
    select.style.removeProperty('margin-bottom');
    wrapper.append(label, select);
    return wrapper;
}
