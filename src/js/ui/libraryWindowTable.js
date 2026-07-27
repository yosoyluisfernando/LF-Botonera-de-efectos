/** Tabla virtual de la Biblioteca: catálogo indexado o carpeta explorada. */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { mmss } from '../util/durationFormat.js';
import { createLibraryWindowSource } from './libraryWindowSource.js';
import { createLibraryVirtualList } from './libraryVirtualList.js';
import {
    activeLibraryPath, clearLibrarySelection, isLibrarySelected,
    selectLibraryClick, selectionForLibraryContext,
} from './librarySelection.js';
import { showLibraryContextMenu } from './libraryContextMenu.js';
import { libraryDisplayMode } from './libraryDisplayMode.js';
import { initLibraryWindowKeyboard } from './libraryWindowKeyboard.js';

const source = createLibraryWindowSource();
let items = [];
let virtual = null;
let scope = {};
let kind = 'catalog';
let debounce = null;
let scrollDebounce = null;

export function initLibraryWindowTable() {
    const table = document.getElementById('library-table');
    virtual = createLibraryVirtualList(table, row);
    initLibraryWindowKeyboard(table, () => items, () => virtual);
    table.addEventListener('scroll', scheduleVisibleWindow);
    document.getElementById('library-window-search').addEventListener('input', scheduleSearch);
}

export async function showCatalog(nextScope = {}) {
    kind = 'catalog';
    scope = nextScope;
    clearLibrarySelection();
    await loadCatalog();
}

export async function showDirectory(path) {
    kind = 'directory';
    scope = { path };
    document.getElementById('library-window-search').value = '';
    clearLibrarySelection();
    const entries = await invoke('library_read_directory', { path });
    items = entries.filter(entry => !entry.is_directory).map(entry => ({
        ...entry,
        file_name: entry.name,
        duration_s: 0,
        title: null,
        artist: null,
        album: null,
    }));
    paint({ offset: 0, total: items.length });
    setStatus(t('library.files_count').replace('{count}', items.length));
}

export function refreshLibraryWindowTable() {
    return kind === 'directory' ? showDirectory(scope.path) : loadCatalog();
}

export function repaintLibraryWindowTable() {
    header();
    virtual.render();
}

async function loadCatalog() {
    const query = document.getElementById('library-window-search').value.trim();
    const effectiveScope = query ? {} : scope;
    const result = await source.fresh(query, {
        ...effectiveScope,
        collection: scope.collection ?? null,
    });
    if (result.stale) return;
    items = result.items;
    document.getElementById('library-table').scrollTop = 0;
    paint(result);
    setStatus(query
        ? t('library.results_count').replace('{count}', result.total)
        : t('library.files_count').replace('{count}', result.total));
}

function paint(metrics = {}) {
    header();
    virtual.setItems(items, metrics);
    document.getElementById('library-empty').classList.toggle('hidden', items.length > 0);
}

function header() {
    const filename = libraryDisplayMode() === 'filename' || kind === 'directory';
    const target = document.getElementById('library-table-header');
    target.className = `library-table-header ${filename ? 'filename' : 'metadata'}`;
    target.innerHTML = filename
        ? `<span>${t('library.col_name')}</span><span>${t('library.col_path')}</span>
           <span>${t('library.col_duration')}</span>`
        : `<span>${t('library.col_title')}</span><span>${t('library.col_artist')}</span>
           <span>${t('library.col_album')}</span><span>${t('library.col_genre')}</span>
           <span>${t('library.col_year')}</span><span>${t('library.col_duration')}</span>`;
}

function row(item, index) {
    const filename = libraryDisplayMode() === 'filename' || kind === 'directory';
    const element = document.createElement('div');
    element.id = `library-window-row-${index}`;
    element.className = `library-table-row ${filename ? 'filename' : 'metadata'}`;
    element.dataset.path = item.path;
    element.setAttribute('role', 'option');
    element.setAttribute('aria-selected', String(isLibrarySelected(item.path)));
    element.classList.toggle('selected', isLibrarySelected(item.path));
    element.classList.toggle('active', activeLibraryPath() === item.path);
    element.classList.toggle('library-folder-row', !!item.is_directory);
    const values = filename
        ? [item.is_directory ? `▸ ${item.file_name}` : item.file_name,
            item.path, item.is_directory ? '' : mmss(item.duration_s)]
        : [item.title || item.file_name, item.artist || '', item.album || '',
            item.genre || '', item.year || '', mmss(item.duration_s)];
    values.forEach((value, cellIndex) => {
        const cell = document.createElement('span');
        cell.className = `library-table-cell${cellIndex === values.length - 1
            ? ' duration' : cellIndex === 1 && filename ? ' path' : ''}`;
        cell.textContent = value;
        cell.title = value;
        element.appendChild(cell);
    });
    element.addEventListener('click', event => {
        if (item.is_directory) return showDirectory(item.path);
        selectLibraryClick(event, item, items);
        virtual.render();
    });
    element.addEventListener('contextmenu', event => {
        if (item.is_directory) return;
        event.preventDefault();
        const selection = selectionForLibraryContext(item, items);
        virtual.render();
        showLibraryContextMenu(event.clientX, event.clientY, selection);
    });
    return element;
}

function scheduleSearch() {
    clearTimeout(debounce);
    debounce = setTimeout(() => {
        kind = 'catalog';
        loadCatalog().catch(showError);
    }, 160);
}

function scheduleVisibleWindow() {
    virtual.render();
    clearTimeout(scrollDebounce);
    scrollDebounce = setTimeout(loadVisibleWindow, 35);
}

async function loadVisibleWindow() {
    if (kind !== 'catalog' || document.getElementById('library-window-search').value.trim()) return;
    const table = document.getElementById('library-table');
    const index = Math.floor(table.scrollTop / virtual.rowHeight);
    const result = await source.at(index);
    if (!result || result.stale) return;
    items = result.items;
    paint(result);
}

function setStatus(message) {
    document.getElementById('library-window-status').textContent = message;
}

function showError(error) {
    setStatus(String(error));
}
