/** Adaptador de la selección común para pistas con id estable. */
import { createListSelection } from './listSelection.js';

const selection = createListSelection(track => track.id);
let wired = false;

export function initPlayerSelection() {
    if (wired) return;
    wired = true;
    document.addEventListener('keydown', event => {
        if (event.key === 'Escape' && selection.size()) clearPlayerSelection();
    });
    document.addEventListener('click', event => {
        if (!selection.size() || event.target.closest('#player-rows, #context-menu')) return;
        clearPlayerSelection();
    }, true);
}

export function handlePlayerSelectionClick(event, track, tracks) {
    selection.click(event, track, tracks);
    paintPlayerSelection();
}

export function movePlayerSelection(tracks, delta, extend) {
    const index = selection.move(tracks, delta, extend);
    paintPlayerSelection();
    return index;
}

export function selectionForContext(track, tracks) {
    const chosen = selection.forContext(track, tracks);
    paintPlayerSelection();
    const selectedIds = new Set(chosen.map(item => item.id));
    const indexes = tracks
        .map((item, index) => selectedIds.has(item.id) ? index : -1)
        .filter(index => index >= 0);
    return { count: indexes.length, indexes };
}

export function isPlayerSelected(id) {
    return selection.has(id);
}

export function activePlayerId() {
    return selection.active();
}

export function clearPlayerSelection() {
    selection.clear();
    paintPlayerSelection();
}

export function prunePlayerSelection(tracks) {
    selection.prune(tracks);
}

function paintPlayerSelection() {
    document.querySelectorAll('#player-rows .player-row').forEach(row => {
        const selected = selection.has(row.dataset.trackId);
        row.classList.toggle('queue-selected', selected);
        row.setAttribute('aria-selected', String(selected));
    });
}
