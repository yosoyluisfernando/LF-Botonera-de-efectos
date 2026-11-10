// Browser UI only. The PC decides playback, timing, validity and current palette.
import { t } from '../util/i18n.js';
import { renderSnapshot } from './view.js';

export function mountClient(root, { browser = false, send }) {
    root.innerHTML = `
        <h2>${t('remote.title')}</h2><p>${t('remote.client_help')}</p>
        <form class="remote-login">
          <label class="remote-address">${t('remote.address')}<input name="address" autocomplete="off" placeholder="192.168.1.20:17865" required></label>
          <label>${t('remote.key')}<input name="token" type="password" autocomplete="off" required maxlength="64"></label>
          <button type="submit">${t('remote.connect')}</button>
        </form>
        <p class="remote-status" role="status" aria-live="polite"></p>
        <section class="remote-session" hidden>
          <div class="remote-actions"><button type="button" data-action="stop-all">${t('remote.stop_all')}</button>
          <button type="button" data-action="disconnect">${t('remote.disconnect')}</button></div>
          <h3 class="remote-palette"></h3><p class="remote-empty"></p>
          <div class="remote-grid"></div><div class="remote-playing"></div>
        </section>`;
    const form = root.querySelector('form');
    const addressInput = form.elements.address;
    addressInput.required = !browser;
    root.querySelector('.remote-address').hidden = browser;
    const status = root.querySelector('.remote-status');
    const session = root.querySelector('.remote-session');
    let active = false, generation = 0, timer, address, token;
    const report = error => {
        const key = `remote.errors.${error?.message || error}`;
        status.textContent = t(key) === key ? t('remote.errors.remote_connection') : t(key);
    };
    const disconnect = () => {
        active = false; generation++; clearTimeout(timer);
        session.hidden = true; form.hidden = false; token = ''; form.elements.token.value = '';
        root.querySelector('.remote-grid').replaceChildren();
        root.querySelector('.remote-playing').replaceChildren();
        delete root.querySelector('.remote-grid').dataset.signature;
        delete root.querySelector('.remote-playing').dataset.ids;
        status.textContent = t('remote.disconnected');
    };
    const poll = async epoch => {
        if (!active || generation !== epoch) return;
        try {
            const snapshot = await send(address, token, { command: 'snapshot' });
            if (!active || generation !== epoch) return;
            root.dataset.ready = 'true';
            if (browser) {
                document.documentElement.dataset.theme = snapshot.theme === 'system'
                    ? (matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark')
                    : snapshot.theme;
            }
            if (!active || generation !== epoch) return;
            renderSnapshot(root, snapshot, command);
            status.textContent = t('remote.connected');
        } catch (error) {
            if (!active || generation !== epoch) return;
            // Keep disconnect/reconnect available, disable stale triggers immediately.
            root.dataset.ready = 'false';
            root.querySelectorAll('[data-trigger]').forEach(b => { b.disabled = true; });
            report(error);
        }
        if (active && generation === epoch) timer = setTimeout(() => poll(epoch), 400);
    };
    async function command(request) {
        if (!active) return;
        const epoch = generation;
        try { await send(address, token, request); }
        catch (error) { if (generation === epoch) report(error); }
    }
    form.addEventListener('submit', async event => {
        event.preventDefault();
        if (active) return;
        address = addressInput.value.trim(); token = form.elements.token.value.trim();
        active = true; generation++; form.hidden = true; session.hidden = false;
        status.textContent = t('remote.connecting');
        await poll(generation);
    });
    root.querySelector('[data-action="stop-all"]').addEventListener('click', () => command({ command: 'stop_all' }));
    root.querySelector('[data-action="disconnect"]').addEventListener('click', disconnect);
    return disconnect;
}
