/** Orquesta carga, guardado y renombrado del editor de metadatos. */
import { emit, invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { alertIpcError } from './ipcError.js';
import { renderMetadataFields } from './libraryMetadataFields.js';
import { createMetadataTags } from './libraryMetadataTags.js';
import { ensureMetadataView, setMetadataBusy } from './libraryMetadataView.js';

let modal = null;
let current = null;
let returnFocus = null;

export async function openLibraryMetadataEditor(selection, onSaved) {
    const paths = selection.map(item => item.path).filter(Boolean);
    if (!paths.length) return;
    returnFocus = document.activeElement;
    modal = ensureMetadataView({ close, save, rename });
    current = { items: [], onSaved, busy: true };
    modal.classList.remove('hidden');
    modal.querySelector('#lm-fields').replaceChildren();
    modal.querySelector('.lm-tags-list').replaceChildren();
    setMetadataBusy(modal, true, t('metadata_editor.loading'));
    try {
        const response = await invoke('library_metadata_get', { paths });
        const items = response?.items ?? [];
        if (!items.length) throw 'library_track_not_found';
        current = {
            items,
            onSaved,
            fields: renderMetadataFields(modal.querySelector('#lm-fields'), items),
            tags: createMetadataTags(modal.querySelector('.lm-tags'), items,
                response.suggestions ?? []),
            busy: false,
        };
        paintHeader(items);
        prepareRename(items);
        prepareFileWriting(items);
        modal.querySelector('#lm-tags-help').textContent = items.length > 1
            ? t('metadata_editor.common_tags_help') : t('metadata_editor.tags_help');
        setMetadataBusy(modal, false);
        if (current.fields.mixed) current.tags.focus();
        else current.fields.focus();
    } catch (error) {
        if (current) current.busy = false;
        close();
        await alertIpcError(error);
    }
}

async function save() {
    if (!current || current.busy) return;
    const fields = current.fields.fields();
    const { addTags, removeTags } = current.tags.changes();
    const writeToFile = modal.querySelector('#lm-write-file').checked;
    if (!Object.keys(fields).length && !addTags.length && !removeTags.length
        && !writeToFile) return close();
    setBusy(true, t('metadata_editor.saving'));
    try {
        const paths = current.items.map(item => item.path);
        await invoke('library_metadata_save', {
            paths, fields, addTags, removeTags, writeToFile,
        });
        await notifyChanged({ paths });
        setBusy(false);
        close();
    } catch (error) {
        setBusy(false);
        await alertIpcError(error);
    }
}

async function rename() {
    if (!current || current.busy || current.items.length !== 1) return;
    const item = current.items[0];
    const input = modal.querySelector('#lm-new-file');
    const newFileName = input.value.trim();
    if (!newFileName || newFileName === item.file_name) return;
    setBusy(true, t('metadata_editor.renaming'));
    try {
        const result = await invoke('library_rename_file', {
            path: item.path, newFileName,
        });
        const oldPath = item.path;
        const newPath = result.new_path ?? result.newPath;
        current.items[0] = result.item ?? {
            ...item, path: newPath, file_name: newFileName,
        };
        input.value = current.items[0].file_name;
        modal.querySelector('#lm-current-file').textContent = current.items[0].file_name;
        modal.querySelector('#lm-rename-enabled').checked = false;
        modal.querySelector('#lm-rename-controls').classList.add('hidden');
        await notifyChanged({ paths: [oldPath], newPath: current.items[0].path });
        setBusy(false, t('metadata_editor.rename_complete'));
    } catch (error) {
        setBusy(false);
        await alertIpcError(error);
    }
}

function paintHeader(items) {
    const collections = new Set(items.map(item => item.collection));
    const kind = collections.size === 1
        ? t(`library.${items[0].collection}`) : t('metadata_editor.mixed');
    modal.querySelector('#lm-kind').textContent = kind;
    modal.querySelector('#lm-count').textContent = items.length > 1
        ? t('metadata_editor.selected_count').replace('{count}', items.length) : '';
}

function prepareRename(items) {
    const section = modal.querySelector('#lm-rename-section');
    const single = items.length === 1;
    section.classList.toggle('hidden', !single);
    modal.querySelector('#lm-rename-enabled').checked = false;
    modal.querySelector('#lm-rename-controls').classList.add('hidden');
    if (single) modal.querySelector('#lm-new-file').value = items[0].file_name;
}

function prepareFileWriting(items) {
    const enabled = items.length === 1 && items[0].collection === 'music';
    modal.querySelector('#lm-write-section').classList.toggle('hidden', !enabled);
    modal.querySelector('#lm-write-file').checked = false;
}

async function notifyChanged(payload) {
    await emit('library-metadata-changed', payload);
    await current?.onSaved?.();
}

function setBusy(busy, message = '') {
    if (current) current.busy = busy;
    setMetadataBusy(modal, busy, message);
}

function close() {
    if (!modal || current?.busy) return;
    modal.classList.add('hidden');
    current = null;
    returnFocus?.focus();
    returnFocus = null;
}
