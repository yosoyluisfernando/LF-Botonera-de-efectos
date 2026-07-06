/**
 * Archivo: profileModal.js
 * Propósito: Controla el modal de creación y edición de perfiles.
 * Delega persistencia a Rust (Regla 4). Sin texto hardcoded (Regla 6).
 */

import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { attachPalette, refreshColorInputs } from './colorPalette.js';

const DEFAULT_BG   = '#008c3a';
const DEFAULT_TEXT = '#ffffff';

/**
 * Abre el modal de perfil.
 * @param {object}      config    AppConfig actual.
 * @param {object|null} profile   ProfileData a editar; null = nuevo perfil.
 * @param {Function}    onRefresh Callback tras guardar.
 */
export function openProfileModal(config, profile, onRefresh) {
    const isEdit = profile !== null;
    const modal  = document.getElementById('profile-modal');

    document.getElementById('profile-modal-title').textContent =
        t(isEdit ? 'profile_modal.title_edit' : 'profile_modal.title_new');

    const nameEl = document.getElementById('profile-name');
    const bgEl   = document.getElementById('profile-bg-color');
    const textEl = document.getElementById('profile-text-color');
    attachPalette(bgEl, textEl, 'profile');

    nameEl.value = isEdit
        ? profile.name
        : `${t('profile_modal.default_name')} ${config.profiles.length + 1}`;
    bgEl.value   = (isEdit && profile.bg)   ? profile.bg   : DEFAULT_BG;
    textEl.value = (isEdit && profile.text) ? profile.text : DEFAULT_TEXT;
    refreshColorInputs();

    modal.classList.remove('hidden');
    nameEl.focus();

    document.getElementById('btn-profile-default-colors').onclick = () => {
        bgEl.value   = DEFAULT_BG;
        textEl.value = DEFAULT_TEXT;
    };

    document.getElementById('btn-save-profile').onclick = async () => {
        const name = nameEl.value.trim() || t('profile_modal.default_name');
        const bg   = bgEl.value;
        const text = textEl.value;

        try {
            if (isEdit) {
                await invoke('update_profile_meta', {
                    id: profile.id, name, bg, text,
                });
            } else {
                await invoke('create_profile', { name });
            }
            modal.classList.add('hidden');
            onRefresh?.();
        } catch (e) { console.error('Error al guardar perfil:', e); }
    };
}
