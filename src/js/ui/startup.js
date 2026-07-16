/**
 * Archivo: startup.js
 * Propósito: Orquesta el arranque de la UI en etapas verificables.
 * La UI solo se muestra cuando Rust entregó configuración y cuadrícula.
 */

import { invoke, listen, waitForTauri } from '../bridge/api.js';
import { loadLanguage, t } from '../util/i18n.js';
import { applyTheme } from './theme.js';
import { initWizard } from './wizard.js';
import { initTabs, updateTabs } from './tabs.js';
import { initProfiles, updateProfiles } from './profiles.js';
import { initShortcuts, updateShortcuts } from './shortcuts.js';
import { initGrid, drawGrid } from './grid.js';
import { initGridDnd } from './gridDnd.js';
import { initFileDrop } from './fileDrop.js';
import { initPlayerDnd } from './playerDnd.js';
import { initPlayerDrop } from './playerDrop.js';
import { initTabDnd, updateTabDnd } from './tabDnd.js';
import { initBottomBar, refreshBottomBar } from './bottomBar.js';
import { initSettingsModal } from './settingsModal.js';
import { initMapping } from './mapping.js';
import { initUpdateNotifier, startUpdateChecks } from './updateNotifier.js';
import { initColorPicker } from './colorPalette.js';
import { initButtonSelection } from './buttonSelection.js';
import { initNumberInputs } from '../util/numberInputs.js';
import { maybeShowPreloadDialog } from './preloadDialog.js';
import { checkAudioDevicesOnStartup } from './audioDeviceRecovery.js';
import { initStartupPrompts, runStartupPrompts } from './startupPrompts.js';
import { initFixedPanel, drawFixedPanel, initialFixedPanel } from './fixedPanel.js';
import { wire as wireConsole, initWindowMode as initConsoleWindow } from './consoleWindow.js';
import { updateConsoleTick } from './consoleView.js';
import { wireRuntimeEvents } from './runtimeEvents.js';

let _closeWired = false;
/** Punto único de arranque llamado desde main.js al cargar el DOM. */
export async function startApp() {
    try {
        await waitForTauri();
        // Ventanas "pop-out": si la URL lo pide, arranca SOLO esa herramienta.
        const params = new URLSearchParams(location.search);
        const editorPath = params.get('editor');
        if (editorPath) { await _startEditorWindow(editorPath); return; }
        if (params.get('console')) { await _startConsoleWindow(); return; }
        initColorPicker();
        initButtonSelection();
        initNumberInputs();
        _blockNativeContextMenu();

        const config = await _loadConfig();
        applyTheme(config.theme || 'dark');
        _applyButtonTextSize(config.button_text_size);
        await loadLanguage(config.language || 'es');
        _wireCloseButtons();

        if (config.is_first_boot) {
            _show('wizard-section');
            initWizard();
            return;
        }

        const [grid, fixedPanel] = await Promise.all([
            invoke('get_grid_state'), initialFixedPanel(),
        ]);
        _initModules(config, grid, fixedPanel);
        initStartupPrompts();
        await wireRuntimeEvents({ onRefresh: _refresh, onDockEditor: _openDockedEditor });
        _show('app-section');
        checkAudioDevicesOnStartup();
        await maybeShowPreloadDialog(); // Rust decide si toca (primer arranque)
        await runStartupPrompts();
        startUpdateChecks();
    } catch (e) {
        console.error('Error iniciando la app:', e);
        await loadLanguage('es').catch(() => {});
        _showError(e.message);
    }
}

/** Arranque en modo ventana de la consola: solo tema, i18n y las tiras. El
 *  audio-tick llega igual — es un evento de Tauri y lo reciben todas las
 *  ventanas, así que los vúmetros se mueven sin nada más. */
async function _startConsoleWindow() {
    const config = await invoke('get_config').catch(() => ({}));
    applyTheme(config.theme || 'dark');
    await loadLanguage(config.language || 'es');
    listen('theme-changed', e => applyTheme(e.payload?.theme || 'dark')).catch(console.error);
    _blockNativeContextMenu();
    document.getElementById('loading-screen')?.classList.add('hidden');
    initConsoleWindow();
    listen('audio-tick', e => updateConsoleTick(e.payload ?? {})).catch(console.error);
}

/** Arranque en modo ventana del editor (pop-out): solo tema, i18n y el editor a
 *  pantalla completa de la ventana. Reutiliza los módulos del editor. */
async function _startEditorWindow(rawPath) {
    const params = new URLSearchParams(location.search);
    const config = await invoke('get_config').catch(() => ({}));
    applyTheme(config.theme || 'dark');
    await loadLanguage(config.language || 'es');
    listen('theme-changed', e => applyTheme(e.payload?.theme || 'dark')).catch(console.error);
    _wireCloseButtons();
    _blockNativeContextMenu();
    document.body.classList.add('editor-window-mode');
    document.getElementById('loading-screen')?.classList.add('hidden');
    // Detener la previa si se cierra la ventana sin usar "Cerrar".
    window.addEventListener('beforeunload', () => {
        if (!window.__lfDockingTrackEditor) {
            invoke('set_editor_mode', { mode: 'window' }).catch(() => {});
        }
        invoke('stop_audio', { id: '__track_preview__' }).catch(() => {});
    });
    const editor = await import('./trackEditor.js');
    editor.openTrackEditor(decodeURIComponent(rawPath), params.get('name') || '', null, {
        zoom: parseFloat(params.get('zoom') || '1'),
    });
}

async function _loadConfig() {
    const config = await invoke('get_config');
    if (!config?.profiles?.length || !config.active_profile_id) {
        throw new Error('INVALID_APP_CONFIG');
    }
    return config;
}

async function _refresh() {
    const [config, grid, fixedPanel] = await Promise.all([
        _loadConfig(),
        invoke('get_grid_state'),
        invoke('get_fixed_panel'),
    ]);
    updateTabs(config, _refresh);
    updateTabDnd(config, _refresh);
    updateProfiles(config, _refresh);
    updateShortcuts(config, _refresh);
    _applyButtonTextSize(config.button_text_size);
    refreshBottomBar();
    drawGrid(grid, _refresh);
    drawFixedPanel(fixedPanel);
}

function _initModules(config, grid, fixedPanel) {
    initTabs(config, _refresh);
    initTabDnd(config, _refresh);
    initProfiles(config, _refresh);
    initShortcuts(config, _refresh);
    initGrid(_refresh);
    initGridDnd(_refresh);
    initFileDrop(_refresh);
    initPlayerDnd();
    initPlayerDrop();
    initBottomBar();
    drawGrid(grid, _refresh);
    initFixedPanel(fixedPanel, _refresh);
    wireConsole();
    initSettingsModal(_refresh);
    initMapping(_refresh);
    initUpdateNotifier();
}

async function _openDockedEditor(payload) {
    const editor = await import('./trackEditor.js');
    editor.openTrackEditor(payload.path, payload.name || '', null, { zoom: payload.zoom });
}

function _applyButtonTextSize(size = 'normal') {
    document.body.dataset.buttonTextSize = size || 'normal';
}

function _blockNativeContextMenu() {
    document.addEventListener('contextmenu', e => e.preventDefault(), true);
}

function _wireCloseButtons() {
    if (_closeWired) return;
    _closeWired = true;
    document.querySelectorAll('[data-close]').forEach(btn => {
        btn.addEventListener('click', () => {
            document.getElementById(btn.getAttribute('data-close'))
                ?.classList.add('hidden');
        });
    });
}

function _show(sectionId) {
    document.getElementById('loading-screen')?.classList.add('hidden');
    document.getElementById(sectionId)?.classList.remove('hidden');
}

function _showError(msg) {
    const screen = document.getElementById('loading-screen');
    screen.innerHTML = `
        <div style="text-align:center;padding:40px">
            <h2 style="color:var(--error-color)">${t('errors.fatal_ipc')}</h2>
            <p style="color:var(--text-secondary);margin-top:10px;font-size:12px">
                ${msg ?? ''}
            </p>
        </div>`;
}
