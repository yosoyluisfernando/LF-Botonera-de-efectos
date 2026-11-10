import { invoke, listen } from '../bridge/api.js';
import { isAndroid } from '../bridge/platform.js';
import { appAlert } from './appDialog.js';
import { t } from '../util/i18n.js';
import { getMidiField, initMidiField, setMidiField } from './midiControls.js';

let pendingSave = Promise.resolve();
let listRequest = 0;

export function initSettingsMidi() {
    initMidiField('config-midi-stop', 'config-midi-stop-capture', 'config-midi-stop-clear');
    initMidiField('config-midi-next', 'config-midi-next-capture', 'config-midi-next-clear');
    initMidiField('config-midi-prev', 'config-midi-prev-capture', 'config-midi-prev-clear');
    document.getElementById('config-midi-refresh')?.addEventListener('click', loadMidiDevices);
    document.getElementById('config-midi-enabled')?.addEventListener('change', () => {
        updateDeviceSelectionState();
        persistDeviceSelection();
    });
    if (isAndroid()) initAndroidMidi();
}

function initAndroidMidi() {
    const add = document.getElementById('android-midi-add');
    const usb = document.getElementById('android-midi-usb');
    const bluetooth = document.getElementById('android-midi-bluetooth');
    const status = document.getElementById('android-midi-status');
    add.classList.remove('hidden');
    document.getElementById('config-midi-refresh').classList.add('hidden');
    usb.addEventListener('click', async () => {
        const devices = await loadMidiDevices();
        status.textContent = t(devices.some(device => device.available && device.id.startsWith('usb:'))
            ? 'midi.select_device' : 'midi.no_usb');
    });
    bluetooth.addEventListener('click', async () => {
        bluetooth.disabled = usb.disabled = true;
        status.textContent = t('midi.scanning');
        try {
            const count = await invoke('midi_scan_bluetooth');
            await loadMidiDevices();
            status.textContent = t(count ? 'midi.select_device' : 'midi.no_bluetooth');
        } catch (error) {
            const key = `midi.${error}`;
            const message = t(key);
            status.textContent = message === key ? t('midi.bluetooth_unavailable') : message;
        } finally {
            bluetooth.disabled = usb.disabled = false;
        }
    });
    listen('midi-devices-changed', () => {
        if (!document.getElementById('settings-modal').classList.contains('hidden')) {
            void loadMidiDevices();
        }
    }).catch(console.error);
}

export async function loadSettingsMidi(config) {
    document.getElementById('config-midi-enabled').checked = !!config.midi?.enabled;
    const profile = config.profiles.find(p => p.id === config.active_profile_id);
    setMidiField('config-midi-stop', profile?.audio?.midi_stop);
    setMidiField('config-midi-next', profile?.audio?.midi_next);
    setMidiField('config-midi-prev', profile?.audio?.midi_prev);
    await loadMidiDevices();
}

export async function saveSettingsMidi() {
    const enabled = document.getElementById('config-midi-enabled').checked;
    const inputIds = [...document.querySelectorAll('[data-midi-device]:checked')]
        .map(input => input.value);
    ++listRequest;
    pendingSave = pendingSave.catch(() => {}).then(() => invoke('midi_set_config', { enabled, inputIds }));
    return pendingSave;
}

async function persistDeviceSelection() {
    try {
        await saveSettingsMidi();
        if (isAndroid()) await loadMidiDevices();
    } catch (error) {
        const config = await invoke('get_config').catch(() => null);
        if (config) await loadSettingsMidi(config);
        await appAlert(String(error));
    }
}

export function globalMidiBindings() {
    return {
        midiStop: getMidiField('config-midi-stop'),
        midiNext: getMidiField('config-midi-next'),
        midiPrev: getMidiField('config-midi-prev'),
    };
}

async function loadMidiDevices() {
    const host = document.getElementById('config-midi-devices');
    if (!host) return [];
    const request = ++listRequest;
    await pendingSave.catch(() => {});
    const devices = await invoke('midi_devices').catch(() => []);
    if (request !== listRequest) return devices;
    host.replaceChildren();
    if (!devices.length) {
        const p = document.createElement('p');
        p.className = 'hint';
        p.textContent = t('midi.no_devices');
        host.appendChild(p);
        return devices;
    }
    devices.forEach(device => host.appendChild(deviceRow(device)));
    updateDeviceSelectionState();
    return devices;
}

function deviceRow(device) {
    const label = document.createElement('label');
    label.className = 'checkbox-line';
    const input = document.createElement('input');
    input.type = 'checkbox';
    input.dataset.midiDevice = '1';
    input.value = device.id;
    input.checked = !!device.selected;
    input.addEventListener('change', persistDeviceSelection);
    const span = document.createElement('span');
    const status = device.connected ? t('midi.connected')
        : device.available ? t('midi.available') : t('midi.disconnected');
    span.textContent = `${device.label} — ${status}`;
    label.append(input, span);
    return label;
}

function updateDeviceSelectionState() {
    const enabled = !!document.getElementById('config-midi-enabled')?.checked;
    document.querySelectorAll('[data-midi-device]')
        .forEach(input => { input.disabled = !enabled; });
}
