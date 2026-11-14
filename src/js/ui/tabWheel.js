/** Rueda de pestañas, adaptada de Danilo Wenner (LF Botonera Radio).
 * JS interpreta el gesto; Rust decide la pestaña mediante cycle_paleta. */
export function initTabWheel(bar, cycle) {
    if (!bar || bar.dataset.wheelReady) return;
    bar.dataset.wheelReady = '1';
    let accumulated = 0;
    let last = -Infinity;
    let busy = false;
    bar.addEventListener('wheel', async e => {
        if (e.ctrlKey || e.altKey || e.shiftKey || Math.abs(e.deltaY) <= Math.abs(e.deltaX)) return;
        e.preventDefault();
        if (busy) return;
        const unit = e.deltaMode === 1 ? 16 : e.deltaMode === 2 ? bar.clientHeight : 1;
        const delta = e.deltaY * unit;
        if (Math.sign(delta) !== Math.sign(accumulated)) accumulated = 0;
        accumulated += delta;
        const now = performance.now();
        if (Math.abs(accumulated) < 40 || now - last < 120) return;
        const offset = accumulated > 0 ? 1 : -1;
        accumulated = 0;
        last = now;
        busy = true;
        try { await cycle(offset); }
        catch (error) { console.error('cycle_paleta:', error); }
        finally { busy = false; }
    }, { passive: false });
}
