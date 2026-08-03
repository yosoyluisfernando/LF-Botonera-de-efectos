import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { getMidiField, initMidiField, setMidiField } from './midiControls.js';

export function initSettingsMidi() {
    initMidiField('config-midi-stop', 'config-midi-stop-capture', 'config-midi-stop-clear');
    initMidiField('config-midi-next', 'config-midi-next-capture', 'config-midi-next-clear');
    initMidiField('config-midi-prev', 'config-midi-prev-capture', 'config-midi-prev-clear');
    document.getElementById('config-midi-refresh')?.addEventListener('click', loadMidiDevices);
    document.getElementById('config-midi-enabled')?.addEventListener('change', () => {
        updateDeviceSelectionState();
        saveSettingsMidi();
    });
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
    await invoke('midi_set_config', { enabled, inputIds });
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
    if (!host) return;
    host.textContent = t('midi.loading');
    const devices = await invoke('midi_devices').catch(() => []);
    host.innerHTML = '';
    if (!devices.length) {
        const p = document.createElement('p');
        p.className = 'hint';
        p.textContent = t('midi.no_devices');
        host.appendChild(p);
        return;
    }
    devices.forEach(device => host.appendChild(deviceRow(device)));
    updateDeviceSelectionState();
}

function deviceRow(device) {
    const label = document.createElement('label');
    label.className = 'checkbox-line';
    const input = document.createElement('input');
    input.type = 'checkbox';
    input.dataset.midiDevice = '1';
    input.value = device.id;
    input.checked = !!device.selected;
    input.addEventListener('change', saveSettingsMidi);
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
