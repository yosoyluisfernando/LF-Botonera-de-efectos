/** Ajustes visuales de Mic; Rust valida y conserva las preferencias. */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { micSettingsMarkup } from './micSettingsView.js';
import { appAlert } from './appDialog.js';

let opening = false;
let previewCommands = Promise.resolve();
function preview(input) {
    previewCommands = previewCommands.then(() => invoke('mic_preview', { input })).catch(console.error);
}
export async function openMicSettings() {
    if (opening || document.getElementById('mic-settings-modal')) return;
    opening = true;
    try {
        const [config] = await invoke('get_mic');
        const focus = document.activeElement;
        const modal = document.createElement('div');
        modal.id = 'mic-settings-modal'; modal.className = 'modal-overlay';
        modal.innerHTML = micSettingsMarkup();
        document.body.appendChild(modal);
        const field = name => modal.querySelector('#mic-' + name);
        const close = () => { preview(null); modal.remove(); focus?.focus(); };
        modal.querySelector('.close-btn').onclick = close;
        field('cancel').onclick = close;
        modal.addEventListener('keydown', e => {
            e.stopPropagation();
            if (e.key === 'Escape') { e.preventDefault(); close(); }
            if (e.key === 'Tab') {
                const fields = [...modal.querySelectorAll('button, input, select')].filter(el => el.offsetParent !== null);
                const next = e.shiftKey ? fields.at(-1) : fields[0];
                if (document.activeElement === (e.shiftKey ? fields[0] : fields.at(-1))) {
                    e.preventDefault(); next.focus();
                }
            }
        });
        for (const key of ['main', 'panel', 'player']) field(key).checked = config[key];
        for (const key of ['mode', 'down_ms', 'up_ms', 'threshold_db', 'detect_ms', 'silence_ms']) field(key).value = config[key];
        field('level').value = Math.round(config.level * 100);
        const option = (value, label) => {
            const el = document.createElement('option'); el.value = value; el.textContent = label;
            field('input').appendChild(el);
        };
        option('', t('mic.default_input'));
        if (config.input) option(config.input, config.input);
        field('input').value = config.input;
        const updateMode = () => {
            const automatic = field('mode').value === 'auto';
            field('auto-fields').classList.toggle('hidden', !automatic);
            preview(automatic ? field('input').value : null);
        };
        field('input').onchange = updateMode;
        field('mode').onchange = updateMode; updateMode();
        field('mode').focus();
        modal.querySelector('form').onsubmit = async e => {
            e.preventDefault();
            const next = { ...config, mode: field('mode').value, input: field('input').value };
            for (const key of ['main', 'panel', 'player']) next[key] = field(key).checked;
            for (const key of ['down_ms', 'up_ms', 'threshold_db', 'detect_ms', 'silence_ms']) next[key] = Number(field(key).value);
            next.level = Number(field('level').value) / 100;
            try { await invoke('set_mic_config', { config: next }); close(); }
            catch (err) { field('settings-error').textContent = t(String(err)); }
        };
        invoke('mic_devices').then(devices => {
            for (const name of devices) if (name !== config.input) option(name, name);
        }).catch(err => { field('settings-error').textContent = t(String(err)); });
    } catch (err) { await appAlert(t(String(err))); }
    finally { opening = false; }
}
