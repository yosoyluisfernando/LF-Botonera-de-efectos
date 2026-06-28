/**
 * Módulo: normConfig.js
 * Propósito: modal de ajustes del normalizador (modo LUFS/Peak, target, ceiling).
 * Se abre desde el botón ⚙ del editor de pistas.
 */
import { invoke } from './api.js';
import { t } from './i18n.js';

let _cfg = null; // NormConfig en vuelo

function buildModal() {
    const el = document.createElement('div');
    el.id = 'norm-config-modal';
    el.className = 'modal-overlay hidden';
    el.innerHTML = `
        <div class="modal-box norm-config-box">
            <h2 data-i18n="norm_config.title"></h2>
            <label>
                <span data-i18n="norm_config.mode_label"></span>
                <select id="nc-mode">
                    <option value="lufs" data-i18n="norm_config.mode_lufs"></option>
                    <option value="peak" data-i18n="norm_config.mode_peak"></option>
                </select>
            </label>
            <div id="nc-lufs-fields">
                <label>
                    <span data-i18n="norm_config.target_lufs"></span>
                    <input type="number" id="nc-target" step="0.5" min="-40" max="0">
                </label>
                <label>
                    <span data-i18n="norm_config.ceiling"></span>
                    <input type="number" id="nc-ceiling" step="0.5" min="-20" max="0">
                    <small data-i18n="norm_config.ceiling_hint"></small>
                </label>
            </div>
            <div id="nc-peak-fields" class="hidden">
                <label>
                    <span data-i18n="norm_config.target_peak"></span>
                    <input type="number" id="nc-target-peak" step="0.5" min="-20" max="0">
                </label>
            </div>
            <div class="modal-actions">
                <button id="nc-cancel" class="btn-secondary" data-i18n="norm_config.cancel"></button>
                <button id="nc-save"   class="btn-primary"   data-i18n="norm_config.save"></button>
            </div>
        </div>
    `;
    document.body.appendChild(el);

    el.querySelector('#nc-mode').addEventListener('change', _syncFields);
    el.querySelector('#nc-save').addEventListener('click', _save);
    el.querySelector('#nc-cancel').addEventListener('click', close);
    return el;
}

function _syncFields() {
    const mode = document.getElementById('nc-mode').value;
    document.getElementById('nc-lufs-fields').classList.toggle('hidden', mode !== 'lufs');
    document.getElementById('nc-peak-fields').classList.toggle('hidden', mode !== 'peak');
}

async function _save() {
    const mode = document.getElementById('nc-mode').value;
    const target = parseFloat(
        mode === 'peak'
            ? document.getElementById('nc-target-peak').value
            : document.getElementById('nc-target').value
    );
    const ceiling = parseFloat(document.getElementById('nc-ceiling').value || '-1');
    try {
        await invoke('set_norm_config', { configIn: { mode, target, ceiling_db: ceiling } });
        _cfg = { mode, target, ceiling_db: ceiling };
        close();
    } catch (e) {
        console.error('set_norm_config:', e);
    }
}

function _applyI18n(el) {
    el.querySelectorAll('[data-i18n]').forEach(node => {
        node.textContent = t(node.dataset.i18n);
    });
}

export function open(cfg) {
    _cfg = cfg;
    let modal = document.getElementById('norm-config-modal');
    if (!modal) modal = buildModal();
    _applyI18n(modal);

    document.getElementById('nc-mode').value = cfg.mode || 'lufs';
    document.getElementById('nc-target').value = cfg.target ?? -14;
    document.getElementById('nc-ceiling').value = cfg.ceiling_db ?? -1;
    document.getElementById('nc-target-peak').value =
        cfg.mode === 'peak' ? (cfg.target ?? -1) : -1;
    _syncFields();
    modal.classList.remove('hidden');
}

export function close() {
    document.getElementById('norm-config-modal')?.classList.add('hidden');
}
