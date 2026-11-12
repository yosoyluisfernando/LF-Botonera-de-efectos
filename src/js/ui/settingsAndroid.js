import { isAndroid } from '../bridge/platform.js';
import { t } from '../util/i18n.js';
import { initKeyboardDevices } from './settingsKeyboardDevices.js';

export function initAndroidSettingsNavigation() {
    if (!isAndroid()) return;
    arrangeMidiSettings();
    initKeyboardDevices();
    const modal = document.getElementById('settings-modal');
    const header = modal.querySelector('.modal-header');
    const title = header.querySelector('h3');
    const tabs = modal.querySelector('.settings-tabs');
    tabs.appendChild(tabs.querySelector('[data-target="s-about"]'));
    const content = modal.querySelector('.settings-content');
    const back = document.createElement('button');
    back.type = 'button';
    back.className = 'android-settings-back btn-dark';
    back.textContent = t('mobile.settings_back');
    header.prepend(back);

    back.addEventListener('click', () => {
        modal.dataset.androidView = 'sections';
        tabs.querySelector('.active')?.focus();
    });
    tabs.addEventListener('click', event => {
        if (!event.target.closest('.s-tab')) return;
        modal.dataset.androidView = 'detail';
        content.scrollTop = 0;
        title.focus({ preventScroll: true });
    });
    title.tabIndex = -1;
}

export function resetAndroidSettingsNavigation() {
    if (isAndroid()) document.getElementById('settings-modal').dataset.androidView = 'sections';
}

function arrangeMidiSettings() {
    const panel = document.getElementById('s-atajos');
    const tab = document.querySelector('[data-target="s-atajos"]');
    tab.dataset.i18n = 'settings.tab_midi_keys';
    tab.textContent = t('settings.tab_midi_keys');
    const keyboard = document.createElement('details');
    keyboard.id = 'android-keyboard-shortcuts';
    keyboard.className = 'android-keyboard-shortcuts';
    const summary = document.createElement('summary');
    summary.dataset.i18n = 'settings.keyboard_shortcuts';
    summary.textContent = t('settings.keyboard_shortcuts');
    keyboard.append(summary, document.getElementById('settings-keyboard-main'),
        document.getElementById('settings-keyboard-extra'));
    panel.append(keyboard);

    const discovery = document.getElementById('android-midi-add');
    const section = document.createElement('section');
    section.id = discovery.id;
    section.className = discovery.className;
    discovery.querySelector('summary').remove();
    section.append(...discovery.childNodes);
    discovery.replaceWith(section);
    const hint = document.createElement('p');
    hint.className = 'hint';
    hint.dataset.i18n = 'midi.bluetooth_hint';
    hint.textContent = t('midi.bluetooth_hint');
    section.prepend(hint);
}
