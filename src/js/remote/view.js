import { t } from '../util/i18n.js';
import { paintAdaptive } from '../util/colorAdapter.js';

export function renderSnapshot(root, snapshot, command) {
    root.querySelector('.remote-palette').textContent = [snapshot.profile, snapshot.name].filter(Boolean).join(' / ');
    root.querySelector('.remote-empty').textContent = snapshot.unavailable
        ? t('remote.audio_only') : (!snapshot.buttons.length ? t('remote.empty') : '');
    renderTabs(root.querySelector('.remote-tabs'), snapshot, command);
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
            const cell = document.createElement('button');
            cell.className = 'remote-cell';
            cell.type = 'button';
            cell.disabled = !button?.enabled;
            const number = document.createElement('span');
            number.className = 'remote-index';
            number.textContent = index;
            cell.append(number);
            if (button) {
                cell.dataset.trigger = button.id;
                const name = button.name || button.label || String(index);
                cell.setAttribute('aria-label', name);
                const label = document.createElement('span');
                label.className = 'remote-label';
                const visual = button.visual;
                if (visual?.kind === 'emoji' && visual.mode !== 'text') {
                    label.textContent = visual.mode === 'visual'
                        ? visual.value : `${visual.value} ${name}`;
                } else {
                    label.textContent = name;
                }
                const duration = document.createElement('span');
                duration.className = 'remote-duration';
                duration.textContent = button.duration_str;
                cell.append(label, duration);
                paintAdaptive(cell, button.color_bg, button.color_text, 'button');
                cell.addEventListener('click', () => command({ command: 'play', palette: snapshot.palette, id: button.id }));
            }
            grid.append(cell);
        }
    }
    const enabled = new Map(snapshot.buttons.map(b => [b.id, b.enabled]));
    const playing = new Set(snapshot.playing.map(b => b.id));
    grid.querySelectorAll('[data-trigger]').forEach(cell => {
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

function renderTabs(list, snapshot, command) {
    const tabs = snapshot.tabs ?? [];
    const signature = JSON.stringify([snapshot.profile_id, snapshot.palette, tabs, document.documentElement.dataset.theme]);
    if (list.dataset.signature !== signature) {
        list.dataset.signature = signature;
        list.replaceChildren();
        for (const tab of tabs) {
            const item = document.createElement('button');
            item.type = 'button';
            item.dataset.remoteTab = tab.id;
            item.textContent = tab.name;
            item.setAttribute('aria-current', tab.id === snapshot.palette ? 'true' : 'false');
            paintAdaptive(item, tab.color_bg || '#3a3f44', tab.color_text || '#ffffff', 'tab');
            item.addEventListener('click', () => command({
                command: 'select_tab', profile: snapshot.profile_id, palette: tab.id,
            }));
            list.append(item);
        }
    }
    const enabled = new Map(tabs.map(tab => [tab.id, tab.enabled]));
    for (const item of list.children) item.disabled = !enabled.get(item.dataset.remoteTab);
}
