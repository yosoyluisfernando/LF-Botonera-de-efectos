/** Adaptador de la selección común para resultados identificados por ruta. */
import { createListSelection } from './listSelection.js';

const selection = createListSelection(item => item.path);

export function clearLibrarySelection() {
    selection.clear();
}

export function selectLibraryClick(event, item, items) {
    selection.click(event, item, items);
}

export function moveLibrarySelection(items, delta, extend) {
    return selection.move(items, delta, extend);
}

export function selectionForLibraryContext(item, visibleItems = []) {
    return selection.forContext(item, visibleItems);
}

export function librarySelection() {
    return selection.values();
}

export function isLibrarySelected(path) {
    return selection.has(path);
}

export function activeLibraryPath() {
    return selection.active();
}
