import { invoke } from '../bridge/api.js';

let _wired = false;

export function initFixedPanelResize() {
    if (_wired) return;
    _wired = true;
    const panel = document.getElementById('fixed-panel');
    const handle = document.getElementById('fixed-panel-resize');
    let startX = 0; let startWidth = 0; let side = 'right';
    handle.addEventListener('pointerdown', event => {
        startX = event.clientX; startWidth = panel.getBoundingClientRect().width;
        side = document.getElementById('workspace-shell').dataset.fixedSide;
        handle.setPointerCapture(event.pointerId);
    });
    handle.addEventListener('pointermove', event => {
        if (!handle.hasPointerCapture(event.pointerId)) return;
        const delta = (event.clientX - startX) * (side === 'right' ? -1 : 1);
        panel.style.width = `${Math.min(600, Math.max(180, startWidth + delta))}px`;
    });
    handle.addEventListener('pointerup', async event => {
        if (!handle.hasPointerCapture(event.pointerId)) return;
        handle.releasePointerCapture(event.pointerId);
        const current = await invoke('get_fixed_panel'); const s = current.settings;
        await invoke('set_fixed_panel_settings', {
            scope: s.scope, view: s.view, side: s.side, visible: s.visible,
            showOnStart: s.show_on_start, columns: s.columns, rowMode: s.row_mode,
            rows: s.rows, width: Math.round(panel.getBoundingClientRect().width),
            modesPosition: s.modes_position,
        });
    });
}
