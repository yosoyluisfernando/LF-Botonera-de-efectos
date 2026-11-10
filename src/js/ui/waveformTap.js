/** Android: toca para situar el cursor y desliza para explorar la onda. */
export function bindWaveformTap(inner, seek) {
    let tap = null;
    inner.addEventListener('pointerdown', event => {
        tap = event.pointerType === 'touch' && event.isPrimary
            ? { id: event.pointerId, x: event.clientX, y: event.clientY }
            : null;
    });
    inner.addEventListener('pointermove', event => {
        if (tap?.id === event.pointerId && moved(event, tap)) tap = null;
    });
    inner.addEventListener('pointercancel', () => { tap = null; });
    inner.addEventListener('pointerup', event => {
        if (tap?.id === event.pointerId && !moved(event, tap)) seek(event.clientX);
        tap = null;
    });
}

function moved(event, tap) {
    return Math.hypot(event.clientX - tap.x, event.clientY - tap.y) > 10;
}
