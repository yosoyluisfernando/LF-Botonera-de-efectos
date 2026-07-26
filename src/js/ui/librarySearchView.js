/** Tercera vista del panel fijo: consulta Rust y pinta una lista virtual. */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { createLibraryVirtualList } from './libraryVirtualList.js';
import {
    activeLibraryPath, clearLibrarySelection, isLibrarySelected,
    librarySelection, moveLibrarySelection, selectLibraryClick,
    selectionForLibraryContext,
} from './librarySelection.js';
import { showLibraryContextMenu } from './libraryContextMenu.js';
import { initLibraryDnd, startLibraryDrag } from './libraryDnd.js';
import { createLibraryResultSource } from './libraryResultSource.js';
import { createLibraryResultRow } from './libraryResultRow.js';
let items = [];
const source = createLibraryResultSource();
let virtual = null;
let wired = false;
let loading = false;
let generation = 0;
let refreshApp = null;
let debounce = null;
export function initLibrarySearch(onRefresh) {
    refreshApp = onRefresh;
    if (wired) return;
    wired = true;
    const rows = element('rows');
    virtual = createLibraryVirtualList(rows, createRow);
    initLibraryDnd(onRefresh);
    element('input').addEventListener('input', scheduleFresh);
    element('collection').addEventListener('change', loadFresh);
    element('input').addEventListener('keydown', focusResults);
    rows.addEventListener('keydown', navigate);
    rows.addEventListener('scroll', maybeLoad);
}
export async function drawLibrarySearch() {
    if (!items.length) await loadFresh();
    else virtual.render();
}
export async function refreshLibrarySearch() {
    await loadFresh();
}
async function loadFresh() {
    const ownGeneration = ++generation;
    loading = true;
    clearLibrarySelection();
    element('rows').removeAttribute('aria-activedescendant');
    status('library.searching');
    try {
        const query = element('input').value.trim();
        const collection = valueOrNull(element('collection').value);
        const result = await source.fresh(query, collection);
        if (result.stale || ownGeneration !== generation) return;
        items = result.items;
        if (result.searchCount !== null) {
            statusCount(result.searchCount);
        } else {
            const totals = await invoke('library_status');
            const total = collection ? totals[collection] : totals.present;
            statusText(t('library.catalog_count').replace('{count}', total));
        }
        element('rows').scrollTop = 0;
        virtual.setItems(items);
    } catch (error) {
        statusText(String(error));
    } finally {
        if (ownGeneration === generation) loading = false;
    }
}
function createRow(item, index) {
    return createLibraryResultRow(item, index, {
        selected: isLibrarySelected(item.path),
        active: activeLibraryPath() === item.path,
    }, {
        click: event => {
            selectLibraryClick(event, item, items);
            element('rows').setAttribute('aria-activedescendant', `library-result-${index}`);
            virtual.render();
        },
        context: event => openContext(event, item),
        drag: event => {
            if (event.button !== 0 || event.ctrlKey || event.shiftKey) return;
            const selection = selectionForLibraryContext(item, items);
            startLibraryDrag(event, selection);
        },
    });
}
function openContext(event, item) {
    event.preventDefault();
    const selection = selectionForLibraryContext(item, items);
    virtual.render();
    showLibraryContextMenu(event.clientX, event.clientY, selection, refreshApp);
}
async function navigate(event) {
    if (event.key === 'Escape') {
        clearLibrarySelection();
        return virtual.render();
    }
    if ((event.key === 'ContextMenu') || (event.shiftKey && event.key === 'F10')) {
        event.preventDefault();
        return openKeyboardContext();
    }
    const page = Math.max(1, Math.floor(element('rows').clientHeight / virtual.rowHeight));
    const delta = event.key === 'ArrowDown' ? 1
        : event.key === 'ArrowUp' ? -1
            : event.key === 'PageDown' ? page
                : event.key === 'PageUp' ? -page : 0;
    if (!delta) return;
    event.preventDefault();
    await ensureDirection(delta);
    const index = moveLibrarySelection(items, delta, event.shiftKey);
    if (index >= 0) {
        virtual.ensureVisible(index);
        element('rows').setAttribute('aria-activedescendant', `library-result-${index}`);
    }
}
async function ensureDirection(delta) {
    const active = items.findIndex(item => item.path === activeLibraryPath());
    if (delta > 0 && active >= items.length - 1 && source.can('forward')) {
        await extend('forward');
    }
    if (delta < 0 && active <= 0 && source.can('backward')) {
        await extend('backward');
    }
}

function openKeyboardContext() {
    const item = items.find(value => value.path === activeLibraryPath()) ?? items[0];
    if (!item) return;
    const row = [...element('rows').querySelectorAll('.library-row')]
        .find(value => value.dataset.path === item.path);
    const rect = row?.getBoundingClientRect() ?? element('rows').getBoundingClientRect();
    openContext({ preventDefault() {}, clientX: rect.left + 12, clientY: rect.top + 12 }, item);
}

function focusResults(event) {
    if (event.key !== 'ArrowDown' || !items.length) return;
    event.preventDefault();
    element('rows').focus();
    if (!librarySelection().length) {
        moveLibrarySelection(items, 1, false);
        virtual.ensureVisible(0);
    }
}

function scheduleFresh() {
    clearTimeout(debounce);
    debounce = setTimeout(loadFresh, 160);
}

async function maybeLoad() {
    if (loading || element('input').value.trim()) return;
    const rows = element('rows');
    if (rows.scrollTop < 80 && source.can('backward')) await extend('backward');
    else if (rows.scrollHeight - rows.scrollTop - rows.clientHeight < 80
        && source.can('forward')) {
        await extend('forward');
    }
}

async function extend(direction) {
    if (loading) return;
    loading = true;
    const rows = element('rows');
    const oldTop = rows.scrollTop;
    try {
        const result = await source.extend(
            direction, valueOrNull(element('collection').value));
        if (!result) return;
        items = result.items;
        virtual.setItems(items);
        rows.scrollTop = Math.max(
            0, oldTop + result.scrollRows * virtual.rowHeight);
    } finally {
        loading = false;
    }
}

function status(key) {
    statusText(t(key));
}

function statusCount(count) {
    statusText(t('library.results_count').replace('{count}', count));
}

function statusText(text) {
    element('status').textContent = text;
}

function valueOrNull(value) {
    return value || null;
}

function element(name) {
    return document.getElementById(`library-search-${name}`);
}
