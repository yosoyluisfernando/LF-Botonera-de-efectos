import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { appAlert } from './appDialog.js';

const EMPTY = {
    device_id: '',
    device_name: '',
    message: '',
    channel: 0,
    number: 0,
};
let activeCapture = null;
let cancellationWired = false;

export function initMidiField(inputId, captureId, clearId) {
    wireCancellation();
    const capture = document.getElementById(captureId);
    if (capture) capture.onclick = () => captureInto(inputId, captureId);
    const clear = document.getElementById(clearId);
    if (clear) clear.onclick = async () => {
        await cancelMidiCapture();
        setMidiField(inputId, null);
        document.getElementById(inputId)?.focus();
    };
}

export async function cancelMidiCapture() {
    if (!activeCapture) return;
    try {
        await invoke('midi_capture_cancel');
    } catch (error) {
        console.error('Error al cancelar captura MIDI:', error);
    }
}

export function setMidiField(inputId, binding) {
    const input = document.getElementById(inputId);
    if (!input) return;
    const value = normalize(binding);
    input.dataset.midiBinding = JSON.stringify(value);
    input.value = display(value);
}

export function getMidiField(inputId) {
    const input = document.getElementById(inputId);
    if (!input?.dataset.midiBinding) return { ...EMPTY };
    try {
        return normalize(JSON.parse(input.dataset.midiBinding));
    } catch (_) {
        return { ...EMPTY };
    }
}

export function display(binding) {
    const value = normalize(binding);
    if (!value.device_id || !value.message) return '';
    return `${value.device_name} ch ${value.channel + 1} ${label(value.message)} ${value.number}`;
}

async function captureInto(inputId, buttonId) {
    const button = document.getElementById(buttonId);
    if (activeCapture) return;
    activeCapture = { inputId, buttonId };
    button.disabled = true;
    button.textContent = t('midi.waiting');
    try {
        const result = await invoke('midi_capture_next');
        setMidiField(inputId, result.binding);
    } catch (error) {
        if (String(error) === 'midi_capture_cancelled') return;
        await appAlert(t(`midi.${error}`) === `midi.${error}` ? String(error) : t(`midi.${error}`));
    } finally {
        activeCapture = null;
        button.disabled = false;
        button.textContent = t('midi.capture');
    }
}

function wireCancellation() {
    if (cancellationWired) return;
    cancellationWired = true;
    document.addEventListener('keydown', event => {
        if (event.key === 'Escape') cancelMidiCapture();
    }, true);
    document.addEventListener('click', event => {
        if (event.target.closest('[data-midi-cancel]')) cancelMidiCapture();
    }, true);
}

function normalize(binding) {
    if (!binding) return { ...EMPTY };
    return {
        device_id: binding.device_id ?? binding.deviceId ?? '',
        device_name: binding.device_name ?? binding.deviceName ?? '',
        message: binding.message ?? '',
        channel: Number(binding.channel ?? 0),
        number: Number(binding.number ?? 0),
    };
}

function label(message) {
    return ({ note: t('midi.note'), cc: 'CC', program: t('midi.program') })[message] || 'MIDI';
}
