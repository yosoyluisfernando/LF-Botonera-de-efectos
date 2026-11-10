/** Gestos visuales; las acciones y el estado siguen siendo responsabilidad de Rust. */
const TARGETS = '.grid-item, .fixed-panel-item, .tab, .player-row, .library-row, '
    + '.library-table-row, #clock-widget, #master-volume-btn';
let wired = false;

export function initTouchControls() {
    if (wired) return;
    wired = true;
    let pending = null;
    let consumed = null;
    const clear = () => {
        if (pending) clearTimeout(pending.timer);
        pending = null;
    };
    document.addEventListener('pointerdown', event => {
        if (event.pointerType !== 'touch' || !event.isPrimary) { clear(); return; }
        const target = event.target.closest(TARGETS);
        if (!target || event.target.closest('input, select, textarea')) return;
        consumed = null;
        clear();
        const { clientX: x, clientY: y, pointerId } = event;
        pending = { target, x, y, pointerId, timer: setTimeout(() => {
            pending = null;
            consumed = { target, until: performance.now() + 900 };
            target.dispatchEvent(new MouseEvent('contextmenu', {
                bubbles: true, cancelable: true, clientX: x, clientY: y, button: 2,
            }));
        }, 550) };
    }, true);
    document.addEventListener('pointermove', event => {
        if (pending && (Math.hypot(event.clientX - pending.x, event.clientY - pending.y) > 10
            || event.pointerId !== pending.pointerId)) clear();
    }, true);
    for (const name of ['pointerup', 'pointercancel', 'scroll']) {
        document.addEventListener(name, clear, true);
    }
    window.addEventListener('blur', clear);
    document.addEventListener('contextmenu', event => {
        if (event.isTrusted && event.pointerType === 'touch' && event.target.closest(TARGETS)) {
            event.preventDefault();
            event.stopImmediatePropagation();
            return;
        }
        if (!event.isTrusted || !consumed || performance.now() > consumed.until) return;
        if (consumed.target.contains(event.target)) {
            event.preventDefault();
            event.stopImmediatePropagation();
        }
    }, true);
    document.addEventListener('click', event => {
        if (!consumed || performance.now() > consumed.until) return;
        if (consumed.target.contains(event.target)) {
            event.preventDefault();
            event.stopImmediatePropagation();
            consumed = null;
        }
    }, true);
}
