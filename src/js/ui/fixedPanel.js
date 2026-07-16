import { invoke } from '../bridge/api.js';
import { paintAdaptive } from '../util/colorAdapter.js';
import { typeIcon } from '../util/typeIcons.js';
import { t } from '../util/i18n.js';
import { openEditModal } from './editModal.js';
import { showContextMenu } from './contextMenu.js';
import { paintPlayback } from './playbackPainter.js';
import { initFixedPlaybackModes, refreshFixedPlaybackModes } from './fixedPlaybackModes.js';
import { initFixedPanelResize } from './fixedPanelResize.js';
import { initPlayerView, drawPlayerView } from './playerView.js';

let _refresh = null;
let _buttons = {};

export function initFixedPanel(state, onRefresh) {
    _refresh = onRefresh;
    document.getElementById('btn-fixed-panel').addEventListener('click', toggleFixedPanel);
    initFixedPlaybackModes();
    initFixedPanelResize();
    initPlayerView();
    document.getElementById('fixed-panel-add').addEventListener('click', async () => {
        const current = await invoke('get_fixed_panel');
        const next = Math.max(0, ...current.buttons.map(b => b.index)) + 1;
        openEditModal(next, null, _refresh, 'fixed').catch(console.error);
    });
    drawFixedPanel(state);
}

export async function initialFixedPanel() {
    const current = await invoke('get_fixed_panel');
    const s = current.settings;
    if (s.visible === s.show_on_start) return current;
    return invoke('set_fixed_panel_settings', {
        scope: s.scope, view: s.view, side: s.side, visible: s.show_on_start,
        showOnStart: s.show_on_start, columns: s.columns, rowMode: s.row_mode,
        rows: s.rows, width: s.width, modesPosition: s.modes_position,
    });
}

export function drawFixedPanel(state) {
    const shell = document.getElementById('workspace-shell');
    const panel = document.getElementById('fixed-panel');
    const list = document.getElementById('fixed-panel-items');
    const settings = state.settings;
    shell.dataset.fixedSide = settings.side;
    panel.classList.toggle('hidden', !settings.visible);
    panel.dataset.view = settings.view;
    panel.dataset.modes = settings.modes_position;
    panel.style.width = `${settings.width}px`;
    list.style.gridTemplateColumns = settings.view === 'buttons'
        ? `repeat(${settings.columns}, minmax(0, 1fr))` : '1fr';
    list.innerHTML = '';
    _buttons = Object.fromEntries(state.buttons.map(button => [button.id, button]));
    state.buttons.sort((a, b) => a.index - b.index).forEach(btn => list.appendChild(_item(btn)));
    document.getElementById('btn-fixed-panel').classList.toggle('active', settings.visible);
    refreshFixedPlaybackModes();
    // El modo reproductor tiene cola propia: se dibuja aparte de los botones fijos.
    if (settings.view === 'player') drawPlayerView().catch(console.error);
}

export function paintFixedAudioTick(payload) {
    paintPlayback('.fixed-panel-item[data-id]', _buttons,
        (payload.buttons ?? []).filter(tick => tick.group === 'fixed'));
}

export async function toggleFixedPanel() {
    const current = await invoke('get_fixed_panel');
    const s = current.settings;
    const next = await invoke('set_fixed_panel_settings', {
        scope: s.scope, view: s.view, side: s.side, visible: !s.visible,
        showOnStart: s.show_on_start, columns: s.columns, rowMode: s.row_mode,
        rows: s.rows, width: s.width, modesPosition: s.modes_position,
    });
    drawFixedPanel(next);
}

function _item(btn) {
    const el = document.createElement('button');
    el.className = 'fixed-panel-item'; el.dataset.id = btn.id; el.dataset.index = btn.index;
    paintAdaptive(el, btn.color_bg, btn.color_text, 'button');
    el.innerHTML = `<span class="fixed-panel-name">${typeIcon(btn.type_icon)}${btn.name || btn.label}</span>
        <span class="timer fixed-panel-time">${btn.duration_str || ''}</span>
        <span class="progress-container"><span class="progress-bar"></span></span>`;
    el.addEventListener('click', e => {
        if (e.altKey) return; // Alt+clic = reordenar/mover (gridDnd.js)
        invoke('play_button', { id: btn.id }).catch(console.error);
    });
    el.addEventListener('contextmenu', e => {
        e.preventDefault();
        showContextMenu(e.clientX, e.clientY, btn.index, btn, _refresh, 'fixed');
    });
    el.title = t('fixed_panel.edit_hint');
    return el;
}
