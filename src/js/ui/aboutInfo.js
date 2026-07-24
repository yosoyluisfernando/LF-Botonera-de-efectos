/**
 * Representa metadatos que Rust grabó en el ejecutable.
 */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

export async function loadAboutInfo() {
    const info = await invoke('get_distribution_info').catch(() => ({
        version: '?',
        channel: 'direct',
        platform: 'other',
        updateManager: 'github_releases',
    }));
    _set('app-version-number', info.version);
    _set('app-distribution-channel', t(`settings.about_channel_${info.channel}`));
    _set('app-platform', t(`settings.about_platform_${info.platform}`));
    _set('app-update-manager', t(`settings.about_updates_${info.updateManager}`));
}

function _set(id, value) {
    const element = document.getElementById(id);
    if (element) element.textContent = value;
}
