/**
 * Archivo: updateNotifier.js
 * Propósito: Aviso visual de actualizaciones. Rust decide cadencia y versión.
 */

import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

const CHECK_EVERY_MS = 12 * 60 * 60 * 1000;
let _latestUrl = '';
let _latestResult = null;
let _wired = false;
let _checksStarted = false;

/** Conecta botones. Las revisiones se inician luego de los prompts de arranque. */
export function initUpdateNotifier() {
    if (_wired) return;
    _wired = true;
    document.getElementById('btn-check-updates')?.addEventListener('click', () => _check(true));
    document.getElementById('btn-update-open')?.addEventListener('click', _openRelease);
    document.getElementById('btn-update-later')?.addEventListener('click', _hideNotice);
    document.getElementById('update-reminder-btn')?.addEventListener('click', _reshowNotice);
    document.getElementById('update-notice-modal')?.addEventListener('click', e => {
        if (e.target?.id === 'update-notice-modal') _hideNotice();
    });

}

export function startUpdateChecks() {
    if (_checksStarted) return;
    _checksStarted = true;
    _check(false, true);
    setInterval(() => _check(false), CHECK_EVERY_MS);
}

async function _check(force, startup = false) {
    const status = document.getElementById('update-status');
    if (force && status) status.textContent = t('updates.checking');
    try {
        const result = await invoke('check_for_updates', { force, startup });
        if (!result.checked && !force) return;
        _paintStatus(result, force);
        if (result.updateAvailable) _showNotice(result);
    } catch (e) {
        if (force && status) status.textContent = t('updates.error');
        console.error('Error revisando actualizaciones:', e);
    }
}

function _paintStatus(result, force) {
    const status = document.getElementById('update-status');
    if (!status) return;
    if (!result.checked && force) {
        status.textContent = t('updates.skipped');
    } else if (result.updateAvailable) {
        status.textContent = `${t('updates.available')} ${result.latestVersion}`;
    } else if (result.checked) {
        status.textContent = t('updates.up_to_date');
    }
}

function _showNotice(result) {
    _latestResult = result;
    _latestUrl = result.releaseUrl;
    _showReminder();
    document.getElementById('update-current-version').textContent = result.currentVersion;
    document.getElementById('update-latest-version').textContent = result.latestVersion;
    document.getElementById('update-notice-modal').classList.remove('hidden');
}

function _hideNotice() {
    document.getElementById('update-notice-modal')?.classList.add('hidden');
}

function _reshowNotice() {
    if (_latestResult) _showNotice(_latestResult);
}

function _showReminder() {
    document.getElementById('update-reminder-btn')?.classList.remove('hidden');
}

function _openRelease() {
    if (!_latestUrl) return;
    window.__TAURI__?.opener?.openUrl(_latestUrl).catch(console.error);
    _hideNotice();
}
