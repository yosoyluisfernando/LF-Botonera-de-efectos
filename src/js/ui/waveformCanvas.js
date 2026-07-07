/**
 * Archivo: waveformCanvas.js
 * Propósito: motor de dibujo e interacción de la onda (Regla 4: solo pinta).
 * Inspirado en el editor del LF Automatizador (traducido, no copiado):
 * envolvente rellena estilo Audition, zoom ensanchando el lienzo dentro de un
 * contenedor con scroll, cursor de reproducción y marcadores Inicio/Fin. La
 * amplitud dibujada se multiplica por la ganancia → la onda refleja el volumen.
 *
 * @param {object} refs  { container, inner, canvas, cursor, timeText }
 * @param {object} hooks { onCursorChange(t), onMarkerChange(start,end), onZoom(dir) }
 */
export function createWaveform(refs, hooks) {
    const { container, inner, canvas, cursor, timeText } = refs;
    const ctx = canvas.getContext('2d');
    let duration = 0, peaks = [], gain = 1, zoom = 1;
    let cueStart = 0, cueEnd = null, cursorT = 0, drag = null;

    const cssVar = (n, fallback = '#0078d4') =>
        getComputedStyle(document.documentElement).getPropertyValue(n).trim() || fallback;
    const baseW = () => container.clientWidth || 800;
    const dpr = () => window.devicePixelRatio || 1;
    const tToX = t => (t / (duration || 1)) * canvas.width;
    const xToT = x => (x / (canvas.width || 1)) * (duration || 0);

    function setData(d) { duration = d.duration || 0; peaks = d.peaks || []; cursorT = 0; layout(); }
    function setGain(g) { gain = g; draw(); }
    function setMarkers(s, e) { cueStart = s; cueEnd = e; draw(); }
    function setCursor(t, follow) { cursorT = Math.max(0, Math.min(duration, t)); placeCursor(follow); }
    function getCursor() { return cursorT; }
    function setZoom(z) { zoom = Math.max(1, z); layout(); }

    function layout() {
        const w = Math.max(1, Math.round(baseW() * zoom));
        inner.style.width = `${w}px`;
        canvas.style.width = `${w}px`;
        canvas.style.height = '100%';
        canvas.width = Math.round(w * dpr());
        canvas.height = Math.round((container.clientHeight || 300) * dpr());
        draw();
        placeCursor(false);
    }

    function draw() {
        const w = canvas.width, h = canvas.height, mid = h / 2;
        ctx.fillStyle = cssVar('--waveform-bg', '#0d0d0f');
        ctx.fillRect(0, 0, w, h);
        const n = peaks.length / 2;
        const top = new Float32Array(w);
        let clipped = false;
        for (let i = 0; i < w; i++) {
            const b0 = Math.floor((i / w) * n);
            const b1 = Math.max(b0 + 1, Math.ceil(((i + 1) / w) * n));
            let a = 0;
            for (let j = b0; j < b1 && j < n; j++) {
                const v = Math.max(Math.abs(peaks[j * 2]), Math.abs(peaks[j * 2 + 1]));
                if (v > a) a = v;
            }
            let amp = a * gain;
            if (amp > 1) { amp = 1; clipped = true; }
            top[i] = amp;
        }
        ctx.fillStyle = clipped ? cssVar('--error-color') : cssVar('--waveform-color');
        ctx.beginPath();
        ctx.moveTo(0, mid);
        for (let i = 0; i < w; i++) ctx.lineTo(i, mid - top[i] * mid);
        for (let i = w - 1; i >= 0; i--) ctx.lineTo(i, mid + top[i] * mid);
        ctx.closePath();
        ctx.fill();
        ctx.strokeStyle = cssVar('--waveform-midline', 'rgba(255,255,255,0.08)');
        ctx.lineWidth = 1;
        ctx.beginPath(); ctx.moveTo(0, mid); ctx.lineTo(w, mid); ctx.stroke();
        dim(0, cueStart);
        if (cueEnd != null) dim(cueEnd, duration);
        marker(cueStart, cssVar('--success-strong'), 'INICIO');
        if (cueEnd != null) marker(cueEnd, cssVar('--error-color'), 'FIN');
    }
    function dim(t0, t1) {
        const x0 = Math.max(0, tToX(t0)), x1 = Math.min(canvas.width, tToX(t1));
        if (x1 <= x0) return;
        ctx.fillStyle = cssVar('--waveform-dim-bg', 'rgba(0,0,0,0.55)');
        ctx.fillRect(x0, 0, x1 - x0, canvas.height);
    }
    function marker(t, color, lbl) {
        const x = tToX(t);
        ctx.strokeStyle = color;
        ctx.lineWidth = 2 * dpr();
        ctx.setLineDash([6 * dpr(), 5 * dpr()]);
        ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, canvas.height); ctx.stroke();
        ctx.setLineDash([]);
        ctx.font = `bold ${12 * dpr()}px Consolas, monospace`;
        const tw = ctx.measureText(lbl).width;
        ctx.fillStyle = color;
        ctx.fillRect(x + 2, 4 * dpr(), tw + 8 * dpr(), 16 * dpr());
        ctx.fillStyle = '#000';
        ctx.fillText(lbl, x + 6 * dpr(), 16 * dpr());
    }

    function placeCursor(follow) {
        const xcss = tToX(cursorT) / dpr();
        cursor.style.left = `${xcss}px`;
        timeText.textContent = fmt(cursorT);
        if (follow) {
            const left = container.scrollLeft, right = left + container.clientWidth;
            if (xcss < left + 20 || xcss > right - 20) container.scrollLeft = xcss - container.clientWidth / 2;
        }
    }
    function fmt(t) {
        const m = Math.floor(t / 60), s = Math.floor(t % 60), ms = Math.floor((t % 1) * 1000);
        return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}.${String(ms).padStart(3, '0')}`;
    }

    const pxTime = clientX => xToT((clientX - inner.getBoundingClientRect().left) * dpr());
    function pick(t) {
        const tol = xToT(9 * dpr());
        if (Math.abs(t - cueStart) <= tol) return 'start';
        if (cueEnd != null && Math.abs(t - cueEnd) <= tol) return 'end';
        return null;
    }
    function moveMarker(which, t) {
        t = Math.max(0, Math.min(duration, t));
        if (which === 'end') cueEnd = Math.max(cueStart + 0.01, t);
        else cueStart = Math.max(0, Math.min(t, cueEnd != null ? cueEnd - 0.01 : duration));
        draw();
        hooks.onMarkerChange(cueStart, cueEnd);
    }
    function fijar(which) {
        if (which === 'start') cueStart = Math.min(cursorT, cueEnd != null ? cueEnd - 0.01 : duration);
        else cueEnd = Math.max(cursorT, cueStart + 0.01);
        draw();
        hooks.onMarkerChange(cueStart, cueEnd);
    }
    function clearEnd() { cueEnd = null; draw(); hooks.onMarkerChange(cueStart, null); }

    inner.addEventListener('pointerdown', e => {
        const t = pxTime(e.clientX);
        drag = pick(t) || 'cursor';
        if (drag === 'cursor') { setCursor(t, false); hooks.onCursorChange(cursorT); }
        inner.setPointerCapture(e.pointerId);
    });
    inner.addEventListener('pointermove', e => {
        if (!drag) return;
        const t = pxTime(e.clientX);
        if (drag === 'cursor') { setCursor(t, false); hooks.onCursorChange(cursorT); }
        else moveMarker(drag, t);
    });
    inner.addEventListener('pointerup', e => { drag = null; inner.releasePointerCapture?.(e.pointerId); });
    container.addEventListener('wheel', e => {
        if (!e.ctrlKey) return;
        e.preventDefault();
        hooks.onZoom(e.deltaY < 0 ? 1 : -1);
    }, { passive: false });
    document.addEventListener('lf-theme-change', draw);

    return { setData, setGain, setMarkers, setCursor, getCursor, setZoom, fijar, clearEnd, relayout: layout };
}
