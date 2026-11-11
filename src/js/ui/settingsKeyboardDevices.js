import { invoke, listen } from '../bridge/api.js';
import { t } from '../util/i18n.js';

export function initKeyboardDevices() {
    const disclosure = document.getElementById('android-keyboard-shortcuts');
    const section = document.createElement('section');
    section.id = 'android-keyboard-devices';
    const title = translatedElement('p', 'keyboard.title', 'fade-section-title');
    const list = document.createElement('ul');
    list.id = 'android-keyboard-device-list';
    list.className = 'keyboard-device-list';
    list.setAttribute('aria-live', 'polite');
    const hint = translatedElement('p', 'keyboard.bluetooth_hint', 'hint');
    const action = translatedElement('button', 'keyboard.bluetooth_settings', 'btn-dark');
    action.id = 'android-keyboard-bluetooth';
    action.type = 'button';
    const status = document.createElement('p');
    status.className = 'hint';
    status.setAttribute('role', 'status');
    section.append(title, list, hint, action, status);
    disclosure.insertBefore(section, document.getElementById('settings-keyboard-main'));

    let request = 0;
    const render = devices => {
        list.replaceChildren();
        if (!devices.length) {
            list.appendChild(translatedElement('li', 'keyboard.empty', 'hint'));
            return;
        }
        for (const device of devices) {
            const item = document.createElement('li');
            item.textContent = device.name;
            list.appendChild(item);
        }
    };
    const refresh = async () => {
        const current = ++request;
        try {
            const devices = await invoke('keyboard_devices');
            if (current === request) render(devices);
        } catch {
            if (current !== request) return;
            list.replaceChildren(translatedElement('li', 'keyboard.devices_unavailable', 'hint'));
        }
    };
    action.addEventListener('click', async () => {
        action.disabled = true;
        status.textContent = '';
        try {
            await invoke('keyboard_open_bluetooth_settings');
        } catch {
            status.textContent = t('keyboard.settings_unavailable');
        } finally {
            action.disabled = false;
        }
    });
    disclosure.addEventListener('toggle', () => {
        if (disclosure.open) void refresh();
    });
    window.addEventListener('focus', refresh);
    listen('keyboard-devices-changed', event => {
        ++request;
        render(event.payload);
    }).catch(console.error);
    void refresh();
}

function translatedElement(tag, key, className) {
    const element = document.createElement(tag);
    element.dataset.i18n = key;
    element.textContent = t(key);
    element.className = className;
    return element;
}
