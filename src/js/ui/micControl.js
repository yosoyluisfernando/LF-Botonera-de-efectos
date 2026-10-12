/** Interacciones de Mic; el estado y las transiciones proceden de Rust. */
import { invoke, listen } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { openMicSettings } from './micSettings.js';
import '../../css/mic.css';

let wired = false;
let pointer = null;
let keyboard = false;
let commands = Promise.resolve();

export function initMic() {
    if (wired) return;
    wired = true;
    const button = document.getElementById('mic-button');
    const send = pressed => {
        commands = commands.then(() => invoke('mic_pointer', { pressed })).catch(console.error);
    };
    button.addEventListener('pointerdown', e => {
        if (e.button !== 0 || pointer !== null) return;
        e.preventDefault();
        pointer = e.pointerId;
        button.setPointerCapture(e.pointerId);
        send(true);
    });
    const release = e => {
        if (pointer !== e.pointerId) return;
        pointer = null;
        send(false);
    };
    ['pointerup', 'pointercancel', 'lostpointercapture'].forEach(event => button.addEventListener(event, release));
    button.addEventListener('keydown', e => {
        if (![' ', 'Enter'].includes(e.key)) return;
        e.preventDefault(); e.stopPropagation();
        if (!keyboard) { keyboard = true; send(true); }
    });
    button.addEventListener('keyup', e => {
        if (![' ', 'Enter'].includes(e.key)) return;
        e.preventDefault(); e.stopPropagation();
        keyboard = false; send(false);
    });
    button.addEventListener('blur', () => {
        if (keyboard) { keyboard = false; send(false); }
    });
    window.addEventListener('blur', () => {
        pointer = null; keyboard = false;
        commands = commands.then(() => invoke('mic_release_local')).catch(console.error);
    });
    button.addEventListener('contextmenu', e => {
        e.preventDefault(); openMicSettings();
    });
    listen('mic-tick', e => paintMic(e.payload)).catch(console.error);
    invoke('get_mic').then(([, state]) => paintMic(state)).catch(console.error);
}

function paintMic(state) {
    const button = document.getElementById('mic-button');
    const status = state.error ? t(state.error) : t('mic.' + (state.active ? 'active' : state.listening ? 'listening' : 'idle'));
    button.classList.toggle('mic-active', state.active);
    button.classList.toggle('mic-listening', state.listening && !state.active);
    button.classList.toggle('mic-error', !!state.error);
    button.setAttribute('aria-pressed', String(state.mode === 'auto' ? state.listening : state.active));
    button.setAttribute('aria-label', `${t('mic.label')}: ${status}`);
    button.title = `${status}. ${t('mic.configure_hint')}`;
    const meter = document.getElementById('mic-level-meter');
    if (meter) meter.value = state.preview_level_db;
    const label = document.getElementById('mic-live-status');
    if (label) label.textContent = state.preview_error ? t(state.preview_error) : status;
}
