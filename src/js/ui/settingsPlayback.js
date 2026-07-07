/**
 * Modulo: settingsPlayback.js
 * Proposito: seccion Reproduccion en ajustes: fundidos y barra de progreso.
 */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

const ALLOWED_STEPS = [1, 2, 5, 10, 20, 30];

export function initPlaybackPanel() {
    _ensureSettingsOrder();
    if (document.getElementById('s-reproduccion')) return;
    const panel = document.createElement('div');
    panel.id = 's-reproduccion';
    panel.className = 's-panel hidden';
    panel.innerHTML = `
      <label class="checkbox-line">
        <input type="checkbox" id="playback-progress-enabled">
        <span>${t('playback_progress.enabled')}</span>
      </label>
      <div class="row"><div class="col">
        <label>${t('playback_progress.step')}</label>
        <select id="playback-progress-step">${_stepOptions()}</select>
      </div></div>
      <p class="hint">${t('playback_progress.hint')}</p>
      <hr class="modal-divider">`;
    const fade = document.querySelector('.fade-settings-section');
    if (fade) panel.appendChild(fade);
    document.getElementById('s-atajos')?.before(panel);
}

export function loadPlaybackPanel(config) {
    const fade = config?.fade || {};
    _setNumber('fade-in', fade.fade_in_s ?? 0);
    _setNumber('fade-out-stop', fade.fade_out_stop_s ?? 0);
    _setNumber('fade-out-end', fade.fade_out_end_s ?? 0);

    const progress = config?.playback_progress || {};
    _setChecked('playback-progress-enabled', !!progress.enabled);
    const step = ALLOWED_STEPS.includes(progress.seek_step_s) ? progress.seek_step_s : 10;
    _setValue('playback-progress-step', String(step));
}

export async function savePlaybackPanel() {
    await invoke('set_fade_config', {
        configIn: {
            fade_in_s: _getNumber('fade-in'),
            fade_out_stop_s: _getNumber('fade-out-stop'),
            fade_out_end_s: _getNumber('fade-out-end'),
        },
    });
    await invoke('set_playback_progress_config', {
        configIn: {
            enabled: _getChecked('playback-progress-enabled'),
            seek_step_s: _getStep(),
        },
    });
}

function _setNumber(id, value) {
    _setValue(id, value);
}

function _setValue(id, value) {
    const el = document.getElementById(id);
    if (el) el.value = value;
}

function _setChecked(id, value) {
    const el = document.getElementById(id);
    if (el) el.checked = value;
}

function _getNumber(id) {
    return parseFloat(document.getElementById(id)?.value || '0') || 0;
}

function _getChecked(id) {
    return !!document.getElementById(id)?.checked;
}

function _getStep() {
    const value = parseInt(document.getElementById('playback-progress-step')?.value || '10', 10);
    return ALLOWED_STEPS.includes(value) ? value : 10;
}

function _ensureSettingsOrder() {
    const tabs = document.querySelector('.settings-tabs');
    const content = document.querySelector('.settings-content');
    const mainTab = tabs?.querySelector('[data-target="s-main"]');
    if (!tabs || !content || !mainTab) return;

    let playbackTab = tabs.querySelector('[data-target="s-reproduccion"]');
    if (!playbackTab) {
        playbackTab = document.createElement('button');
        playbackTab.className = 's-tab';
        playbackTab.dataset.target = 's-reproduccion';
        playbackTab.dataset.i18n = 'settings.tab_playback';
        playbackTab.textContent = t('settings.tab_playback');
    }
    mainTab.after(playbackTab);
    _moveTab(tabs, 's-precarga');
    _moveTab(tabs, 's-locuciones');
    _moveTab(tabs, 's-atajos');
    _moveTab(tabs, 's-about');
}

function _moveTab(tabs, target) {
    const tab = tabs.querySelector(`[data-target="${target}"]`);
    if (tab) tabs.appendChild(tab);
}

function _stepOptions() {
    return ALLOWED_STEPS.map(s =>
        `<option value="${s}">${t(`playback_progress.step_${s}`)}</option>`
    ).join('');
}
