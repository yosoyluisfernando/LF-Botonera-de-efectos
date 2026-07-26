/** Renderiza únicamente las filas visibles y un colchón de 50 por lado. */
const ROW_HEIGHT = 40;
const OVERSCAN = 50;

export function createLibraryVirtualList(container, createRow) {
    let items = [];

    function setItems(next) {
        items = next;
        render();
    }

    function render() {
        const firstVisible = Math.floor(container.scrollTop / ROW_HEIGHT);
        const visible = Math.ceil(container.clientHeight / ROW_HEIGHT) || 1;
        const start = Math.max(0, firstVisible - OVERSCAN);
        const end = Math.min(items.length, firstVisible + visible + OVERSCAN);
        container.replaceChildren(
            spacer(start * ROW_HEIGHT),
            ...items.slice(start, end).map((item, offset) =>
                createRow(item, start + offset)),
            spacer((items.length - end) * ROW_HEIGHT),
        );
    }

    function ensureVisible(index) {
        const top = index * ROW_HEIGHT;
        const bottom = top + ROW_HEIGHT;
        if (top < container.scrollTop) container.scrollTop = top;
        else if (bottom > container.scrollTop + container.clientHeight) {
            container.scrollTop = bottom - container.clientHeight;
        }
        render();
    }

    container.addEventListener('scroll', render);
    return { setItems, render, ensureVisible, rowHeight: ROW_HEIGHT };
}

function spacer(height) {
    const element = document.createElement('div');
    element.className = 'library-spacer';
    element.style.height = `${height}px`;
    return element;
}
