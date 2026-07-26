/** Menú accesible para elegir directamente la vista del panel fijo. */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

const LABELS = {
    buttons: ['fixed_panel.title', 'fixed_panel.title_compact'],
    player: ['player.title', 'player.title_compact'],
    search: ['library.search_title', 'library.search_title_compact'],
};

let current = 'buttons';
let onChange = null;
let observer = null;

export function initFixedPanelViews(callback) {
    onChange = callback;
    const selector = document.querySelector('.fixed-view-selector');
    const button = document.getElementById('fixed-view-switch');
    const menu = document.getElementById('fixed-view-menu');
    button.addEventListener('click', () => toggleMenu(menu.classList.contains('hidden')));
    menu.querySelectorAll('[data-fixed-view]').forEach(item => {
        item.addEventListener('click', () => selectView(item.dataset.fixedView));
    });
    menu.addEventListener('keydown', navigateMenu);
    document.addEventListener('pointerdown', event => {
        if (!selector.contains(event.target)) closeMenu();
    });
    observer = new ResizeObserver(paintLabel);
    observer.observe(document.querySelector('.fixed-panel-header'));
}

export function refreshFixedPanelView(settings) {
    current = settings.view;
    paintLabel();
    document.querySelectorAll('#fixed-view-menu [data-fixed-view]').forEach(item => {
        item.setAttribute('aria-checked', String(item.dataset.fixedView === current));
    });
}

async function selectView(view) {
    closeMenu();
    if (view === current) return;
    const state = await invoke('get_fixed_panel');
    const settings = state.settings;
    const next = await invoke('set_fixed_panel_settings', {
        scope: settings.scope,
        view,
        side: settings.side,
        visible: settings.visible,
        showOnStart: settings.show_on_start,
        columns: settings.columns,
        rowMode: settings.row_mode,
        rows: settings.rows,
        width: settings.width,
        modesPosition: settings.modes_position,
    });
    onChange?.(next);
}

function toggleMenu(open) {
    const menu = document.getElementById('fixed-view-menu');
    const button = document.getElementById('fixed-view-switch');
    menu.classList.toggle('hidden', !open);
    button.setAttribute('aria-expanded', String(open));
    if (open) {
        const selected = menu.querySelector(`[data-fixed-view="${current}"]`);
        (selected ?? menu.querySelector('button'))?.focus();
    }
}

function closeMenu() {
    toggleMenu(false);
}

function navigateMenu(event) {
    if (event.key === 'Escape') {
        event.preventDefault();
        closeMenu();
        return document.getElementById('fixed-view-switch').focus();
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

function paintLabel() {
    const button = document.getElementById('fixed-view-switch');
    const label = document.getElementById('fixed-view-label');
    if (!button || !label) return;
    const keys = LABELS[current] ?? LABELS.buttons;
    label.textContent = t(keys[0]);
    label.title = label.textContent;
    requestAnimationFrame(() => {
        if (button.scrollWidth > button.clientWidth) label.textContent = t(keys[1]);
    });
}
