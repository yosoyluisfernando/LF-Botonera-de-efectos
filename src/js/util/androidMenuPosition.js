/** Posición de menús Android según el viewport visible y los márgenes del sistema. */
const active = new Map();
let safeArea;

export function placeAndroidMenu(menu, x, y) {
    if (!safeArea) {
        safeArea = document.createElement('div');
        safeArea.className = 'android-safe-area-probe';
        safeArea.setAttribute('aria-hidden', 'true');
        document.body.append(safeArea);
        window.addEventListener('resize', reposition);
        window.visualViewport?.addEventListener('resize', reposition);
        window.visualViewport?.addEventListener('scroll', reposition);
    }
    active.set(menu, { x, y });
    reposition();
}

function reposition() {
    for (const [menu, point] of active) {
        if (!menu.isConnected || menu.classList.contains('hidden')) {
            active.delete(menu);
            continue;
        }
        position(menu, point);
    }
}

function position(menu, { x, y }) {
    const bounds = visibleBounds();
    const margin = 8;
    const left = bounds.left + margin;
    const top = bounds.top + margin;
    const right = bounds.right - margin;
    const bottom = bounds.bottom - margin;
    menu.classList.add('android-viewport-menu');
    menu.style.maxWidth = `${Math.max(0, right - left)}px`;
    menu.style.maxHeight = `${Math.max(0, bottom - top)}px`;
    const { width, height } = menu.getBoundingClientRect();
    menu.style.left = `${Math.max(left, Math.min(x, right - width))}px`;
    menu.style.top = `${Math.max(top, Math.min(y, bottom - height))}px`;
}

function visibleBounds() {
    const viewport = window.visualViewport;
    const x = viewport?.offsetLeft ?? 0;
    const y = viewport?.offsetTop ?? 0;
    const width = viewport?.width ?? window.innerWidth;
    const height = viewport?.height ?? window.innerHeight;
    const style = getComputedStyle(safeArea);
    const top = parseFloat(style.paddingTop) || 0;
    const bottom = parseFloat(style.paddingBottom) || 0;
    const left = parseFloat(style.paddingLeft) || 0;
    const right = parseFloat(style.paddingRight) || 0;
    // Algunos WebView no exponen las barras. En ese caso reserva margen solo para los menús.
    const fallback = bottom || left || right ? 0 : 48;
    const side = width > height ? fallback : 0;
    return {
        left: Math.max(x, left, side),
        top: Math.max(y, top),
        right: Math.min(x + width, window.innerWidth - Math.max(right, side)),
        bottom: Math.min(y + height, window.innerHeight - Math.max(bottom, fallback)),
    };
}
