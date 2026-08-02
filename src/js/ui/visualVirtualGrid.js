/** Cuadrícula virtual con separadores de categoría y un DOM máximo de 300 iconos. */
import { buildVisualGroupLayouts } from './visualGridLayout.js';

const WINDOW_SIZE = 300;
const ITEM_HEIGHT = 67;
const HEADER_HEIGHT = 38;
const ITEM_MIN_WIDTH = 64;
const GAP = 5;
const HORIZONTAL_SPACE = 24;
const EDGE_BUFFER = 100;

export function createVisualVirtualGrid(
    container, createItem, load, onRange, onCategory,
) {
    const space = document.createElement('div');
    const windowElement = document.createElement('div');
    space.className = 'visual-results-space';
    windowElement.className = 'visual-results-window';
    space.appendChild(windowElement);
    container.replaceChildren(space);

    let items = [];
    let groups = [];
    let layouts = [];
    let total = 0;
    let start = 0;
    let columns = columnCount();
    let generation = 0;
    let loading = false;
    let queued = null;
    let frame = 0;

    container.addEventListener('scroll', schedule);
    new ResizeObserver(resize).observe(container);

    async function reset(nextGroups = []) {
        generation += 1;
        items = [];
        groups = nextGroups;
        total = groups.reduce((sum, group) => sum + group.count, 0);
        start = 0;
        container.scrollTop = 0;
        layout();
        await fetchWindow(0, generation);
    }

    function schedule() {
        if (frame) return;
        frame = requestAnimationFrame(() => {
            frame = 0;
            ensureWindow().catch(console.error);
        });
    }

    function resize() {
        if (!container.clientWidth || !container.clientHeight) return;
        const next = columnCount();
        if (next === columns) return;
        const anchor = indexAt(container.scrollTop);
        columns = next;
        layout();
        container.scrollTop = yFor(anchor);
        if (total) fetchCentered(anchor);
    }

    async function ensureWindow() {
        if (!total) return;
        const target = indexAt(container.scrollTop);
        announce(target);
        const count = items.length;
        const buffer = Math.min(EDGE_BUFFER, Math.floor(count / 3));
        const safeStart = start > 0 ? start + buffer : start;
        const safeEnd = start + count < total ? start + count - buffer : start + count;
        if (target >= safeStart && target < safeEnd) return;
        await fetchCentered(target);
    }

    function fetchCentered(target) {
        const desired = Math.max(0, target - Math.floor(WINDOW_SIZE / 2));
        return fetchWindow(alignToRow(desired), generation);
    }

    async function fetchWindow(offset, ownGeneration) {
        if (loading) {
            queued = { offset, generation: ownGeneration };
            return;
        }
        loading = true;
        container.dataset.loading = 'true';
        container.setAttribute('aria-busy', 'true');
        try {
            const page = await load(offset, WINDOW_SIZE);
            if (ownGeneration !== generation) return;
            total = page.total;
            start = page.offset;
            items = page.items;
            layout();
            onRange(start, items.length, total);
            announce(indexAt(container.scrollTop));
        } finally {
            loading = false;
            delete container.dataset.loading;
            container.removeAttribute('aria-busy');
            if (queued) {
                const next = queued;
                queued = null;
                fetchWindow(next.offset, next.generation).catch(console.error);
            }
        }
    }

    function layout() {
        layouts = buildVisualGroupLayouts(
            groups, columns, HEADER_HEIGHT, ITEM_HEIGHT);
        const height = layouts.length
            ? layouts.at(-1).endY
            : Math.ceil(total / columns) * ITEM_HEIGHT;
        space.style.height = `${height}px`;
        render();
    }

    function render() {
        const headers = layouts.map(groupHeader);
        const buttons = items.map((item, offset) => {
            const button = createItem(item);
            place(button, start + offset);
            return button;
        });
        windowElement.replaceChildren(...headers, ...buttons);
    }

    function place(button, index) {
        const position = positionFor(index);
        const widthAdjustment = GAP * (columns - 1) / columns;
        const leftAdjustment = GAP * position.column
            - GAP * (columns - 1) * position.column / columns;
        button.style.top = `${position.y}px`;
        button.style.width = `calc(${100 / columns}% - ${widthAdjustment}px)`;
        button.style.left =
            `calc(${100 * position.column / columns}% + ${leftAdjustment}px)`;
    }

    function positionFor(index) {
        const group = layoutFor(index);
        const local = group ? index - group.start : index;
        return {
            y: (group?.itemsY ?? 0) + Math.floor(local / columns) * ITEM_HEIGHT,
            column: local % columns,
        };
    }

    function indexAt(y) {
        if (!total) return 0;
        if (!layouts.length) {
            return Math.min(total - 1, Math.floor(y / ITEM_HEIGHT) * columns);
        }
        const group = layouts.find(value => y < value.endY) ?? layouts.at(-1);
        if (y <= group.itemsY) return group.start;
        const row = Math.floor((y - group.itemsY) / ITEM_HEIGHT);
        return Math.min(group.end - 1, group.start + row * columns);
    }

    function yFor(index) {
        return positionFor(Math.min(index, Math.max(0, total - 1))).y;
    }

    function layoutFor(index) {
        return layouts.find(group => index >= group.start && index < group.end);
    }

    function announce(index) {
        onCategory(layoutFor(index)?.label ?? '');
    }

    function groupHeader(group) {
        const header = document.createElement('div');
        header.className = 'visual-group-header';
        header.textContent = group.label;
        header.style.top = `${group.headerY}px`;
        return header;
    }

    function columnCount() {
        const width = Math.max(ITEM_MIN_WIDTH, container.clientWidth - HORIZONTAL_SPACE);
        return Math.max(1, Math.floor((width + GAP) / (ITEM_MIN_WIDTH + GAP)));
    }

    function alignToRow(offset) {
        const group = layoutFor(offset);
        const local = group ? offset - group.start : offset;
        return Math.max(0, offset - (local % columns));
    }

    return { reset };
}
