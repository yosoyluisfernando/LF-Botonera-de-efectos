import { isAndroid } from '../bridge/platform.js';
import { t } from '../util/i18n.js';

export function initAndroidSettingsNavigation() {
    if (!isAndroid()) return;
    const modal = document.getElementById('settings-modal');
    const header = modal.querySelector('.modal-header');
    const title = header.querySelector('h3');
    const tabs = modal.querySelector('.settings-tabs');
    const content = modal.querySelector('.settings-content');
    const back = document.createElement('button');
    back.type = 'button';
    back.className = 'android-settings-back btn-dark';
    back.textContent = t('mobile.settings_back');
    header.prepend(back);

    back.addEventListener('click', () => {
        modal.dataset.androidView = 'sections';
        tabs.querySelector('.active')?.focus();
    });
    tabs.addEventListener('click', event => {
        if (!event.target.closest('.s-tab')) return;
        modal.dataset.androidView = 'detail';
        content.scrollTop = 0;
        title.focus({ preventScroll: true });
    });
    title.tabIndex = -1;
}

export function resetAndroidSettingsNavigation() {
    if (isAndroid()) document.getElementById('settings-modal').dataset.androidView = 'sections';
}
