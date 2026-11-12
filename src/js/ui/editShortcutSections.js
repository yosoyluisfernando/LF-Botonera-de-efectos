import { invoke, listen } from '../bridge/api.js';
import { isAndroid } from '../bridge/platform.js';
import { t } from '../util/i18n.js';
import { cancelMidiCapture, hasActiveMidiCapture } from './midiControls.js';

let wired = false;
let request = 0;
let previous = null;

export async function prepareEditShortcutSections() {
    if (!isAndroid()) return;
    if (!wired) wireSections();
    previous = null;
    await refreshSections();
}

function wireSections() {
    wired = true;
    for (const kind of ['keyboard', 'midi']) {
        const group = document.getElementById(`edit-${kind}-shortcut`);
        const details = document.createElement('details');
        details.id = `edit-${kind}-disclosure`;
        details.className = 'edit-shortcut-disclosure';
        const summary = document.createElement('summary');
        summary.dataset.i18n = `edit_modal.${kind}_shortcuts`;
        summary.textContent = t(summary.dataset.i18n);
        const hint = document.createElement('p');
        hint.id = `edit-${kind}-availability`;
        hint.className = 'hint';
        group.replaceWith(details);
        details.append(summary, group, hint);
    }
    const refresh = () => {
        if (!document.getElementById('edit-modal').classList.contains('hidden')) void refreshSections();
    };
    for (const event of ['midi-devices-changed', 'keyboard-devices-changed']) {
        listen(event, refresh).catch(console.error);
    }
    window.addEventListener('focus', refresh);
}

async function refreshSections() {
    const current = ++request;
    try {
        const state = await invoke('get_shortcut_editor_state');
        if (current !== request) return;
        applyState(state);
    } catch {
        if (current !== request) return;
        applyState({ keyboard_available: false, midi_available: false }, true);
    }
}

function applyState(state, failed = false) {
    for (const kind of ['keyboard', 'midi']) {
        const available = state[`${kind}_available`];
        const details = document.getElementById(`edit-${kind}-disclosure`);
        if (previous?.[`${kind}_available`] !== available) details.open = available;
        const hint = document.getElementById(`edit-${kind}-availability`);
        hint.textContent = available ? '' : t(failed ? 'edit_modal.shortcuts_unavailable'
            : `edit_modal.${kind}_unavailable`);
        hint.classList.toggle('hidden', available);
        const control = document.getElementById(kind === 'midi' ? 'edit-midi-capture' : 'edit-shortcut');
        control.dataset.unavailable = String(!available);
        control.disabled = !available || (kind === 'midi' && hasActiveMidiCapture());
        if (kind === 'midi' && !available) void cancelMidiCapture();
    }
    previous = state;
}
