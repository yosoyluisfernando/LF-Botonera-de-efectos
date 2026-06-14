/**
 * Archivo: tabModal.js
 * Propósito: Controla el modal de creación y edición de pestañas (paletas).
 * Delega persistencia a Rust (Regla 4). Sin texto hardcoded (Regla 6).
 */

import { invoke } from './api.js';
import { t } from './i18n.js';

/**
 * Abre el modal de pestaña.
 * @param {object}      config    AppConfig actual.
 * @param {object|null} paleta    PaletaData a editar; null = nueva pestaña.
 * @param {Function}    onRefresh Callback tras guardar.
 */
export async function openTabModal(config, paleta, onRefresh) {
    const isEdit = paleta !== null;
    const modal  = document.getElementById('tab-modal');

    document.getElementById('tab-modal-title').textContent =
        t(isEdit ? 'tab_modal.title_edit' : 'tab_modal.title_new');

    const nameEl  = document.getElementById('tab-name');
    const rowsEl  = document.getElementById('tab-v');
    const colsEl  = document.getElementById('tab-h');
    const audioEl = document.getElementById('tab-audio-out');

    if (isEdit) {
        nameEl.value  = paleta.nombre;
        rowsEl.value  = paleta.rows;
        colsEl.value  = paleta.cols;
    } else {
        const count   = _tabCount(config);
        nameEl.value  = `${t('tabs.default_name')} ${count + 1}`;
        rowsEl.value  = 5;
        colsEl.value  = 5;
    }

    await _fillAudioSelect(audioEl, isEdit ? paleta.audio_out : '');

    modal.classList.remove('hidden');
    nameEl.focus();

    document.getElementById('btn-save-tab').onclick = async () => {
        const datos = {
            profileId: config.active_profile_id,
            nombre:    nameEl.value.trim() || t('tabs.default_name'),
            rows:      parseInt(rowsEl.value) || 5,
            cols:      parseInt(colsEl.value) || 5,
            audioOut:  audioEl.value,
        };
        try {
            if (isEdit) {
                await invoke('update_paleta_meta', { ...datos, paletaId: paleta.id });
            } else {
                await invoke('create_paleta', datos);
            }
            modal.classList.add('hidden');
            onRefresh?.();
        } catch (e) { console.error('Error al guardar pestaña:', e); }
    };
}

async function _fillAudioSelect(select, current) {
    const devices = await invoke('get_audio_devices').catch(() => ['default']);
    select.innerHTML = `<option value="">${t('tab_modal.audio_global')}</option>`;
    devices.forEach(d => {
        const opt = document.createElement('option');
        opt.value = d;
        opt.textContent = d;
        if (d === current) opt.selected = true;
        select.appendChild(opt);
    });
    if (!current) select.value = '';
}

function _tabCount(config) {
    return config.profiles
        .find(p => p.id === config.active_profile_id)
        ?.paletas.length ?? 0;
}
