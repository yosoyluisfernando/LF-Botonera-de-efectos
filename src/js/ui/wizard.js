/**
 * Archivo: wizard.js
 * Propósito: Asistente de primer arranque.
 * Solo recoge opciones iniciales visibles y delega el guardado a Rust.
 */

import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { showPreloadPrompt } from './preloadDialog.js';

/** Inicializa el asistente dentro de la sección #wizard-section. */
export function initWizard() {
    const area = document.getElementById('wizard-area');
    if (!area) return;

    area.innerHTML = `
        <div class="modal-content" style="width:450px">
            <div class="modal-header" style="justify-content:center">
                <h3 style="color:var(--accent-color);font-weight:bold;font-size:16px">
                    ${t('wizard.welcome')}
                </h3>
            </div>
            <div class="modal-body">
                <div class="wizard-option">
                    <label>
                        <input type="checkbox" id="wizard-weather">
                        <span>${t('wizard.module_question')}</span>
                    </label>
                    <p class="hint" style="margin-left:24px">${t('wizard.module_desc')}</p>
                </div>
            </div>
            <div class="modal-footer" style="justify-content:center">
                <button id="wizard-start-btn" class="btn-blue"
                        style="width:100%;font-size:14px">
                    ${t('wizard.btn_start')}
                </button>
            </div>
        </div>
    `;

    document.getElementById('wizard-start-btn').addEventListener('click', _finishWizard);
}

async function _finishWizard() {
    const weatherEnabled = document.getElementById('wizard-weather').checked;
    try {
        await invoke('set_first_boot_complete', { weatherEnabled, linkEnabled: false });
        await showPreloadPrompt(() => window.location.reload());
    } catch (e) {
        console.error('Error al guardar configuración inicial:', e);
    }
}
