import { initVolumeSync } from './volumeSync.js';
import { invoke, listen } from '../bridge/api.js';
import { isAndroid } from '../bridge/platform.js';
import { t } from '../util/i18n.js';
import { isDocked, setDockWidth, wireDockResize } from './consoleDock.js';
import { wireConsolePlacementMenu } from './consolePlacementMenu.js';
import { renderConsole, clearConsole, updateConsoleTick, refreshConsoleOutputs } from './consoleView.js';

const WINDOW_LABEL = 'audio-console';
let insideMode;
let openingWindow;
const byId = id => document.getElementById(id);

export async function openPreferred() {
    if (isAndroid()) return;
    const cfg = await invoke('get_config').catch(() => ({}));
    const mode = cfg.console_mode;
    if (mode !== 'modal' && !isDocked(mode) && await openWindow()) {
        closeInside();
        return;
    }
    await showInside(isDocked(mode) ? mode : 'modal', cfg.console?.dock_width);
}

async function showInside(mode, width) {
    const modal = byId('console-modal');
    const dock = byId('console-dock');
    const view = document.querySelector('.console-content');
    const alreadyOpen = Boolean(insideMode);
    const sameSide = insideMode === mode;
    insideMode = mode;
    if (width !== undefined && !sameSide) setDockWidth(dock, byId('console-dock-width'), width);
    if (isDocked(mode)) {
        dock.dataset.side = mode;
        if (view.parentElement !== dock) dock.append(view);
        modal.classList.add('hidden');
        dock.classList.remove('hidden');
    } else {
        if (view.parentElement !== modal) modal.append(view);
        dock.classList.add('hidden');
        modal.classList.remove('hidden');
    }
    byId('console-placement').value = mode;
    if (!alreadyOpen) {
        await renderConsole();
        if (!insideMode) clearConsole();
    }
}

function closeInside() {
    if (!insideMode) return;
    insideMode = undefined;
    byId('console-modal').classList.add('hidden');
    byId('console-dock').classList.add('hidden');
    clearConsole();
}

export function hideDockedConsole() {
    if (isDocked(insideMode)) closeInside();
}

export function initWindowMode() {
    document.body.classList.add('console-window-mode');
    initVolumeSync().catch(console.error);
    byId('console-modal').classList.remove('hidden');
    renderConsole();
    listen('tab-outputs-changed', () => refreshConsoleOutputs()).catch(console.error);
    listen('meter-tick', e => updateConsoleTick(e.payload ?? {})).catch(console.error);
}

export function wire() {
    if (isAndroid()) return;
    const trigger = byId('btn-console');
    trigger.addEventListener('click', () => {
        if (isDocked(insideMode)) hideDockedConsole();
        else return openPreferred();
    });
    wireConsolePlacementMenu(trigger, byId('console-placement-menu'), changePlacement);
    byId('close-console').addEventListener('click', closeInside);
    const select = byId('console-placement');
    select.addEventListener('change', async () => {
        select.disabled = true;
        try {
            await changePlacement(select.value);
        } catch (error) {
            console.error(error);
        } finally {
            select.value = insideMode ?? 'modal';
            select.disabled = false;
            if (insideMode === 'modal') select.focus();
        }
    });
    wireDockResize(byId('console-dock'), byId('console-dock-width'));
}

async function changePlacement(mode) {
    if (mode === 'window' && !await openWindow()) return;
    await invoke('set_console_mode', { mode });
    if (mode === 'window') {
        closeInside();
        return;
    }
    if (!insideMode) await closeNativeConsole().catch(console.error);
    const cfg = await invoke('get_config').catch(() => ({}));
    await showInside(mode, cfg.console?.dock_width);
    byId(mode === 'modal' ? 'console-placement' : 'btn-console').focus();
}

async function closeNativeConsole() {
    await openingWindow;
    const WindowClass = window.__TAURI__?.webviewWindow?.WebviewWindow;
    const existing = await WindowClass?.getByLabel(WINDOW_LABEL);
    await existing?.close();
}

function openWindow() {
    openingWindow ??= openNativeConsole().then(() => true).catch(error => {
        console.error('Error abriendo la consola de audio:', error);
        return false;
    }).finally(() => {
        openingWindow = undefined;
    });
    return openingWindow;
}

async function openNativeConsole() {
    const WindowClass = window.__TAURI__.webviewWindow.WebviewWindow;
    const existing = await WindowClass.getByLabel(WINDOW_LABEL);
    if (existing) {
        await existing.unminimize();
        await existing.show();
        await existing.setFocus();
        return;
    }
    const native = new WindowClass(WINDOW_LABEL, {
        url: 'index.html?console=1',
        title: t('console.title'),
        width: 980,
        height: 680,
        center: true,
        resizable: true,
    });
    await new Promise((resolve, reject) => {
        native.once('tauri://created', resolve).catch(reject);
        native.once('tauri://error', e => reject(e.payload)).catch(reject);
    });
}
