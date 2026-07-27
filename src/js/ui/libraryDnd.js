/** Arrastre interno con ratón; evita competir con el receptor de archivos de Tauri. */
import { invoke } from '../bridge/api.js';
import { appAlert } from './appDialog.js';
import { alertIpcError } from './ipcError.js';
import { dropFileOnGrid } from './fileDrop.js';
import { t } from '../util/i18n.js';

const DRAG_THRESHOLD = 6;
let paths = [];
let candidate = null;
let dragging = false;
let onRefresh = null;
let wired = false;

export function initLibraryDnd(callback) {
    onRefresh = callback;
    if (wired) return;
    wired = true;
    document.addEventListener('mousemove', move);
    document.addEventListener('mouseup', finish);
    window.addEventListener('blur', clear);
}

export function startLibraryDrag(event, selection) {
    if (event.button !== 0 || event.ctrlKey || event.shiftKey) return;
    candidate = {
        x: event.clientX,
        y: event.clientY,
        paths: selection.map(item => item.path),
        source: event.currentTarget,
    };
}

function move(event) {
    if (!candidate) return;
    if (!dragging && distanceFromStart(event) < DRAG_THRESHOLD) return;
    if (!dragging) {
        dragging = true;
        paths = candidate.paths;
        candidate.source.classList.add('drag-source');
    }
    const target = destination(event.target);
    clearTargets();
    event.preventDefault();
    target?.classList.add('library-drag-over');
}

async function finish(event) {
    if (!candidate) return;
    const target = dragging ? destination(event.target) : null;
    const selected = [...paths];
    const wasDragging = dragging;
    clear();
    if (!wasDragging || !target) return;
    try {
        if (target.matches('.grid-item[data-index]')) {
            if (selected.length !== 1) {
                await appAlert(t('library.single_track_for_button'));
                return;
            }
            await dropFileOnGrid(target, selected[0]);
        } else {
            await invoke('library_assign_to_paleta', {
                paths: selected,
                paletaId: target.dataset.paletaId,
            });
            await onRefresh?.();
        }
    } catch (error) {
        await alertIpcError(error);
    }
}

function distanceFromStart(event) {
    return Math.hypot(event.clientX - candidate.x, event.clientY - candidate.y);
}

function destination(node) {
    return node.closest?.(
        '.grid-item[data-index], #tabs-list .tab[data-paleta-id]',
    ) ?? null;
}

function clear() {
    document.querySelectorAll('.library-drag-over, .library-row.drag-source')
        .forEach(element => element.classList.remove('library-drag-over', 'drag-source'));
    paths = [];
    candidate = null;
    dragging = false;
}

function clearTargets() {
    document.querySelectorAll('.library-drag-over')
        .forEach(element => element.classList.remove('library-drag-over'));
}
