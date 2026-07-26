/** Ventana bidireccional de resultados; nunca conserva el catálogo completo. */
import { invoke } from '../bridge/api.js';

const CHUNK = 120;
const MAX_WINDOW = 180;

export function createLibraryResultSource() {
    let items = [];
    let previousCursor = null;
    let nextCursor = null;
    let hasPrevious = false;
    let hasNext = false;
    let loading = false;
    let version = 0;

    async function fresh(query, collection) {
        const ownVersion = ++version;
        if (query) {
            const found = await invoke('library_search', { query, collection, limit: 500 });
            if (ownVersion !== version) return { stale: true };
            items = found;
            previousCursor = nextCursor = null;
            hasPrevious = hasNext = false;
            return { items, searchCount: items.length, scrollRows: 0 };
        }
        const page = await invoke('library_browse', {
            collection, limit: CHUNK, direction: 'forward',
        });
        if (ownVersion !== version) return { stale: true };
        items = page.items;
        previousCursor = page.previous_cursor;
        nextCursor = page.next_cursor;
        hasPrevious = false;
        hasNext = page.has_more;
        return { items, searchCount: null, scrollRows: 0 };
    }

    async function extend(direction, collection) {
        const cursor = direction === 'forward' ? nextCursor : previousCursor;
        if (loading || !cursor) return null;
        loading = true;
        try {
            const page = await invoke('library_browse', {
                collection, limit: CHUNK, cursor, direction,
            });
            return direction === 'forward' ? append(page) : prepend(page);
        } finally {
            loading = false;
        }
    }

    function append(page) {
        items.push(...unique(page.items));
        nextCursor = page.next_cursor;
        hasNext = page.has_more;
        let removed = 0;
        if (items.length > MAX_WINDOW) {
            removed = items.length - MAX_WINDOW;
            items.splice(0, removed);
            hasPrevious = true;
            previousCursor = cursorFor(items[0]);
        }
        return { items, scrollRows: -removed };
    }

    function prepend(page) {
        const additions = unique(page.items);
        items.unshift(...additions);
        previousCursor = page.previous_cursor;
        hasPrevious = page.has_more;
        if (items.length > MAX_WINDOW) {
            items.splice(MAX_WINDOW);
            hasNext = true;
            nextCursor = cursorFor(items.at(-1));
        }
        return { items, scrollRows: additions.length };
    }

    function can(direction) {
        return direction === 'forward' ? hasNext : hasPrevious;
    }

    function current() {
        return items;
    }

    function unique(values) {
        const known = new Set(items.map(item => item.path));
        return values.filter(item => !known.has(item.path));
    }

    return { fresh, extend, can, current };
}

function cursorFor(item) {
    return { file_name: item.file_name, path_key: item.path_key };
}
