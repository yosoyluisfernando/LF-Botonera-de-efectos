import { invoke } from '../bridge/api.js';
import { isAndroid } from '../bridge/platform.js';
import { t } from '../util/i18n.js';
import { mountClient } from '../remote/client.js';
import '../../css/remote.css';

let disconnectClient = null;
let saveHost = null;
let refreshHost = null;

export function initSettingsRemote(config) {
    const tabs = document.querySelector('.settings-tabs');
    const tab = document.createElement('button');
    tab.type = 'button'; tab.className = 's-tab'; tab.dataset.target = 's-remote';
    tab.textContent = t('remote.title');
    tabs.insertBefore(tab, tabs.querySelector('[data-target="s-about"]'));
    const panel = document.createElement('div');
    panel.id = 's-remote'; panel.className = 's-panel hidden';
    document.querySelector('.settings-content').append(panel);
    const development = document.createElement('p');
    development.className = 'settings-development-notice';
    development.dataset.i18n = 'remote.in_development';
    development.textContent = t('remote.in_development');
    if (isAndroid()) {
        const dialog = document.createElement('dialog');
        dialog.className = 'remote-dialog';
        const close = document.createElement('button');
        close.type = 'button';
        close.textContent = t('mobile.back');
        close.addEventListener('click', () => dialog.close());
        const client = document.createElement('div');
        client.className = 'remote-client';
        dialog.append(close, client);
        document.body.append(dialog);
        const disconnect = mountClient(client, {
            send: (address, token, request) => invoke('remote_request', { address, token, request }),
        });
        disconnectClient = disconnect;
        dialog.addEventListener('close', disconnect);
        const open = document.createElement('button');
        open.type = 'button';
        open.textContent = t('remote.title');
        open.addEventListener('click', () => dialog.showModal());
        const label = document.createElement('label');
        label.className = 'checkbox-line';
        const enabled = document.createElement('input');
        enabled.type = 'checkbox'; enabled.id = 'remote-client-enabled';
        label.append(enabled, document.createTextNode(t('remote.client_enabled')));
        const warning = document.createElement('p');
        warning.className = 'hint';
        warning.dataset.i18n = 'remote.encryption_warning';
        warning.textContent = t('remote.encryption_warning');
        panel.append(development, label, open, warning);
        const shortcut = open.cloneNode(true);
        shortcut.id = 'btn-remote-mirror';
        shortcut.className = 'btn-settings remote-shortcut';
        shortcut.textContent = '🎮';
        shortcut.title = t('remote.title');
        shortcut.setAttribute('aria-label', t('remote.title'));
        shortcut.addEventListener('click', () => dialog.showModal());
        document.getElementById('btn-settings').before(shortcut);
        enabled.addEventListener('change', () => _applyClientEnabled(enabled.checked));
        loadSettingsRemote(config);
        return;
    }
    panel.innerHTML = development.outerHTML + `<label class="checkbox-line"><input type="checkbox" id="remote-host-enabled">
      <span>${t('remote.client_enabled')}</span></label>
      <fieldset id="remote-host-options" class="settings-dependent" disabled inert>
      <p>${t('remote.host_help')}</p>
      <label>${t('remote.address')} (IPv4)<input type="text" id="remote-ip" required inputmode="decimal" placeholder="192.168.1.10"></label>
      <label>${t('remote.port')}<input id="remote-port" type="number" min="1024" max="65535" value="17865"></label>
      <div id="remote-host-details" hidden>
      <label>${t('remote.address')}<input type="text" id="remote-url" readonly></label>
      <label>${t('remote.key')}<input type="text" id="remote-token" readonly></label></div></fieldset>
      <p id="remote-host-status" role="status"></p>
      <p class="hint" data-i18n="remote.encryption_warning">${t('remote.encryption_warning')}</p>`;
    const enabled = panel.querySelector('#remote-host-enabled');
    const options = panel.querySelector('#remote-host-options');
    const ip = panel.querySelector('#remote-ip');
    const port = panel.querySelector('#remote-port');
    const message = panel.querySelector('#remote-host-status');
    let lastStatus = null;
    const sync = () => { options.disabled = options.inert = !enabled.checked; };
    const update = status => {
        lastStatus = status;
        enabled.checked = status.enabled;
        sync();
        ip.disabled = port.disabled = status.enabled;
        if (!ip.value) ip.value = status.suggested_ip;
        panel.querySelector('#remote-host-details').hidden = !status.enabled;
        panel.querySelector('#remote-url').value = status.enabled ? `${status.url}/remote.html` : '';
        panel.querySelector('#remote-token').value = status.token;
        message.textContent = t(status.enabled ? 'remote.enabled' : 'remote.disabled');
    };
    async function act(command, args) {
        enabled.disabled = true;
        try { await invoke(command, args); update(await invoke('remote_status')); return true; }
        catch (error) {
            try { update(await invoke('remote_status')); } catch {
                enabled.checked = !!lastStatus?.enabled;
                sync();
            }
            const key = `remote.errors.${error}`;
            message.textContent = t(key) === key ? t('remote.errors.remote_start') : t(key);
            return false;
        } finally { enabled.disabled = false; }
    }
    enabled.addEventListener('change', () => {
        sync();
        if (!enabled.checked) void act('remote_stop');
        else message.textContent = t('remote.apply_hint');
    });
    saveHost = async () => {
        if (enabled.checked && (!ip.reportValidity() || !port.reportValidity())) throw t('remote.errors.remote_address');
        const success = await act(enabled.checked ? 'remote_start' : 'remote_stop',
            { ip: ip.value, port: Number(port.value) });
        if (!success) throw t('remote.errors.remote_connection');
    };
    refreshHost = () => act('remote_status');
    void refreshHost();
}

export function loadSettingsRemote(config) {
    if (!isAndroid()) return refreshHost?.();
    const enabled = config.remote_client_enabled ?? false;
    document.getElementById('remote-client-enabled').checked = enabled;
    _applyClientEnabled(enabled);
}

export async function saveSettingsRemote() {
    if (!isAndroid()) return saveHost?.();
    const enabled = document.getElementById('remote-client-enabled').checked;
    await invoke('set_toolbar_button', { button: 'remote_client', visible: enabled });
    _applyClientEnabled(enabled);
}

function _applyClientEnabled(enabled) {
    document.getElementById('btn-remote-mirror').classList.toggle('hidden', !enabled);
    document.querySelector('#s-remote > button').disabled = !enabled;
    if (!enabled) {
        document.querySelector('.remote-dialog').close();
        disconnectClient?.();
    }
}
