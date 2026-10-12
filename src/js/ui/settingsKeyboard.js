/** Campos de teclado: captura visual existente y guardado validado por Rust. */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { globalMidiBindings } from './settingsMidi.js';

const fields = [
    ['stop', 'key_stop', 'keyStop'],
    ['next', 'key_next', 'keyNext'],
    ['prev', 'key_prev', 'keyPrev'],
    ['player-play', 'key_player_play', 'keyPlayerPlay'],
    ['player-stop', 'key_player_stop', 'keyPlayerStop'],
    ['player-next', 'key_player_next', 'keyPlayerNext'],
    ['mic', 'key_mic', 'keyMic'],
];

export function loadKeyboardSettings(audio = {}) {
    for (const [id, saved] of fields) {
        document.getElementById('config-key-' + id).value = audio[saved] || '';
    }
    document.getElementById('config-global-keys').checked = !!audio.global_keys;
}

export function saveKeyboardSettings() {
    const payload = { ...globalMidiBindings() };
    for (const [id, , argument] of fields) {
        payload[argument] = document.getElementById('config-key-' + id).value;
    }
    payload.globalKeys = document.getElementById('config-global-keys').checked;
    return invoke('set_global_keys', payload);
}

/** Muestra atajos asignados a botones sin archivo de audio en el panel Atajos. */
export function renderOrphanedShortcuts(config) {
    const section = document.getElementById('shortcuts-orphan-section');
    if (!section) return;
    section.innerHTML = '';

    const profile = config.profiles?.find(p => p.id === config.active_profile_id);
    const orphans = [];
    for (const paleta of (profile?.paletas ?? [])) {
        for (const btn of (paleta.botones ?? [])) {
            if (btn.shortcut && !btn.path && !btn.folder) {
                orphans.push({ paletaId: paleta.id, paletaNombre: paleta.nombre, btn });
            }
        }
    }

    if (!orphans.length) {
        const p = document.createElement('p');
        p.className = 'hint';
        p.textContent = t('settings.no_orphaned');
        section.appendChild(p);
        return;
    }

    const hint = document.createElement('p');
    hint.className = 'hint danger';
    hint.textContent = t('settings.orphaned_hint');
    section.appendChild(hint);

    orphans.forEach(({ paletaId, paletaNombre, btn }) => {
        const row = document.createElement('div');
        row.className = 'orphan-row';
        const lbl = document.createElement('span');
        lbl.innerHTML = `${paletaNombre} — ${btn.label}: <code>${btn.shortcut}</code>`;
        const clrBtn = document.createElement('button');
        clrBtn.className = 'btn-xs-danger';
        clrBtn.textContent = t('settings.clear_shortcut');
        clrBtn.addEventListener('click', async () => {
            await invoke('clear_button_shortcut', { paletaId, index: btn.index });
            row.remove();
        });
        row.appendChild(lbl);
        row.appendChild(clrBtn);
        section.appendChild(row);
    });
}
