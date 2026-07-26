/** Selector accesible del encabezado común del panel fijo. */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

const ORDER = ['buttons', 'player', 'search'];
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
    const button = document.getElementById('fixed-view-switch');
    button.addEventListener('click', cycle);
    observer = new ResizeObserver(paintLabel);
    observer.observe(document.querySelector('.fixed-panel-header'));
}

export function refreshFixedPanelView(settings) {
    current = settings.view;
    paintLabel();
}

async function cycle() {
    const state = await invoke('get_fixed_panel');
    const settings = state.settings;
    const position = ORDER.indexOf(settings.view);
    const view = ORDER[(position + 1) % ORDER.length];
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

function paintLabel() {
    const button = document.getElementById('fixed-view-switch');
    const label = document.getElementById('fixed-view-label');
    if (!button || !label) return;
    const keys = LABELS[current] ?? LABELS.buttons;
    label.textContent = t(keys[0]);
    label.title = label.textContent;
    requestAnimationFrame(() => {
        if (label.scrollWidth > button.clientWidth) label.textContent = t(keys[1]);
    });
}
