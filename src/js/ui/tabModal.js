/**
 * Archivo: tabModal.js
 * Propósito: Controla el modal de creación y edición de pestañas (paletas).
 * Delega persistencia a Rust (Regla 4). Sin texto hardcoded (Regla 6).
 */

import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { attachPalette, refreshColorInputs } from './colorPalette.js';
import { appAlert } from './appDialog.js';

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
    _ensureColorControls();
    const bgEl    = document.getElementById('tab-bg-color');
    const textEl  = document.getElementById('tab-text-color');
    await attachPalette(bgEl, textEl, 'tab');

    if (isEdit) {
        nameEl.value  = paleta.nombre;
        rowsEl.value  = paleta.rows;
        colsEl.value  = paleta.cols;
        bgEl.value    = paleta.tab_bg || '#3a3f44';
        textEl.value  = paleta.tab_text || '#ffffff';
    } else {
        const count   = _tabCount(config);
        // "Pestaña 3", no "BOTONERA 1 3": el nombre base traía un número pegado
        // y al concatenarle la posición salían nombres raros. El número va por
        // posición, así que renombrar una pestaña no descoloca a las siguientes.
        nameEl.value  = _newTabName(count);
        rowsEl.value  = 5;
        colsEl.value  = 5;
        bgEl.value    = '#3a3f44';
        textEl.value  = '#ffffff';
    }
    await refreshColorInputs();

    modal.classList.remove('hidden');
    nameEl.focus();

    document.getElementById('btn-save-tab').onclick = async () => {
        const datos = {
            profileId: config.active_profile_id,
            nombre:    nameEl.value.trim() || _newTabName(_tabCount(config)),
            rows:      parseInt(rowsEl.value) || 5,
            cols:      parseInt(colsEl.value) || 5,
            tabBg:     bgEl.value,
            tabText:   textEl.value,
        };
        try {
            if (isEdit) {
                await invoke('update_paleta_meta', { ...datos, paletaId: paleta.id });
            } else {
                await invoke('create_paleta', datos);
            }
            modal.classList.add('hidden');
            onRefresh?.();
        } catch (e) {
            const key = `errors.${e}`;
            const msg = t(key);
            await appAlert(msg === key ? String(e) : msg);
            console.error('Error al guardar pestaña:', e);
        }
    };
}

/** Los colores van al final del modal. Se insertaban antes de la fila de la
 *  salida de audio, que se retiró por no rutear nada; ya no hay a qué anclarse ni
 *  falta: este es su sitio. */
function _ensureColorControls() {
    if (document.getElementById('tab-bg-color')) return;
    const body = document.querySelector('#tab-modal .modal-body');
    if (!body) return;
    const row = document.createElement('div');
    row.className = 'row tab-color-row';
    row.innerHTML = `
        <div class="col">
          <label data-i18n="tab_modal.bg_color">${t('tab_modal.bg_color')}</label>
          <input type="color" id="tab-bg-color" value="#3a3f44">
        </div>
        <div class="col">
          <label data-i18n="tab_modal.text_color">${t('tab_modal.text_color')}</label>
          <input type="color" id="tab-text-color" value="#ffffff">
        </div>`;
    body.appendChild(row);
}

function _tabCount(config) {
    return config.profiles
        .find(p => p.id === config.active_profile_id)
        ?.paletas.length ?? 0;
}

/** Nombre propuesto para una pestaña nueva: "Pestaña 3" (por posición). */
function _newTabName(count) {
    return `${t('tabs.new_name')} ${count + 1}`;
}
