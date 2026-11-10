import { invoke } from '../bridge/api.js';
import { isAndroid } from '../bridge/platform.js';
import { t } from '../util/i18n.js';
import { mountClient } from '../remote/client.js';
import '../../css/remote.css';

export function initSettingsRemote() {
    const tabs = document.querySelector('.settings-tabs');
    const tab = document.createElement('button');
    tab.type = 'button'; tab.className = 's-tab'; tab.dataset.target = 's-remote';
    tab.textContent = t('remote.title'); tabs.append(tab);
    const panel = document.createElement('div');
    panel.id = 's-remote'; panel.className = 's-panel hidden';
    document.querySelector('.settings-content').append(panel);
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
        dialog.addEventListener('close', disconnect);
        const open = document.createElement('button');
        open.type = 'button';
        open.textContent = t('remote.title');
        open.addEventListener('click', () => dialog.showModal());
        panel.append(open);
        const shortcut = open.cloneNode(true);
        shortcut.id = 'btn-remote-mirror';
        shortcut.className = 'btn-settings remote-shortcut';
        shortcut.addEventListener('click', () => dialog.showModal());
        document.getElementById('btn-settings').before(shortcut);
        return;
    }
    panel.innerHTML = `<h3>${t('remote.title')}</h3><p>${t('remote.host_help')}</p>
      <label>${t('remote.address')} (IPv4)<input id="remote-ip" required inputmode="decimal" placeholder="192.168.1.10"></label>
      <label>${t('remote.port')}<input id="remote-port" type="number" min="1024" max="65535" value="17865"></label>
      <div class="remote-actions"><button type="button" id="remote-start">${t('remote.enable')}</button>
      <button type="button" id="remote-stop">${t('remote.disable')}</button></div>
      <p id="remote-host-status" role="status"></p><div id="remote-host-details" hidden>
      <label>${t('remote.address')}<input id="remote-url" readonly></label>
      <label>${t('remote.key')}<input id="remote-token" readonly></label>
      <p>${t('remote.network_help')}</p></div>`;
    const start = panel.querySelector('#remote-start');
    const stop = panel.querySelector('#remote-stop');
    const ip = panel.querySelector('#remote-ip');
    const port = panel.querySelector('#remote-port');
    const message = panel.querySelector('#remote-host-status');
    const update = status => {
        start.disabled = status.enabled; stop.disabled = !status.enabled; port.disabled = status.enabled;
        ip.disabled = status.enabled;
        if (!ip.value) ip.value = status.suggested_ip;
        panel.querySelector('#remote-host-details').hidden = !status.enabled;
        panel.querySelector('#remote-url').value = status.enabled ? `${status.url}/remote.html` : '';
        panel.querySelector('#remote-token').value = status.token;
        message.textContent = t(status.enabled ? 'remote.enabled' : 'remote.disabled');
    };
    async function act(command, args) {
        start.disabled = true; stop.disabled = true;
        try { await invoke(command, args); update(await invoke('remote_status')); }
        catch (error) {
            try { update(await invoke('remote_status')); } catch { /* Preserve the reported failure. */ }
            const key = `remote.errors.${error}`;
            message.textContent = t(key) === key ? t('remote.errors.remote_start') : t(key);
        }
    }
    start.addEventListener('click', () => {
        if (ip.reportValidity() && port.reportValidity()) {
            act('remote_start', { ip: ip.value, port: Number(port.value) });
        }
    });
    stop.addEventListener('click', () => act('remote_stop'));
    act('remote_status');
}
