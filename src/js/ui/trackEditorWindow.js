/**
 * Archivo: trackEditorWindow.js
 * Proposito: coordinacion de ventana flotante/modal del editor de pista.
 */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

export async function openPreferred(path, name, onSaved, openModal) {
    if (!path) return;
    const cfg = await invoke('get_config').catch(() => ({}));
    if (cfg.editor_mode === 'window' && !document.body.classList.contains('editor-window-mode')) {
        if (await openWindow(path, name || '')) return;
    }
    return openModal(path, name, onSaved);
}

export function syncButton() {
    const btn = document.getElementById('te-popout');
    const windowMode = document.body.classList.contains('editor-window-mode');
    btn.textContent = '\u23CF\uFE0F';
    btn.title = t(windowMode ? 'track_editor.dock' : 'track_editor.popout');
}

export async function popOut(path, name, onClose, zoom = 1) {
    if (await openWindow(path, name, zoom)) {
        await invoke('set_editor_mode', { mode: 'window' }).catch(console.error);
        onClose();
    }
}

export async function dockIn(path, name, onClose, zoom = 1) {
    await invoke('set_editor_mode', { mode: 'modal' }).catch(console.error);
    const payload = { path, name, zoom };
    const events = window.__TAURI__?.event;
    if (events?.emitTo) await events.emitTo('main', 'track-editor-dock', payload);
    else if (events?.emit) await events.emit('track-editor-dock', payload);
    window.__lfDockingTrackEditor = true;
    onClose();
    window.__TAURI__?.window?.getCurrentWindow?.().close();
}

async function openWindow(path, name, zoom = 1) {
    const url = `index.html?editor=${encodeURIComponent(path)}&name=${encodeURIComponent(name)}&zoom=${encodeURIComponent(zoom)}`;
    try {
        const api = window.__TAURI__;
        const WindowClass = api.webviewWindow.WebviewWindow;
        const existing = await WindowClass.getByLabel('track-editor');
        if (existing) {
            await api.event.emitTo('track-editor', 'track-editor-open', { path, name, zoom });
            await existing.unminimize();
            await existing.show();
            await existing.setFocus();
            return true;
        }
        new WindowClass('track-editor', {
            url, title: t('track_editor.title'), width: 1100, height: 720, center: true, resizable: true,
        });
        return true;
    } catch (e) {
        console.error('Error al abrir el editor en ventana:', e);
        return false;
    }
}
