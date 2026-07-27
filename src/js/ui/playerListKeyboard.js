/** Teclado de la cola: reproduce la misma selección que el buscador. */
import {
    activePlayerId, clearPlayerSelection, movePlayerSelection,
} from './playerSelection.js';

let tracks = () => [];
let openContext = null;

export function initPlayerListKeyboard(getTracks, onContext) {
    tracks = getTracks;
    openContext = onContext;
    document.getElementById('player-rows').addEventListener('keydown', navigate);
}

function navigate(event) {
    const items = tracks();
    if (event.key === 'Escape') {
        clearPlayerSelection();
        return;
    }
    if (event.key === 'ContextMenu' || (event.shiftKey && event.key === 'F10')) {
        event.preventDefault();
        return openKeyboardContext(items);
    }
    const rows = event.currentTarget;
    const page = Math.max(1, Math.floor(rows.clientHeight / rowHeight(rows)));
    const delta = event.key === 'ArrowDown' ? 1
        : event.key === 'ArrowUp' ? -1
            : event.key === 'PageDown' ? page
                : event.key === 'PageUp' ? -page : 0;
    if (!delta) return;
    event.preventDefault();
    const index = movePlayerSelection(items, delta, event.shiftKey);
    if (index >= 0) focusRow(rows, index);
}

function openKeyboardContext(items) {
    const item = items.find(value => value.id === activePlayerId()) ?? items[0];
    if (!item) return;
    const row = [...document.querySelectorAll('#player-rows .player-row')]
        .find(value => value.dataset.trackId === item.id);
    const rect = row?.getBoundingClientRect()
        ?? document.getElementById('player-rows').getBoundingClientRect();
    openContext?.({ preventDefault() {}, clientX: rect.left + 12, clientY: rect.top + 12 }, item);
}

function focusRow(rows, index) {
    const row = rows.querySelector(`[data-index="${index}"]`);
    row?.scrollIntoView({ block: 'nearest' });
    if (row) rows.setAttribute('aria-activedescendant', row.id);
}

function rowHeight(rows) {
    return rows.querySelector('.player-row')?.getBoundingClientRect().height || 38;
}
