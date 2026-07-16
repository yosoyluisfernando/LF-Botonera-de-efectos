export function paintPlayback(selector, buttons, ticks) {
    const playing = new Map(ticks.map(t => [t.id, t]));
    document.querySelectorAll(selector).forEach(cell => {
        const tick = playing.get(cell.dataset.id);
        const timer = cell.querySelector('.timer');
        const bar = cell.querySelector('.progress-bar');
        if (tick) {
            cell.classList.add('playing');
            if (timer) timer.textContent = tick.duration > 0
                ? `${Number(tick.remaining).toFixed(1)}s` : `${Number(tick.pos).toFixed(1)}s`;
            if (bar) bar.style.width = `${tick.progress_percent}%`;
        } else {
            cell.classList.remove('playing');
            if (timer) timer.textContent = buttons[cell.dataset.id]?.duration_str || '';
            if (bar) bar.style.width = '100%';
        }
    });
}
