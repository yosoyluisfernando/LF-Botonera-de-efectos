/** Selector y avisos visuales. Rust decide destinos, disponibilidad y reconexión. */
import '../../css/tabOutputs.css';
import { invoke, listen } from '../bridge/api.js';
import { isAndroid } from '../bridge/platform.js';
import { t } from '../util/i18n.js';
import { refreshConsoleOutputs } from './consoleView.js';

let banner;
let status = { devices: [], issues: [] };
let wired = false;

export async function initTabOutputs() {
    if (isAndroid() || wired) return;
    wired = true;
    await listen('tab-outputs-changed', e => {
        paintOutputStatus(e.payload);
        refreshConsoleOutputs();
    });
    await checkTabOutputs();
}

export async function checkTabOutputs(retry = false) {
    if (isAndroid()) return;
    const next = await invoke('get_tab_output_status', { retry });
    paintOutputStatus(next);
    refreshConsoleOutputs();
    return next;
}

export function paintOutputStatus(next) {
    status = next ?? { devices: [], issues: [] };
    if (!banner) {
        banner = document.createElement('div');
        banner.className = 'tab-output-notice';
        banner.setAttribute('role', 'status');
        document.getElementById('app-section').prepend(banner);
    }
    banner.replaceChildren();
    banner.hidden = !status.issues.length;
    if (banner.hidden) return;
    const text = document.createElement('span');
    text.textContent = t('tab_outputs.silent') + ' ' + status.issues.map(issue =>
        `${issue.nombre} (${issue.device}): ${t(`tab_outputs.${issue.reason}`)}`).join('; ');
    const retry = document.createElement('button');
    retry.textContent = t('tab_outputs.retry');
    retry.onclick = async () => {
        retry.disabled = true;
        try { await checkTabOutputs(true); }
        catch (error) { console.error(error); }
        finally { retry.disabled = false; }
    };
    banner.append(text, retry);
}

export async function fillTabOutput(paleta) {
    if (isAndroid()) return;
    let row = document.getElementById('tab-output-row');
    if (!row) {
        row = document.createElement('div'); row.id = 'tab-output-row';
        const label = document.createElement('label'); label.htmlFor = 'tab-audio-out';
        const select = document.createElement('select'); select.id = 'tab-audio-out';
        const help = document.createElement('p'); help.className = 'tab-output-help';
        row.append(label, select, help);
        document.querySelector('#tab-modal .modal-body').append(row);
    }
    row.querySelector('label').textContent = t('tab_outputs.label');
    row.querySelector('p').textContent = t('tab_outputs.help');
    row.hidden = !!paleta?.obs || document.getElementById('tab-kind')?.value === 'obs';
    const kind = document.getElementById('tab-kind');
    if (kind) kind.onchange = () => { row.hidden = kind.value === 'obs'; };
    const devices = await invoke('get_tab_output_devices');
    const select = row.querySelector('select'); select.replaceChildren();
    const add = (value, label, name, id = '', usable = true) => {
        const option = document.createElement('option'); option.value = value;
        option.textContent = label; option.dataset.name = name; option.dataset.deviceId = id;
        option.disabled = !usable; select.append(option);
    };
    add('', t('tab_outputs.main'), '');
    add('default', t('tab_outputs.default'), 'default');
    for (const device of devices) {
        if (device.name === 'default' && !device.id) continue;
        add(device.id || device.name, device.name + (device.usable ? '' : ` — ${t('tab_outputs.ambiguous')}`),
            device.name, device.id, device.usable);
    }
    const current = paleta?.audio_out || '';
    const id = paleta?.audio_out_id || '';
    const found = devices.find(d => id ? d.id === id : d.name === current);
    const value = current === '' || current === 'default' ? current : found?.id || id || current;
    if (current && current !== 'default' && !found) add(value, `${current} — ${t('tab_outputs.missing')}`, current, id);
    select.value = value;

}

export function selectedTabOutput(paleta) {
    // Android no sobrescribe una asignación de escritorio al editar la pestaña.
    if (isAndroid() || paleta?.obs || document.getElementById('tab-kind')?.value === 'obs') return undefined;
    return document.getElementById('tab-audio-out')?.selectedOptions[0]?.dataset.name;
}

export function selectedTabOutputId(paleta) {
    if (selectedTabOutput(paleta) === undefined) return undefined;
    return document.getElementById('tab-audio-out')?.selectedOptions[0]?.dataset.deviceId || '';
}
