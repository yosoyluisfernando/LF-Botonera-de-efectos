/** Selector persistente del contenido principal de cada fila del buscador. */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

const MODES = new Set(['filename', 'metadata']);
const LABELS = {
    filename: ['library.display_filename', 'library.display_filename_short'],
    metadata: ['library.display_metadata', 'library.display_metadata_short'],
};

let current = 'metadata';
let onChange = null;

export function initLibraryDisplayMode(callback) {
    onChange = callback;
    const selector = document.querySelector('.library-display-selector');
    const button = document.getElementById('library-display-switch');
    const menu = document.getElementById('library-display-menu');
    button.addEventListener('click', () => toggle(menu.classList.contains('hidden')));
    menu.querySelectorAll('[data-library-display]').forEach(item => {
        item.addEventListener('click', () => select(item.dataset.libraryDisplay));
    });
    menu.addEventListener('keydown', navigate);
    document.addEventListener('pointerdown', event => {
        if (!selector.contains(event.target)) close();
    });
}

export function refreshLibraryDisplayMode(settings) {
    current = MODES.has(settings?.library_display) ? settings.library_display : 'metadata';
    const keys = LABELS[current];
    const label = document.getElementById('library-display-label');
    const button = document.getElementById('library-display-switch');
    label.textContent = t(keys[1]);
    button.setAttribute('aria-label', `${t('library.display_mode')}: ${t(keys[0])}`);
    button.title = button.getAttribute('aria-label');
    document.querySelectorAll('#library-display-menu [data-library-display]').forEach(item => {
        item.setAttribute('aria-checked', String(item.dataset.libraryDisplay === current));
    });
}

export function libraryDisplayMode() {
    return current;
}

async function select(mode) {
    close();
    if (!MODES.has(mode) || mode === current) return;
    const state = await invoke('set_library_display_mode', { mode });
    refreshLibraryDisplayMode(state.settings);
    onChange?.(state);
}

function toggle(open) {
    const menu = document.getElementById('library-display-menu');
    const button = document.getElementById('library-display-switch');
    menu.classList.toggle('hidden', !open);
    button.setAttribute('aria-expanded', String(open));
    if (open) {
        (menu.querySelector(`[data-library-display="${current}"]`)
            ?? menu.querySelector('button'))?.focus();
    }
}

function close() {
    toggle(false);
}

function navigate(event) {
    if (event.key === 'Escape') {
        event.preventDefault();
        close();
        return document.getElementById('library-display-switch').focus();
    }
    const items = [...event.currentTarget.querySelectorAll('button')];
    const index = items.indexOf(document.activeElement);
    const next = event.key === 'ArrowDown' ? index + 1
        : event.key === 'ArrowUp' ? index - 1
            : event.key === 'Home' ? 0
                : event.key === 'End' ? items.length - 1 : null;
    if (next === null) return;
    event.preventDefault();
    items[(next + items.length) % items.length].focus();
}
