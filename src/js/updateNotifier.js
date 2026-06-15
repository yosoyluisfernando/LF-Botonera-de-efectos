/**
 * Archivo: updateNotifier.js
 * Propósito: Aviso visual de actualizaciones. Rust decide cadencia y versión.
 */

import { invoke } from './api.js';
import { t } from './i18n.js';

const CHECK_EVERY_MS = 12 * 60 * 60 * 1000;
let _latestUrl = '';
let _wired = false;

/** Conecta botones y programa revisiones livianas cada 12 horas. */
export function initUpdateNotifier() {
    if (_wired) return;
    _wired = true;
    document.getElementById('btn-check-updates')?.addEventListener('click', () => _check(true));
    document.getElementById('btn-update-open')?.addEventListener('click', _openRelease);
    document.getElementById('btn-update-later')?.addEventListener('click', _hideNotice);

    setTimeout(() => _check(false), 20000);
    setInterval(() => _check(false), CHECK_EVERY_MS);
}

async function _check(force) {
    const status = document.getElementById('update-status');
    if (force && status) status.textContent = t('updates.checking');
    try {
        const result = await invoke('check_for_updates', { force });
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
    _latestUrl = result.releaseUrl;
    document.getElementById('update-current-version').textContent = result.currentVersion;
    document.getElementById('update-latest-version').textContent = result.latestVersion;
    document.getElementById('update-notice-modal').classList.remove('hidden');
}

function _hideNotice() {
    document.getElementById('update-notice-modal')?.classList.add('hidden');
}

function _openRelease() {
    if (!_latestUrl) return;
    window.__TAURI__?.opener?.openUrl(_latestUrl).catch(console.error);
    _hideNotice();
}
