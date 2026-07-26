/** Arrastre de resultados hacia celdas o pestañas; Rust hace la mutación real. */
import { invoke } from '../bridge/api.js';
import { appAlert, appConfirm } from './appDialog.js';
import { alertIpcError } from './ipcError.js';
import { t } from '../util/i18n.js';

let paths = [];
let onRefresh = null;
let wired = false;

export function initLibraryDnd(callback) {
    onRefresh = callback;
    if (wired) return;
    wired = true;
    document.addEventListener('dragover', markTarget);
    document.addEventListener('dragleave', clearIfOutside);
    document.addEventListener('drop', drop);
    document.addEventListener('dragend', clear);
}

export function startLibraryDrag(event, selection) {
    paths = selection.map(item => item.path);
    event.dataTransfer.effectAllowed = 'copy';
    event.dataTransfer.setData('text/plain', paths.join('\n'));
    event.currentTarget.classList.add('drag-source');
}

function markTarget(event) {
    if (!paths.length) return;
    const target = destination(event.target);
    clearTargets();
    if (!target) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = 'copy';
    target.classList.add('library-drag-over');
}

async function drop(event) {
    if (!paths.length) return;
    const target = destination(event.target);
    if (!target) return clear();
    event.preventDefault();
    const selected = [...paths];
    clear();
    try {
        if (target.matches('.grid-item[data-index]')) {
            await dropOnCell(target, selected);
        } else {
            await invoke('library_assign_to_paleta', {
                paths: selected,
                paletaId: target.dataset.paletaId,
            });
        }
        await onRefresh?.();
    } catch (error) {
        await alertIpcError(error);
    }
}

async function dropOnCell(cell, selected) {
    if (selected.length !== 1) {
        await appAlert(t('library.single_track_for_button'));
        return;
    }
    if (cell.dataset.id && !await appConfirm(t('app.button_has_content'), {
        ok: t('app.replace'),
        cancel: t('app.dont_add'),
    }, { ok: 1, cancel: 2 })) return;
    await invoke('assign_file_to_button', {
        index: Number(cell.dataset.index),
        path: selected[0],
    });
}

function destination(node) {
    return node.closest?.(
        '.grid-item[data-index], #tabs-list .tab[data-paleta-id]',
    ) ?? null;
}

function clearIfOutside(event) {
    if (!event.relatedTarget) clear();
}

function clear() {
    document.querySelectorAll('.library-drag-over, .library-row.drag-source')
        .forEach(element => element.classList.remove('library-drag-over', 'drag-source'));
    paths = [];
}

function clearTargets() {
    document.querySelectorAll('.library-drag-over')
        .forEach(element => element.classList.remove('library-drag-over'));
}
