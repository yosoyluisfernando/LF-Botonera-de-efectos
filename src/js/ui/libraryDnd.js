/** Convierte pistas de Biblioteca en archivos nativos para reutilizar fileDrop.js. */
import { startNativeFileDrag } from '../bridge/api.js';
import { alertIpcError } from './ipcError.js';

const DRAG_THRESHOLD = 6;
let candidate = null;
let dragging = false;
let wired = false;
let iconPromise = null;

export function initLibraryDnd() {
    if (wired) return;
    wired = true;
    iconPromise = loadDragIcon();
    document.addEventListener('mousemove', move);
    document.addEventListener('mouseup', cancelCandidate);
    window.addEventListener('blur', cancelCandidate);
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
    if (!candidate || dragging || distanceFromStart(event) < DRAG_THRESHOLD) return;
    const drag = candidate;
    candidate = null;
    dragging = true;
    drag.source.classList.add('drag-source');
    event.preventDefault();
    void beginNativeDrag(drag);
}

async function beginNativeDrag(drag) {
    try {
        const icon = await iconPromise;
        await startNativeFileDrag(drag.paths, icon, clear);
    } catch (error) {
        clear();
        await alertIpcError(error);
    }
}

function distanceFromStart(event) {
    return Math.hypot(event.clientX - candidate.x, event.clientY - candidate.y);
}

function clear() {
    document.querySelectorAll('.library-row.drag-source, .library-table-row.drag-source')
        .forEach(element => element.classList.remove('drag-source'));
    candidate = null;
    dragging = false;
}

function cancelCandidate() {
    if (!dragging) candidate = null;
}

function loadDragIcon() {
    return fetch('/icono_circular.png')
        .then(response => {
            if (!response.ok) throw new Error('NATIVE_DRAG_ICON_UNAVAILABLE');
            return response.blob();
        })
        .then(blob => createImageBitmap(blob))
        .then(bitmap => {
            const canvas = document.createElement('canvas');
            canvas.width = 64;
            canvas.height = 64;
            canvas.getContext('2d').drawImage(bitmap, 0, 0, 64, 64);
            bitmap.close();
            return canvas.toDataURL('image/png');
        });
}
