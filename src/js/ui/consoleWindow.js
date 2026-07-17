/**
 * Archivo: consoleWindow.js
 * Propósito: abrir la consola donde toque — ventana flotante o modal — y
 * recordar cuál se eligió. Mismo patrón que el editor de pistas: la preferencia
 * la guarda Rust (`console_mode`), aquí solo se obedece.
 */
import { invoke, listen } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { renderConsole, clearConsole, updateConsoleTick } from './consoleView.js';

const MODAL = 'console-modal';

/** Abre por donde diga la preferencia. En la propia ventana, siempre modal. */
export async function openPreferred() {
    const cfg = await invoke('get_config').catch(() => ({}));
    if (cfg.console_mode !== 'modal' && !_isWindow() && _openWindow()) return;
    await openModal();
}

export async function openModal() {
    await renderConsole();
    _syncPopout();
    document.getElementById(MODAL)?.classList.remove('hidden');
}

export function closeModal() {
    document.getElementById(MODAL)?.classList.add('hidden');
    clearConsole();
}

/**
 * La consola vive sola en su ventana: no hay botonera detrás que oscurecer.
 *
 * Se cablea su propio tick, que en la ventana principal reparte `runtimeEvents`
 * junto al resto. Llega porque la ventana está en `capabilities/default.json`:
 * sin eso `listen` falla en silencio y los vúmetros no se mueven.
 */
export function initWindowMode() {
    document.body.classList.add('console-window-mode');
    document.getElementById(MODAL)?.classList.remove('hidden');
    document.getElementById('close-console')?.addEventListener('click', () => {
        window.__TAURI__?.window?.getCurrentWindow?.().close();
    });
    document.getElementById('console-popout')?.addEventListener('click', dockIn);
    renderConsole();
    _syncPopout();
    listen('audio-tick', e => updateConsoleTick(e.payload ?? {})).catch(console.error);
}

export function wire() {
    document.getElementById('btn-console')?.addEventListener('click', openPreferred);
    document.getElementById('close-console')?.addEventListener('click', closeModal);
    document.getElementById('console-popout')?.addEventListener('click', popOut);
}

async function popOut() {
    if (!_openWindow()) return;
    await invoke('set_console_mode', { mode: 'window' }).catch(console.error);
    closeModal();
}

/** Vuelve al modal: se guarda la preferencia y se cierra la ventana. */
async function dockIn() {
    await invoke('set_console_mode', { mode: 'modal' }).catch(console.error);
    const events = window.__TAURI__?.event;
    if (events?.emitTo) await events.emitTo('main', 'console-dock', {});
    else if (events?.emit) await events.emit('console-dock', {});
    window.__TAURI__?.window?.getCurrentWindow?.().close();
}

function _openWindow() {
    try {
        new window.__TAURI__.webviewWindow.WebviewWindow('audio-console', {
            url: 'index.html?console=1',
            title: t('console.title'),
            width: 980,
            height: 680,
            center: true,
            resizable: true,
        });
        return true;
    } catch (e) {
        console.error('Error al abrir la consola en ventana:', e);
        return false;
    }
}

function _isWindow() {
    return document.body.classList.contains('console-window-mode');
}

/** El mismo botón sirve para sacar y para meter: cambia lo que promete. */
function _syncPopout() {
    const btn = document.getElementById('console-popout');
    if (!btn) return;
    btn.textContent = '⏏️';
    btn.title = t(_isWindow() ? 'console.dock' : 'console.popout');
}
