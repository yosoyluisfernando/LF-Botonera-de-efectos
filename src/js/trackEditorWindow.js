/**
 * Archivo: trackEditorWindow.js
 * Proposito: coordinacion de ventana flotante/modal del editor de pista.
 */
import { invoke } from './api.js';
import { t } from './i18n.js';

export async function openPreferred(path, name, onSaved, openModal) {
    if (!path) return;
    const cfg = await invoke('get_config').catch(() => ({}));
    if (cfg.editor_mode === 'window' && !document.body.classList.contains('editor-window-mode')) {
        if (openWindow(path, name || '')) return;
    }
    return openModal(path, name, onSaved);
}

export function syncButton() {
    const btn = document.getElementById('te-popout');
    const windowMode = document.body.classList.contains('editor-window-mode');
    btn.textContent = windowMode ? '⇱' : '⤢';
    btn.title = t(windowMode ? 'track_editor.dock' : 'track_editor.popout');
}

export async function popOut(path, name, onClose) {
    if (openWindow(path, name)) {
        await invoke('set_editor_mode', { mode: 'window' }).catch(console.error);
        onClose();
    }
}

export async function dockIn(path, name, onClose) {
    await invoke('set_editor_mode', { mode: 'modal' }).catch(console.error);
    const payload = { path, name };
    const events = window.__TAURI__?.event;
    if (events?.emitTo) await events.emitTo('main', 'track-editor-dock', payload);
    else if (events?.emit) await events.emit('track-editor-dock', payload);
    window.__lfDockingTrackEditor = true;
    onClose();
    window.__TAURI__?.window?.getCurrentWindow?.().close();
}

function openWindow(path, name) {
    const url = `index.html?editor=${encodeURIComponent(path)}&name=${encodeURIComponent(name)}`;
    try {
        new window.__TAURI__.webviewWindow.WebviewWindow('track-editor', {
            url, title: t('track_editor.title'), width: 1100, height: 720, resizable: true,
        });
        return true;
    } catch (e) {
        console.error('Error al abrir el editor en ventana:', e);
        return false;
    }
}
