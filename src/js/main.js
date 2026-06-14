/**
 * Archivo: main.js
 * Propósito: Punto de entrada. Orquesta la vista activa (Wizard o App).
 * Regla 4: este es el ÚNICO archivo que registra listeners de eventos Rust.
 * Todos los módulos de UI exponen funciones de actualización; aquí se conectan.
 */

import { loadLanguage, t } from './i18n.js';
import { listen, invoke, waitForTauri } from './api.js';
import { initWizard }      from './wizard.js';
import { initGrid, drawGrid } from './grid.js';
import { initGridDnd }     from './gridDnd.js';
import { paintAudioTick }  from './gridPlayback.js';
import { initTabs, updateTabs }       from './tabs.js';
import { initProfiles, updateProfiles } from './profiles.js';
import { initSettingsModal }  from './settingsModal.js';
import { initShortcuts, updateShortcuts } from './shortcuts.js';
import { initTitlebar }    from './titlebar.js';
import { initMapping }     from './mapping.js';
import { initBottomBar }   from './bottomBar.js';
import { updateClockTick, updateAudioTick } from './clockWidget.js';
import { updateVuMeter }   from './vuMeter.js';
import { applyTheme }      from './theme.js';

async function refresh() {
    const [config, grid] = await Promise.all([
        invoke('get_config'),
        invoke('get_grid_state'),
    ]);
    updateTabs(config, refresh);
    updateProfiles(config, refresh);
    updateShortcuts(config, refresh);
    drawGrid(grid, refresh);
}

async function initApp() {
    initTitlebar();

    try {
        await waitForTauri();
        const config = await invoke('get_config');
        applyTheme(config.theme || 'dark');
        await loadLanguage(config.language || 'es');
        _wireCloseButtons();

        if (config.is_first_boot) {
            _show('wizard-section');
            initWizard();
            return;
        }

        const grid = await invoke('get_grid_state');

        // ── Fase 1: todo lo síncrono — nunca falla, siempre corre ────────────
        initTabs(config, refresh);
        initProfiles(config, refresh);
        initShortcuts(config, refresh);
        initGrid(refresh);
        initGridDnd(refresh);
        initBottomBar();
        drawGrid(grid, refresh);
        initSettingsModal(refresh);
        initMapping(refresh);

        // ── Fase 2: conectar los dos únicos canales de eventos con Rust ───────
        // La UI aparece solo cuando ambos listeners están confirmados (Regla 4).
        await Promise.all([
            listen('clock-tick', e => updateClockTick(e.payload ?? {})),
            listen('audio-tick', e => {
                const p = e.payload ?? {};
                paintAudioTick(p);
                updateAudioTick(p);
                updateVuMeter(p);
            }),
        ]);

        // ── Fase 3: mostrar la app — todo está listo ──────────────────────────
        _show('app-section');

    } catch (e) {
        console.error('Error iniciando la app:', e);
        await loadLanguage('es').catch(() => {});
        _showError(e.message);
    }
}

function _show(sectionId) {
    document.getElementById('loading-screen')?.classList.add('hidden');
    document.getElementById(sectionId)?.classList.remove('hidden');
}

function _wireCloseButtons() {
    document.querySelectorAll('[data-close]').forEach(btn => {
        btn.addEventListener('click', () => {
            document.getElementById(btn.getAttribute('data-close'))
                ?.classList.add('hidden');
        });
    });
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

document.addEventListener('DOMContentLoaded', initApp);
