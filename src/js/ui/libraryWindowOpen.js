/** Abre o enfoca la única ventana Biblioteca. */
import { t } from '../util/i18n.js';

export function initLibraryWindowOpen() {
    document.getElementById('btn-library')?.addEventListener('click', openLibraryWindow);
}

export async function openLibraryWindow() {
    try {
        const api = window.__TAURI__;
        const WindowClass = api.webviewWindow.WebviewWindow;
        const existing = await WindowClass.getByLabel('library');
        if (existing) {
            await existing.unminimize();
            await existing.show();
            await existing.setFocus();
            return true;
        }
        new WindowClass('library', {
            url: 'library.html',
            title: t('library.window_title'),
            width: 1180,
            height: 760,
            minWidth: 880,
            minHeight: 560,
            center: true,
            resizable: true,
        });
        return true;
    } catch (error) {
        console.error('Error abriendo Biblioteca:', error);
        return false;
    }
}
