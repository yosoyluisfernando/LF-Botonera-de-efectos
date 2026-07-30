/** Modal transaccional para preparar raíces y comenzar su indexación. */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import {
    clearLibraryIndexProgress, initLibraryIndexProgress, setLibraryIndexError,
    setLibraryIndexFinished, setLibraryIndexStarting,
} from './libraryIndexProgress.js';
import {
    configuredRemoveButton, initLibraryRootRetention, loadLibraryRootRetention,
    setLibraryRetentionDisabled,
} from './libraryRootRetention.js';

let configured = [];
let drafts = [];
let running = false;
let wired = false;
let onComplete = null;

export function initLibraryRootsModal(options = {}) {
    onComplete = options.onComplete ?? null;
    if (wired) return;
    wired = true;
    initLibraryIndexProgress();
    initLibraryRootRetention({ onChanged: refreshAfterRootChange });
    document.getElementById('library-roots-open').addEventListener('click', openModal);
    document.getElementById('library-add-music')
        .addEventListener('click', () => pickFolder('music'));
    document.getElementById('library-add-effects')
        .addEventListener('click', () => pickFolder('effects'));
    document.getElementById('library-roots-start').addEventListener('click', startIndexing);
    document.getElementById('library-roots-hide').addEventListener('click', hideModal);
}

async function openModal() {
    if (!running && !drafts.length) {
        notice('');
        clearLibraryIndexProgress();
    }
    render();
    document.getElementById('library-roots-modal').classList.remove('hidden');
    setControlsDisabled(running);
    (running
        ? document.getElementById('library-roots-hide')
        : document.getElementById('library-add-music')).focus();
    [configured] = await Promise.all([
        invoke('library_list_roots'),
        loadLibraryRootRetention(),
    ]);
    render();
}

async function pickFolder(collection) {
    try {
        const selected = await invoke('pick_named_folder');
        if (isAlreadyListed(selected.path)) {
            return notice(t('library.root_already_listed'));
        }
        drafts.push({ path: selected.path, collection });
        notice(t('library.pending_not_saved'));
        render();
    } catch (_) {
        // Cerrar el selector nativo no modifica el borrador ni muestra un error.
    }
}

async function startIndexing() {
    if (!drafts.length || running) return;
    running = true;
    setControlsDisabled(true);
    setLibraryIndexStarting();
    notice(t('library.saving_roots'));
    try {
        const outcomes = await invoke('library_add_roots', { roots: drafts });
        const summary = outcomeSummary(outcomes);
        drafts = [];
        configured = await invoke('library_list_roots');
        render();
        if (summary) notice(summary);
        await invoke('library_sync_all');
        const status = await invoke('library_status');
        setLibraryIndexFinished(status.present);
        await onComplete?.();
        configured = await invoke('library_list_roots');
        render();
    } catch (error) {
        setLibraryIndexError(error);
        notice(t('library.index_failed').replace('{error}', String(error)));
    } finally {
        running = false;
        setControlsDisabled(false);
    }
}

function render() {
    renderCollection('music');
    renderCollection('effects');
    document.getElementById('library-roots-start').disabled = !drafts.length || running;
}

function renderCollection(collection) {
    const target = document.getElementById(`library-${collection}-roots`);
    const roots = [
        ...configured.filter(root => root.collection === collection)
            .map(root => ({ ...root, pending: false })),
        ...drafts.filter(root => root.collection === collection)
            .map((root, draftIndex) => ({ ...root, pending: true, draftIndex })),
    ];
    target.replaceChildren();
    if (!roots.length) {
        const empty = document.createElement('p');
        empty.className = 'library-root-empty';
        empty.textContent = t('library.no_roots');
        return target.appendChild(empty);
    }
    roots.forEach(root => target.appendChild(rootRow(root)));
}

function rootRow(root) {
    const row = document.createElement('div');
    row.className = 'library-root-row';
    const path = document.createElement('span');
    path.className = 'library-root-path';
    path.textContent = root.path;
    path.title = root.path;
    const state = document.createElement('span');
    state.className = 'library-root-state';
    state.textContent = t(root.pending ? 'library.pending' : 'library.configured');
    row.append(path, state);
    if (root.pending) {
        const remove = document.createElement('button');
        remove.type = 'button';
        remove.className = 'library-root-remove';
        remove.textContent = '×';
        remove.setAttribute('aria-label', t('library.remove_pending'));
        remove.addEventListener('click', () => {
            const index = drafts.findIndex(value =>
                value.path === root.path && value.collection === root.collection);
            if (index >= 0) drafts.splice(index, 1);
            render();
        });
        row.appendChild(remove);
    } else {
        const remove = configuredRemoveButton(root);
        remove.disabled = running;
        row.appendChild(remove);
    }
    return row;
}

function setControlsDisabled(disabled) {
    ['library-add-music', 'library-add-effects']
        .forEach(id => { document.getElementById(id).disabled = disabled; });
    document.getElementById('library-roots-start').disabled = disabled || !drafts.length;
    setLibraryRetentionDisabled(disabled);
}

async function refreshAfterRootChange() {
    configured = await invoke('library_list_roots');
    render();
    await onComplete?.();
}

function hideModal() {
    document.getElementById('library-roots-modal').classList.add('hidden');
    document.getElementById('library-roots-open').focus();
}

function isAlreadyListed(path) {
    const key = path.replaceAll('\\', '/').toLocaleLowerCase();
    return [...configured, ...drafts].some(root =>
        root.path.replaceAll('\\', '/').toLocaleLowerCase() === key);
}

function outcomeSummary(outcomes) {
    const merged = outcomes.some(value => value.Added?.merged?.length);
    const covered = outcomes.some(value => value.AlreadyCovered);
    const changed = outcomes.some(value => value.Reclassified);
    return [
        merged ? t('library.roots_merged') : '',
        covered ? t('library.roots_covered') : '',
        changed ? t('library.roots_reclassified') : '',
    ].filter(Boolean).join(' ');
}

function notice(message) {
    document.getElementById('library-roots-notice').textContent = message;
}
