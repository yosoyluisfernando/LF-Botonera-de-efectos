/**
 * Selección múltiple de filas del reproductor.
 * Ctrl alterna filas; Shift selecciona un intervalo desde el último ancla.
 * La identidad se guarda por id estable, nunca por posición mutable.
 */

let _selected = new Set();
let _anchorId = null;
let _wired = false;

export function initPlayerSelection() {
    if (_wired) return;
    _wired = true;
    document.addEventListener('keydown', e => {
        if (e.key === 'Escape' && _selected.size) clearPlayerSelection();
    });
    document.addEventListener('click', e => {
        if (!_selected.size || e.target.closest('#player-rows, #context-menu')) return;
        clearPlayerSelection();
    }, true);
}

export function handlePlayerSelectionClick(e, track, tracks) {
    if (!e.ctrlKey && !e.shiftKey) {
        if (_selected.size) clearPlayerSelection();
        return;
    }
    e.preventDefault();
    e.stopPropagation();
    if (e.shiftKey) {
        _selectRange(track.id, tracks, e.ctrlKey);
    } else {
        _toggle(track.id);
        _anchorId = track.id;
    }
    paintPlayerSelection();
}

/** El clic derecho conserva el grupo si la fila ya pertenece a él. */
export function selectionForContext(track, tracks) {
    if (!_selected.has(track.id)) {
        _selected.clear();
        _selected.add(track.id);
        _anchorId = track.id;
        paintPlayerSelection();
    }
    const indexes = tracks
        .map((item, index) => _selected.has(item.id) ? index : -1)
        .filter(index => index >= 0);
    return { count: indexes.length, indexes };
}

export function isPlayerSelected(id) {
    return _selected.has(id);
}

export function clearPlayerSelection() {
    _selected.clear();
    _anchorId = null;
    paintPlayerSelection();
}

export function prunePlayerSelection(tracks) {
    const valid = new Set(tracks.map(track => track.id));
    _selected = new Set([..._selected].filter(id => valid.has(id)));
    if (_anchorId && !valid.has(_anchorId)) _anchorId = null;
}

function _toggle(id) {
    if (_selected.has(id)) _selected.delete(id);
    else _selected.add(id);
}

function _selectRange(targetId, tracks, additive) {
    const target = tracks.findIndex(track => track.id === targetId);
    let anchor = tracks.findIndex(track => track.id === _anchorId);
    if (anchor < 0) anchor = target;
    if (!additive) _selected.clear();
    const [start, end] = anchor <= target ? [anchor, target] : [target, anchor];
    for (let index = start; index <= end; index += 1) {
        _selected.add(tracks[index].id);
    }
    if (!_anchorId) _anchorId = targetId;
}

function paintPlayerSelection() {
    document.querySelectorAll('#player-rows .player-row').forEach(row => {
        const selected = _selected.has(row.dataset.trackId);
        row.classList.toggle('queue-selected', selected);
        row.setAttribute('aria-selected', String(selected));
    });
}
