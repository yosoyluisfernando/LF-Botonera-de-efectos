// Presentación y edición de preferencias OBS; Rust valida y mantiene la conexión.
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { initObsAppearance, loadObsAppearance, obsAppearance, applyObsAppearance } from './obsAppearance.js';
let current = null;
export function initSettingsObs() {
    const about = document.querySelector('.s-tab[data-target="s-about"]');
    const tab = document.createElement('button');
    tab.className = 's-tab'; tab.dataset.target = 's-obs';
    tab.dataset.i18n = 'obs.title'; tab.textContent = t('obs.title');
    about.before(tab);
    const panel = document.createElement('div');
    panel.id = 's-obs'; panel.className = 's-panel hidden';
    panel.innerHTML = `
      <label><input id="obs-enabled" type="checkbox"> ${t('obs.enable')}</label>
      <p>${t('obs.setup_hint')}</p>
      <label for="obs-host">${t('obs.host')}</label><input id="obs-host" type="text" autocomplete="off">
      <label for="obs-port">${t('obs.port')}</label><input id="obs-port" type="number" min="1" max="65535">
      <label for="obs-password">${t('obs.password')}</label><input id="obs-password" type="password" autocomplete="new-password">
      <p>${t('obs.password_hint')}</p>
      <label><input id="obs-reconnect" type="checkbox"> ${t('obs.reconnect')}</label>
      <button type="button" id="obs-test">${t('obs.test')}</button>
      <p id="obs-test-result" role="status"></p><p id="obs-status" role="status"></p>`;
    document.getElementById('s-about').before(panel);
    initObsAppearance(panel);
    document.getElementById('obs-test').onclick = async () => {
        const button = document.getElementById('obs-test');
        button.disabled = true;
        const result = document.getElementById('obs-test-result');
        result.textContent = t('obs.testing');
        try {
            await invoke('obs_test', { config: draft(), password: document.getElementById('obs-password').value || null });
            result.textContent = t('obs.test_ok');
        } catch (error) { result.textContent = t(`errors.${error}`); }
        finally { button.disabled = false; }
    };
}
function draft() {
    return { enabled: document.getElementById('obs-enabled').checked,
        host: document.getElementById('obs-host').value.trim(),
        port: Number(document.getElementById('obs-port').value),
        reconnect: document.getElementById('obs-reconnect').checked,
        button_view: obsAppearance() };
}
export async function loadSettingsObs(config) {
    current = config.obs;
    loadObsAppearance(current.button_view);
    document.getElementById('obs-enabled').checked = current.enabled;
    document.getElementById('obs-host').value = current.host;
    document.getElementById('obs-port').value = current.port;
    document.getElementById('obs-reconnect').checked = current.reconnect;
    document.getElementById('obs-password').value = '';
    document.getElementById('obs-test-result').textContent = '';
    paintObsSettings(await invoke('obs_get'));
}
export async function saveSettingsObs() {
    const config = draft();
    const password = document.getElementById('obs-password').value;
    if (JSON.stringify(current) === JSON.stringify(config) && !password) return;
    await invoke('obs_set_config', { config, password: password || null });
    current = config;
    applyObsAppearance(config.button_view);
    document.getElementById('obs-password').value = '';
}
export function paintObsSettings(state) {
    const el = document.getElementById('obs-status');
    if (el) el.textContent = `${t('obs.status.' + (state.status || 'disabled'))}${state.error ? ': ' + t('errors.' + state.error) : ''}`;
}
