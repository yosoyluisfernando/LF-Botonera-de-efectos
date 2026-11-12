/**
 * Representa metadatos que Rust grabó en el ejecutable.
 */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

export async function loadAboutInfo() {
    const info = await invoke('get_distribution_info').catch(() => null);
    if (!info) {
        for (const id of ['app-version-number', 'app-distribution-channel', 'app-platform', 'app-update-manager']) {
            _set(id, t('settings.about_info_unavailable'));
        }
        return;
    }
    _set('app-version-number', info.version);
    _set('app-distribution-channel', t(`settings.about_channel_${info.channel}`));
    _set('app-platform', t(`settings.about_platform_${info.platform}`));
    _set('app-update-manager', t(`settings.about_updates_${info.updateManager}`));
}

function _set(id, value) {
    const element = document.getElementById(id);
    if (element) element.textContent = value;
}
