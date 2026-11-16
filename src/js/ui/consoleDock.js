import '../../css/consoleDock.css';
import { invoke } from '../bridge/api.js';

export function isDocked(side) {
    return side === 'left' || side === 'right';
}

export function setDockWidth(dock, handle, width) {
    const value = Math.round(Math.min(640, Math.max(280, width)));
    dock.style.setProperty('--console-dock-width', `${value}px`);
    handle.setAttribute('aria-valuenow', value);
    return value;
}

export function wireDockResize(dock, handle) {
    let drag = null;
    const readWidth = () => Number(handle.getAttribute('aria-valuenow'));
    const save = width => invoke('set_console_mode', { mode: dock.dataset.side, width })
        .catch(console.error);

    handle.addEventListener('pointerdown', event => {
        if (event.button !== 0 || drag) return;
        drag = { id: event.pointerId, x: event.clientX, width: dock.getBoundingClientRect().width,
            preferred: readWidth() };
        handle.setPointerCapture(event.pointerId);
    });
    handle.addEventListener('pointermove', event => {
        if (drag?.id !== event.pointerId) return;
        const delta = event.clientX - drag.x;
        const width = !delta ? drag.preferred
            : drag.width + delta * (dock.dataset.side === 'left' ? 1 : -1);
        setDockWidth(dock, handle, width);
    });
    handle.addEventListener('pointerup', event => {
        if (drag?.id !== event.pointerId) return;
        const { preferred } = drag;
        drag = null;
        const width = readWidth();
        if (width !== preferred) save(width);
    });
    handle.addEventListener('lostpointercapture', event => {
        if (drag?.id !== event.pointerId) return;
        setDockWidth(dock, handle, drag.preferred);
        drag = null;
    });
    handle.addEventListener('keydown', event => {
        if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
        event.preventDefault();
        const expands = event.key === (dock.dataset.side === 'left' ? 'ArrowRight' : 'ArrowLeft');
        const width = { Home: 280, End: 640 }[event.key] ?? (readWidth() + (expands ? 16 : -16));
        save(setDockWidth(dock, handle, width));
    });
}
