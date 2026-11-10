/** Renderiza únicamente las filas visibles y un colchón de 50 por lado. */

const OVERSCAN = 50;

export function createLibraryVirtualList(container, createRow) {
    const rowHeight = () => parseFloat(getComputedStyle(container)
        .getPropertyValue("--library-row-height")) || 40;
    let items = [];
    let offset = 0;
    let total = 0;

    function setItems(next, metrics = {}) {
        items = next;
        offset = Math.max(0, metrics.offset ?? 0);
        total = Math.max(items.length, metrics.total ?? items.length);
        render();
    }

    function render() {
        const firstVisible = Math.floor(container.scrollTop / rowHeight()) - offset;
        const visible = Math.ceil(container.clientHeight / rowHeight()) || 1;
        const start = Math.min(items.length, Math.max(0, firstVisible - OVERSCAN));
        const end = Math.min(
            items.length, Math.max(start, firstVisible + visible + OVERSCAN));
        container.replaceChildren(
            spacer((offset + start) * rowHeight()),
            ...items.slice(start, end).map((item, offset) =>
                createRow(item, start + offset)),
            spacer(Math.max(0, total - offset - end) * rowHeight()),
        );
    }

    function ensureVisible(index) {
        const top = (offset + index) * rowHeight();
        const bottom = top + rowHeight();
        if (top < container.scrollTop) container.scrollTop = top;
        else if (bottom > container.scrollTop + container.clientHeight) {
            container.scrollTop = bottom - container.clientHeight;
        }
        render();
    }

    container.addEventListener('scroll', render);
    return { setItems, render, ensureVisible, get rowHeight() { return rowHeight(); } };
}

function spacer(height) {
    const element = document.createElement('div');
    element.className = 'library-spacer';
    element.style.height = `${height}px`;
    return element;
}
