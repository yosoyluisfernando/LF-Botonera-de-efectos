/** Presenta en todas las superficies el progreso emitido por el motor Rust. */
import { listen } from '../bridge/api.js';
import { t } from '../util/i18n.js';

let wired = false;

export function initLibraryIndexProgress() {
    if (wired) return;
    wired = true;
    listen('library-index-progress', event => paintProgress(event.payload))
        .catch(console.error);
}

export function setLibraryIndexStarting() {
    paint(t('library.index_starting'));
}

export function setLibraryIndexFinished(count) {
    paint(t('library.index_complete').replace('{count}', count));
}

export function setLibraryIndexError(error) {
    paint(t('library.index_failed').replace('{error}', String(error)), true);
}

export function clearLibraryIndexProgress() {
    document.querySelectorAll('[data-library-progress]').forEach(element => {
        element.textContent = '';
        element.classList.add('hidden');
        element.classList.remove('error');
    });
}

function paintProgress(progress) {
    const phase = progress?.phase ?? 'discovering';
    const total = Number(progress?.total ?? 0);
    const processed = Number(progress?.processed ?? 0);
    const key = phase === 'catalog_ready'
        ? 'library.index_catalog_ready'
        : phase === 'enriching'
        ? 'library.index_enriching'
        : phase === 'cataloging'
            ? total > 0
                ? 'library.index_cataloging'
                : 'library.index_cataloging_progress'
            : 'library.index_discovering';
    const message = t(key)
        .replace('{processed}', processed)
        .replace('{total}', total);
    paint(message);
}

function paint(message, error = false) {
    document.querySelectorAll('[data-library-progress]').forEach(element => {
        element.textContent = message;
        element.classList.remove('hidden');
        element.classList.toggle('error', error);
    });
}
