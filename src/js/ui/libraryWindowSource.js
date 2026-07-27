/** Fuente posicionable: permite saltar con la barra a cualquier punto del catálogo. */
import { invoke } from '../bridge/api.js';

const WINDOW = 180;
const LEAD = 50;

export function createLibraryWindowSource() {
    let items = [];
    let offset = 0;
    let total = 0;
    let scope = {};
    let query = '';
    let version = 0;

    async function fresh(nextQuery, nextScope) {
        query = nextQuery;
        scope = nextScope;
        const ownVersion = ++version;
        if (query) {
            const found = await invoke('library_search', {
                query,
                collection: scope.collection ?? null,
                limit: 500,
            });
            if (ownVersion !== version) return { stale: true };
            items = found;
            offset = 0;
            total = found.length;
            return result();
        }
        return loadAt(0, ownVersion);
    }

    async function at(index) {
        if (query || covers(index)) return null;
        return loadAt(Math.max(0, index - LEAD), ++version);
    }

    async function loadAt(requestedOffset, ownVersion) {
        const page = await invoke('library_browse_window', {
            collection: scope.collection ?? null,
            rootId: scope.rootId,
            relativePrefix: scope.relativePrefix,
            offset: requestedOffset,
            limit: WINDOW,
        });
        if (ownVersion !== version) return { stale: true };
        items = page.items;
        offset = page.offset;
        total = page.total;
        return result();
    }

    function covers(index) {
        const lower = offset === 0 ? 0 : offset + LEAD;
        const end = offset + items.length;
        const upper = end >= total ? total : end - LEAD;
        return index >= lower && index < upper;
    }

    function result() {
        return { items, offset, total, stale: false };
    }

    return { fresh, at };
}
