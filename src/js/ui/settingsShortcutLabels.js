import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { isAndroid } from '../bridge/platform.js';

export function initShortcutLabels(config) {
    const section = document.createElement('section');
    section.className = 'shortcut-label-settings';
    const title = document.createElement('p');
    title.className = 'fade-section-title';
    title.dataset.i18n = 'settings.shortcut_labels_title';
    title.textContent = t(title.dataset.i18n);
    section.appendChild(title);
    for (const kind of ['keyboard', 'midi']) {
        const label = document.createElement('label');
        label.className = 'checkbox-line';
        const input = document.createElement('input');
        input.id = `config-show-${kind}-shortcut-labels`;
        input.type = 'checkbox';
        const text = document.createElement('span');
        text.dataset.i18n = `settings.show_${kind}_shortcut_labels`;
        text.textContent = t(text.dataset.i18n);
        label.append(input, text);
        section.appendChild(label);
    }
    if (isAndroid()) document.getElementById('settings-midi-section').appendChild(section);
    else {
        const panel = document.getElementById('s-atajos');
        const mapping = document.getElementById('btn-enter-mapping').parentElement;
        panel.prepend(mapping);
        mapping.after(section);
    }
    loadShortcutLabels(config);
}

export function loadShortcutLabels(config) {
    for (const kind of ['keyboard', 'midi']) {
        document.getElementById(`config-show-${kind}-shortcut-labels`).checked =
            config[`show_${kind}_shortcut_labels`] !== false;
    }
    applyShortcutLabels(config);
}

export async function saveShortcutLabels() {
    const keyboard = document.getElementById('config-show-keyboard-shortcut-labels').checked;
    const midi = document.getElementById('config-show-midi-shortcut-labels').checked;
    await invoke('set_shortcut_label_visibility', { keyboard, midi });
    applyShortcutLabels({ show_keyboard_shortcut_labels: keyboard, show_midi_shortcut_labels: midi });
}

function applyShortcutLabels(config) {
    document.body.dataset.keyboardShortcutLabels = String(config.show_keyboard_shortcut_labels !== false);
    document.body.dataset.midiShortcutLabels = String(config.show_midi_shortcut_labels !== false);
}
