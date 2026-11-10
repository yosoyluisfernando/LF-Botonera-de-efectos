import { t } from '../util/i18n.js';
import { paintAdaptive } from '../util/colorAdapter.js';

export function renderSnapshot(root, snapshot, command) {
    root.querySelector('.remote-palette').textContent = [snapshot.profile, snapshot.name].filter(Boolean).join(' / ');
    root.querySelector('.remote-empty').textContent = snapshot.unavailable
        ? t('remote.audio_only') : (!snapshot.buttons.length ? t('remote.empty') : '');
    const grid = root.querySelector('.remote-grid');
    const signature = JSON.stringify([snapshot.palette, snapshot.columns, snapshot.rows, snapshot.buttons, document.documentElement.dataset.theme]);
    if (grid.dataset.signature !== signature) {
        grid.dataset.signature = signature;
        grid.replaceChildren();
        grid.style.setProperty('--remote-columns', Math.max(1, snapshot.columns));
        const byIndex = new Map(snapshot.buttons.map(button => [button.index, button]));
        const count = Math.max(1, snapshot.rows * snapshot.columns);
        for (let index = 1; index <= count; index++) {
            const button = byIndex.get(index);
            const cell = document.createElement(button ? 'button' : 'div');
            cell.className = 'remote-cell';
            if (button) {
                cell.type = 'button'; cell.dataset.trigger = button.id;
                cell.textContent = button.name || button.label || String(index);
                paintAdaptive(cell, button.color_bg, button.color_text);
                cell.addEventListener('click', () => command({ command: 'play', palette: snapshot.palette, id: button.id }));
            } else { cell.setAttribute('aria-hidden', 'true'); }
            grid.append(cell);
        }
    }
    const enabled = new Map(snapshot.buttons.map(b => [b.id, b.enabled]));
    const playing = new Set(snapshot.playing.map(b => b.id));
    grid.querySelectorAll('button').forEach(cell => {
        cell.disabled = !enabled.get(cell.dataset.trigger);
        cell.classList.toggle('is-playing', playing.has(cell.dataset.trigger));
        cell.setAttribute('aria-pressed', String(playing.has(cell.dataset.trigger)));
    });
    const list = root.querySelector('.remote-playing');
    const currentIds = snapshot.playing.map(b => b.id).join('|');
    if (list.dataset.ids !== currentIds) {
        list.dataset.ids = currentIds; list.replaceChildren();
        for (const item of snapshot.playing) {
            const stop = document.createElement('button');
            const button = snapshot.buttons.find(b => b.id === item.id);
            stop.type = 'button';
            stop.textContent = `${t('remote.stop')}: ${button?.name || button?.label || t('remote.other_effect')}`;
            stop.addEventListener('click', () => command({ command: 'stop', id: item.id }));
            list.append(stop);
        }
    }
}
