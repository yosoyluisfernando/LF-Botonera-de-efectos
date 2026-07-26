/** Selección estable por ruta para listas virtuales. */
const selected = new Map();
let activePath = '';
let anchorPath = '';

export function clearLibrarySelection() {
    selected.clear();
    activePath = '';
    anchorPath = '';
}

export function selectLibraryClick(event, item, items) {
    const index = items.findIndex(value => value.path === item.path);
    if (event.shiftKey && anchorPath) {
        selectRange(items, index, event.ctrlKey);
    } else if (event.ctrlKey) {
        if (selected.has(item.path)) selected.delete(item.path);
        else selected.set(item.path, item);
        anchorPath = item.path;
    } else {
        selected.clear();
        selected.set(item.path, item);
        anchorPath = item.path;
    }
    activePath = item.path;
}

export function moveLibrarySelection(items, delta, extend) {
    if (!items.length) return -1;
    let index = items.findIndex(item => item.path === activePath);
    if (index < 0) index = delta >= 0 ? 0 : items.length - 1;
    else index = Math.max(0, Math.min(items.length - 1, index + delta));
    const item = items[index];
    if (extend) {
        if (!anchorPath) anchorPath = activePath || item.path;
        selectRange(items, index, false);
    } else {
        selected.clear();
        selected.set(item.path, item);
        anchorPath = item.path;
    }
    activePath = item.path;
    return index;
}

export function selectionForLibraryContext(item, visibleItems = []) {
    if (!selected.has(item.path)) {
        selected.clear();
        selected.set(item.path, item);
        activePath = item.path;
        anchorPath = item.path;
    }
    const order = new Map(visibleItems.map((value, index) => [value.path, index]));
    return [...selected.values()].sort((left, right) =>
        (order.get(left.path) ?? Number.MAX_SAFE_INTEGER)
        - (order.get(right.path) ?? Number.MAX_SAFE_INTEGER));
}

export function librarySelection() {
    return [...selected.values()];
}

export function isLibrarySelected(path) {
    return selected.has(path);
}

export function activeLibraryPath() {
    return activePath;
}

function selectRange(items, targetIndex, additive) {
    const anchor = items.findIndex(item => item.path === anchorPath);
    if (anchor < 0) {
        selected.set(items[targetIndex].path, items[targetIndex]);
        return;
    }
    if (!additive) selected.clear();
    const [start, end] = anchor < targetIndex
        ? [anchor, targetIndex] : [targetIndex, anchor];
    items.slice(start, end + 1).forEach(item => selected.set(item.path, item));
}
