/** Menú contextual específico de resultados de Biblioteca. */
import { placeMenu } from '../util/menuPosition.js';
import { alertIpcError } from './ipcError.js';
import { dispatchLibraryAction } from './libraryActionDispatch.js';

let cleanup = null;
let returnFocus = null;

export function showLibraryContextMenu(x, y, selection, onEdited) {
    const menu = document.getElementById('library-context-menu');
    returnFocus = document.activeElement;
    const single = selection.length === 1;
    setDisabled('library-menu-live', !single);
    setDisabled('library-menu-preview', !single);
    setDisabled('library-menu-editor', !single);
    setDisabled('library-menu-add-player', !selection.length);
    if (cleanup) cleanup();
    cleanup = wire(selection, single, onEdited);
    placeMenu(menu, x, y);
    menu.addEventListener('keydown', navigateMenu);
    firstEnabled(menu)?.focus();
    setTimeout(() => document.addEventListener('click', hideOutside), 10);
}

function wire(selection, single, onEdited) {
    const live = document.getElementById('library-menu-live');
    const preview = document.getElementById('library-menu-preview');
    const add = document.getElementById('library-menu-add-player');
    const editor = document.getElementById('library-menu-editor');
    const onLive = () => single && action(
        () => dispatchLibraryAction('live', selection, onEdited));
    const onPreview = () => single && action(
        () => dispatchLibraryAction('preview', selection, onEdited));
    const onAdd = () => action(
        () => dispatchLibraryAction('player', selection, onEdited));
    const onEditor = () => single && action(
        () => dispatchLibraryAction('editor', selection, onEdited));
    live.addEventListener('click', onLive);
    preview.addEventListener('click', onPreview);
    add.addEventListener('click', onAdd);
    editor.addEventListener('click', onEditor);
    return () => {
        live.removeEventListener('click', onLive);
        preview.removeEventListener('click', onPreview);
        add.removeEventListener('click', onAdd);
        editor.removeEventListener('click', onEditor);
    };
}

function action(callback) {
    hide();
    Promise.resolve(callback()).catch(alertIpcError);
}

function setDisabled(id, disabled) {
    document.getElementById(id).setAttribute('aria-disabled', String(disabled));
}

function hide() {
    const menu = document.getElementById('library-context-menu');
    menu.classList.add('hidden');
    menu.removeEventListener('keydown', navigateMenu);
    if (menu.contains(document.activeElement)) returnFocus?.focus();
    document.removeEventListener('click', hideOutside);
}

function navigateMenu(event) {
    const enabled = [...event.currentTarget.querySelectorAll('[role="menuitem"]')]
        .filter(item => item.getAttribute('aria-disabled') !== 'true');
    const index = enabled.indexOf(document.activeElement);
    if (event.key === 'Escape') return hide();
    if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        return document.activeElement?.click();
    }
    const delta = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0;
    if (!delta || !enabled.length) return;
    event.preventDefault();
    enabled[(index + delta + enabled.length) % enabled.length].focus();
}

function firstEnabled(menu) {
    return [...menu.querySelectorAll('[role="menuitem"]')]
        .find(item => item.getAttribute('aria-disabled') !== 'true');
}

function hideOutside(event) {
    if (!event.target.closest('#library-context-menu')) hide();
}
