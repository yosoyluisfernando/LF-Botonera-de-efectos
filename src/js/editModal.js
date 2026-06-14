/**
 * Archivo: editModal.js
 * Propósito: Modal de edición de botón, calcado de la botonera del LFA:
 * un select estrecho elige el tipo (Audio / Hora / Temperatura / Humedad) y
 * el botón "..." abre archivo de audio (tipo Audio) o carpeta (locuciones).
 * Cada locución guarda SU PROPIA carpeta en el botón. Reglas 1, 4 y 6.
 */

import { invoke } from './api.js';
import { t } from './i18n.js';

const LOCUTION_LABELS = {
    time:        'locutions.label_time',
    temperature: 'locutions.label_temp',
    humidity:    'locutions.label_hum',
};

const isLocution = type => type in LOCUTION_LABELS;

/**
 * Abre el modal de edición para el botón indicado.
 * @param {number}      index    Índice del botón en la cuadrícula.
 * @param {object|null} btnData  Datos actuales del botón (null = celda vacía).
 * @param {Function}    onSave   Callback ejecutado tras guardar con éxito.
 */
export function openEditModal(index, btnData, onSave) {
    const modal  = document.getElementById('edit-modal');
    const typeEl = document.getElementById('edit-type');
    const pathEl = document.getElementById('edit-filepath');
    const nameEl = document.getElementById('edit-name');

    modal.querySelector('.modal-header h3').textContent =
        `${t('edit_modal.title')} ${index})`;

    // Tipo original del botón (serde expone type_field como "type")
    const type0 = isLocution(btnData?.type) ? btnData.type : 'audio';
    typeEl.value = type0;
    pathEl.value = isLocution(type0) ? (btnData?.folder ?? '') : (btnData?.path ?? '');
    nameEl.value = btnData?.name || btnData?.label || '';
    document.getElementById('edit-volume').value     = btnData?.vol        ?? 1.0;
    document.getElementById('edit-bg-color').value   = btnData?.color_bg   ?? '#444444';
    document.getElementById('edit-text-color').value = btnData?.color_text ?? '#ffffff';
    document.getElementById('edit-shortcut').value   = btnData?.shortcut   ?? '';

    modal.classList.remove('hidden');
    nameEl.focus();

    // Cambiar tipo (LFA): restaura la ruta original si vuelve al tipo del
    // botón, si no la limpia; las locuciones proponen su nombre estándar
    typeEl.onchange = () => {
        const sel = typeEl.value;
        pathEl.value = sel === type0
            ? (isLocution(sel) ? (btnData?.folder ?? '') : (btnData?.path ?? ''))
            : '';
        if (isLocution(sel)) nameEl.value = t(LOCUTION_LABELS[sel]);
    };

    // "..." → audio: explorador de ARCHIVOS filtrado a audio (vía Rust/rfd);
    //          locuciones: explorador de CARPETAS
    document.getElementById('btn-select-file').onclick = async () => {
        try {
            if (isLocution(typeEl.value)) {
                const folder = await invoke('pick_folder');
                if (folder) pathEl.value = folder;
                return;
            }
            const newState = await invoke('assign_file_to_button', { index, path: null });
            const updated  = newState?.buttons?.find(b => b.index === index);
            if (updated) {
                pathEl.value = updated.path;
                nameEl.value = updated.name || updated.label;
                // Rust ya asignó un color aleatorio al botón nuevo
                document.getElementById('edit-bg-color').value   = updated.color_bg;
                document.getElementById('edit-text-color').value = updated.color_text;
            }
        } catch (_) { /* Usuario canceló el diálogo */ }
    };

    // Pre-escucha: solo aplica a archivos de audio
    document.getElementById('btn-prelisten').onclick = () => {
        if (isLocution(typeEl.value) || !pathEl.value) return;
        const vol = parseFloat(document.getElementById('edit-volume').value);
        import('./prelisten.js').then(m =>
            m.openPrelisten(pathEl.value, nameEl.value, vol, btnData?.duration ?? 0));
    };

    // Guardar: crea o actualiza el botón en Rust (upsert)
    document.getElementById('btn-save-edit').onclick = async () => {
        const sel   = typeEl.value;
        const isLoc = isLocution(sel);
        try {
            await invoke('update_button_data', {
                index,
                label:     nameEl.value.trim() || btnData?.label || String(index),
                colorBg:   document.getElementById('edit-bg-color').value,
                colorText: document.getElementById('edit-text-color').value,
                btnType:   sel,
                path:      isLoc ? '' : pathEl.value,
                folder:    isLoc ? pathEl.value : '',
                vol:       parseFloat(document.getElementById('edit-volume').value),
                shortcut:  document.getElementById('edit-shortcut').value,
            });
            modal.classList.add('hidden');
            onSave?.();
        } catch (e) { console.error('Error al guardar botón:', e); }
    };

    document.getElementById('btn-cancel-edit').onclick = () => {
        modal.classList.add('hidden');
    };
}
