/** Estado común de selección por identidad estable para listas normales o virtuales. */
export function createListSelection(keyOf) {
    const selected = new Map();
    let activeKey = '';
    let anchorKey = '';

    function clear() {
        selected.clear();
        activeKey = '';
        anchorKey = '';
    }

    function click(event, item, items) {
        const key = keyOf(item);
        const index = items.findIndex(value => keyOf(value) === key);
        if (event.shiftKey && anchorKey) {
            selectRange(items, index, event.ctrlKey);
        } else if (event.ctrlKey) {
            if (selected.has(key)) selected.delete(key);
            else selected.set(key, item);
            anchorKey = key;
        } else {
            selected.clear();
            selected.set(key, item);
            anchorKey = key;
        }
        activeKey = key;
    }

    function move(items, delta, extend) {
        if (!items.length) return -1;
        let index = items.findIndex(item => keyOf(item) === activeKey);
        if (index < 0) index = delta >= 0 ? 0 : items.length - 1;
        else index = Math.max(0, Math.min(items.length - 1, index + delta));
        const item = items[index];
        if (extend) {
            if (!anchorKey) anchorKey = activeKey || keyOf(item);
            selectRange(items, index, false);
        } else {
            selected.clear();
            selected.set(keyOf(item), item);
            anchorKey = keyOf(item);
        }
        activeKey = keyOf(item);
        return index;
    }

    function forContext(item, orderedItems = []) {
        const key = keyOf(item);
        if (!selected.has(key)) {
            selected.clear();
            selected.set(key, item);
            activeKey = key;
            anchorKey = key;
        }
        const order = new Map(orderedItems.map((value, index) => [keyOf(value), index]));
        return [...selected.values()].sort((left, right) =>
            (order.get(keyOf(left)) ?? Number.MAX_SAFE_INTEGER)
            - (order.get(keyOf(right)) ?? Number.MAX_SAFE_INTEGER));
    }

    function prune(items) {
        const valid = new Set(items.map(keyOf));
        for (const key of selected.keys()) {
            if (!valid.has(key)) selected.delete(key);
        }
        if (activeKey && !valid.has(activeKey)) activeKey = '';
        if (anchorKey && !valid.has(anchorKey)) anchorKey = '';
    }

    function selectRange(items, targetIndex, additive) {
        const anchor = items.findIndex(item => keyOf(item) === anchorKey);
        if (anchor < 0) {
            selected.set(keyOf(items[targetIndex]), items[targetIndex]);
            return;
        }
        if (!additive) selected.clear();
        const [start, end] = anchor < targetIndex
            ? [anchor, targetIndex] : [targetIndex, anchor];
        items.slice(start, end + 1).forEach(item => selected.set(keyOf(item), item));
    }

    return {
        clear, click, move, forContext, prune,
        has: key => selected.has(key),
        values: () => [...selected.values()],
        active: () => activeKey,
        size: () => selected.size,
    };
}
