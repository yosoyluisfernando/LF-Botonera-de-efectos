/**
 * Modulo: normConfig.js
 * Proposito: modal de ajustes de normalizacion y deteccion automatica de cue.
 */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

let _cfg = null, _cue = null;

function buildModal() {
    const el = document.createElement('div');
    el.id = 'norm-config-modal';
    el.className = 'modal-overlay hidden';
    el.innerHTML = `
        <div class="modal-content">
            <div class="modal-header"><h3>⚙️ <span data-i18n="norm_config.title"></span></h3></div>
            <div class="modal-body">
                <label><span data-i18n="norm_config.mode_label"></span><select id="nc-mode"></select></label>
                <div id="nc-lufs-fields" style="margin-top:10px">
                    <label><span data-i18n="norm_config.target_lufs"></span><input type="number" id="nc-target" step="0.5" min="-40" max="0"></label>
                    <label style="margin-top:8px"><span data-i18n="norm_config.ceiling"></span><input type="number" id="nc-ceiling" step="0.5" min="-20" max="0"><small class="hint" data-i18n="norm_config.ceiling_hint"></small></label>
                </div>
                <div id="nc-peak-fields" class="hidden" style="margin-top:10px">
                    <label><span data-i18n="norm_config.target_peak"></span><input type="number" id="nc-target-peak" step="0.5" min="-20" max="0"></label>
                </div>
                <hr class="modal-divider">
                <div class="modal-section-title" data-i18n="cue_detect.title"></div>
                <label class="modal-toggle-row"><input type="checkbox" id="nc-cue-enabled"><span data-i18n="cue_detect.enabled"></span></label>
                <div id="nc-cue-sub" style="margin-top:8px">
                    <label class="modal-toggle-row"><input type="checkbox" id="nc-cue-start"><span data-i18n="cue_detect.detect_start"></span></label>
                    <label style="margin-top:6px"><span data-i18n="cue_detect.start_thresh"></span><input type="number" id="nc-start-thresh" step="1" min="-80" max="0"></label>
                    <label class="modal-toggle-row" style="margin-top:10px"><input type="checkbox" id="nc-cue-end"><span data-i18n="cue_detect.detect_end"></span></label>
                    <label style="margin-top:6px"><span data-i18n="cue_detect.end_thresh"></span><input type="number" id="nc-end-thresh" step="1" min="-80" max="0"></label>
                    <small class="hint" data-i18n="cue_detect.hint"></small>
                </div>
                <div id="nc-error" class="form-error hidden" data-i18n="cue_detect.need_one"></div>
                <label class="modal-toggle-row hidden" id="nc-dont-ask-row" style="margin-top:14px"><input type="checkbox" id="nc-dont-ask"><span data-i18n="cue_detect.dont_ask"></span></label>
            </div>
            <div class="modal-footer">
                <button id="nc-cancel" class="btn-dark" data-i18n="norm_config.cancel"></button>
                <button id="nc-save" class="btn-blue" data-i18n="norm_config.save"></button>
            </div>
        </div>`;
    document.body.appendChild(el);
    el.querySelector('#nc-mode').innerHTML = `
        <option value="lufs" data-i18n="norm_config.mode_lufs"></option>
        <option value="peak" data-i18n="norm_config.mode_peak"></option>`;
    el.querySelector('#nc-mode').addEventListener('change', _syncFields);
    el.querySelector('#nc-cue-enabled').addEventListener('change', _syncCue);
    el.querySelector('#nc-save').addEventListener('click', _save);
    el.querySelector('#nc-cancel').addEventListener('click', close);
    el.addEventListener('click', e => { if (e.target === el) close(); });
    return el;
}

function _syncFields() {
    const mode = document.getElementById('nc-mode').value;
    document.getElementById('nc-lufs-fields').classList.toggle('hidden', mode !== 'lufs');
    document.getElementById('nc-peak-fields').classList.toggle('hidden', mode !== 'peak');
}

function _syncCue() {
    const on = document.getElementById('nc-cue-enabled').checked;
    document.getElementById('nc-cue-sub').classList.toggle('hidden', !on);
    document.getElementById('nc-error').classList.add('hidden');
}

async function _save() {
    const mode = document.getElementById('nc-mode').value;
    const target = parseFloat(document.getElementById(mode === 'peak' ? 'nc-target-peak' : 'nc-target').value);
    const cueEnabled = document.getElementById('nc-cue-enabled').checked;
    const detectStart = document.getElementById('nc-cue-start').checked;
    const detectEnd = document.getElementById('nc-cue-end').checked;
    if (cueEnabled && !detectStart && !detectEnd) {
        document.getElementById('nc-error').classList.remove('hidden');
        return;
    }
    try {
        const cue = {
            enabled: cueEnabled,
            detect_start: detectStart,
            detect_end: detectEnd,
            start_thresh_db: parseFloat(document.getElementById('nc-start-thresh').value),
            end_thresh_db: parseFloat(document.getElementById('nc-end-thresh').value),
        };
        await invoke('set_norm_config', { configIn: { mode, target, ceiling_db: parseFloat(document.getElementById('nc-ceiling').value || '-1') } });
        await invoke('set_cue_detect_config', { configIn: cue });
        if (document.getElementById('nc-dont-ask')?.checked) await invoke('mark_norm_prompted');
        _cfg = { mode, target, ceiling_db: parseFloat(document.getElementById('nc-ceiling').value || '-1') };
        _cue = cue;
        close();
    } catch (e) { console.error('save norm/cue config:', e); }
}

function _applyI18n(el) {
    el.querySelectorAll('[data-i18n]').forEach(node => { node.textContent = t(node.dataset.i18n); });
}

export function open(cfg, cueCfg = {}, options = {}) {
    _cfg = cfg; _cue = cueCfg;
    let modal = document.getElementById('norm-config-modal');
    if (!modal) modal = buildModal();
    _applyI18n(modal);
    document.getElementById('nc-mode').value = cfg.mode || 'lufs';
    document.getElementById('nc-target').value = cfg.target ?? -14;
    document.getElementById('nc-ceiling').value = cfg.ceiling_db ?? -1;
    document.getElementById('nc-target-peak').value = cfg.mode === 'peak' ? (cfg.target ?? -1) : -1;
    document.getElementById('nc-cue-enabled').checked = !!cueCfg.enabled;
    document.getElementById('nc-cue-start').checked = cueCfg.detect_start ?? true;
    document.getElementById('nc-cue-end').checked = cueCfg.detect_end ?? true;
    document.getElementById('nc-start-thresh').value = cueCfg.start_thresh_db ?? -36;
    document.getElementById('nc-end-thresh').value = cueCfg.end_thresh_db ?? -48;
    document.getElementById('nc-dont-ask').checked = false;
    document.getElementById('nc-dont-ask-row').classList.toggle('hidden', !options.firstTime);
    _syncFields(); _syncCue();
    modal.classList.remove('hidden');
}

export function close() {
    document.getElementById('norm-config-modal')?.classList.add('hidden');
}
