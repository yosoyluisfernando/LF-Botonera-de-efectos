/** Retiro reversible, restauración y plazo de conservación de raíces. */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

let retired = [];
let retentionDays = 30;
let pendingRoot = null;
let previousFocus = null;
let onChanged = null;

export function initLibraryRootRetention(options = {}) {
    onChanged = options.onChanged ?? null;
    document.getElementById('library-retention-save').addEventListener('click', saveRetention);
    document.getElementById('library-remove-root-confirm').addEventListener('click', confirmRemove);
    document.getElementById('library-remove-root-cancel').addEventListener('click', closeConfirm);
    document.getElementById('library-remove-root-modal').addEventListener('keydown', event => {
        if (event.key === 'Escape' && !event.repeat) closeConfirm();
    });
}

export async function loadLibraryRootRetention() {
    const [settings, roots] = await Promise.all([
        invoke('library_get_retention'),
        invoke('library_list_retired_roots'),
    ]);
    retentionDays = settings.retention_days;
    retired = roots;
    document.getElementById('library-retention-days').value = String(retentionDays);
    renderRetired();
}

export function configuredRemoveButton(root) {
    const remove = document.createElement('button');
    remove.type = 'button';
    remove.className = 'library-root-remove';
    remove.textContent = '×';
    remove.setAttribute('aria-label',
        t('library.remove_configured').replace('{path}', root.path));
    remove.addEventListener('click', () => openConfirm(root, remove));
    return remove;
}

export function setLibraryRetentionDisabled(disabled) {
    document.getElementById('library-retention-days').disabled = disabled;
    document.getElementById('library-retention-save').disabled = disabled;
    document.querySelectorAll('.library-root-remove,.library-root-restore')
        .forEach(control => { control.disabled = disabled; });
}

async function saveRetention() {
    const input = document.getElementById('library-retention-days');
    const days = Number(input.value);
    if (!Number.isInteger(days) || days < 30 || days > 365) {
        input.focus();
        return announce(t('library.retention_invalid'));
    }
    try {
        const settings = await invoke('library_set_retention', { retentionDays: days });
        retentionDays = settings.retention_days;
        input.value = String(retentionDays);
        announce(t('library.retention_saved').replace('{days}', String(retentionDays)));
    } catch (error) {
        announce(t('library.retention_save_failed').replace('{error}', String(error)));
    }
}

function openConfirm(root, trigger) {
    pendingRoot = root;
    previousFocus = trigger;
    document.getElementById('library-remove-root-path').textContent = root.path;
    document.getElementById('library-remove-root-description').textContent =
        t('library.remove_root_description').replace('{days}', String(retentionDays));
    document.getElementById('library-remove-root-modal').classList.remove('hidden');
    document.getElementById('library-remove-root-cancel').focus();
}

async function confirmRemove() {
    if (!pendingRoot) return;
    setConfirmDisabled(true);
    try {
        const result = await invoke('library_remove_root', { rootId: pendingRoot.id });
        closeConfirm(false);
        await loadLibraryRootRetention();
        announce(t('library.root_retired').replace('{date}', formatDate(result.purge_after)));
        await onChanged?.();
        document.querySelector('#library-retired-roots .library-root-restore')?.focus();
    } catch (error) {
        announce(t('library.root_retire_failed').replace('{error}', String(error)));
        setConfirmDisabled(false);
    }
}

function closeConfirm(restoreFocus = true) {
    pendingRoot = null;
    document.getElementById('library-remove-root-modal').classList.add('hidden');
    setConfirmDisabled(false);
    if (restoreFocus) previousFocus?.focus();
    previousFocus = null;
}

function renderRetired() {
    const target = document.getElementById('library-retired-roots');
    target.replaceChildren();
    if (!retired.length) {
        const empty = document.createElement('p');
        empty.className = 'library-root-empty';
        empty.textContent = t('library.no_retired_roots');
        return target.appendChild(empty);
    }
    retired.forEach(root => target.appendChild(retiredRow(root)));
}

function retiredRow(root) {
    const row = document.createElement('div');
    row.className = 'library-root-row';
    const path = document.createElement('span');
    path.className = 'library-root-path';
    path.textContent = root.path;
    path.title = root.path;
    const state = document.createElement('span');
    state.className = 'library-root-state';
    state.textContent = t('library.purge_on').replace('{date}', formatDate(root.purge_after));
    const restore = document.createElement('button');
    restore.type = 'button';
    restore.className = 'btn-dark library-root-restore';
    restore.textContent = t('library.restore_root');
    restore.addEventListener('click', () => restoreRoot(root, restore));
    row.append(path, state, restore);
    return row;
}

async function restoreRoot(root, control) {
    control.disabled = true;
    try {
        await invoke('library_restore_root', { rootId: root.id });
        await loadLibraryRootRetention();
        announce(t('library.root_restored'));
        await onChanged?.();
    } catch (error) {
        control.disabled = false;
        announce(t('library.root_restore_failed').replace('{error}', String(error)));
    }
}

function setConfirmDisabled(disabled) {
    document.getElementById('library-remove-root-confirm').disabled = disabled;
    document.getElementById('library-remove-root-cancel').disabled = disabled;
}

function announce(message) {
    document.getElementById('library-roots-notice').textContent = message;
}

function formatDate(epoch) {
    return new Date(epoch * 1000).toLocaleDateString();
}
