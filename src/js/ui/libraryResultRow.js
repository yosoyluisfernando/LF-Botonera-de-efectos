/** Construye una fila accesible sin decidir selección ni acciones. */
import { mmss } from '../util/durationFormat.js';

export function createLibraryResultRow(item, index, state, handlers) {
    const row = document.createElement('div');
    row.id = `library-result-${index}`;
    row.className = 'library-row';
    row.dataset.path = item.path;
    row.setAttribute('role', 'option');
    row.setAttribute('aria-selected', String(state.selected));
    row.classList.toggle('selected', state.selected);
    row.classList.toggle('active', state.active);
    const metadata = [item.artist, item.album].filter(Boolean).join(' — ');
    const tagged = [item.title, item.artist].filter(Boolean).join(' — ');
    const filenameMode = state.displayMode === 'filename';
    row.innerHTML = `<span class="library-row-main">
        <span class="library-row-title"></span><span class="library-row-meta"></span>
      </span><span class="library-row-duration">${mmss(item.duration_s)}</span>`;
    row.querySelector('.library-row-title').textContent = filenameMode
        ? item.file_name : item.title || item.file_name;
    row.querySelector('.library-row-meta').textContent = filenameMode
        ? tagged : metadata || item.file_name;
    row.addEventListener('click', event => handlers.click(event, item, index));
    row.addEventListener('contextmenu', event => handlers.context(event, item, index));
    row.addEventListener('mousedown', event => handlers.drag(event, item, index));
    return row;
}
