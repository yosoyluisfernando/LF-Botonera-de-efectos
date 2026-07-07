/**
 * Archivo: editModal.js
 * Propósito: Modal de edición de botón, calcado de la botonera del LFA:
 * un selector nativo elige el tipo (Audio / Hora / Temperatura / Humedad) y
 * el botón "..." abre archivo de audio (tipo Audio) o carpeta (locuciones).
 * Cada locución guarda SU PROPIA carpeta en el botón. Reglas 1, 4 y 6.
 */

import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { invokeShortcutSave } from './shortcutSave.js';
import { attachPalette, refreshColorInputs } from './colorPalette.js';
import { appAlert } from './appDialog.js';
import {
    canPrelisten, defaultFolder, isFolderType, isLocution,
    labelKey, placeholderKey, selectedType, setTypeState, typeOptions,
} from './editTypes.js';
import {
    currentEditVolumeLinear, setEditVolumeFromLinear, syncEditVolumeControl,
} from './editVolumeControl.js';

/**
 * Abre el modal de edición para el botón indicado.
 * @param {number}      index    Índice del botón en la cuadrícula.
 * @param {object|null} btnData  Datos actuales del botón (null = celda vacía).
 * @param {Function}    onSave   Callback ejecutado tras guardar con éxito.
 */
export async function openEditModal(index, btnData, onSave) {
    const modal  = document.getElementById('edit-modal');
    const typeEl = document.getElementById('edit-type');
    const pathEl = document.getElementById('edit-filepath');
    const nameEl = document.getElementById('edit-name');

    modal.querySelector('.modal-header h3').textContent =
        `${t('edit_modal.title')} ${index})`;

    // Tipo original del botón (serde expone type_field como "type")
    await _loadTypeState(btnData?.type);
    _syncTypeOptions(typeEl);
    const type0 = selectedType();
    typeEl.value = type0;
    pathEl.value = isFolderType(type0) ? (btnData?.folder ?? '') : (btnData?.path ?? '');
    if (isLocution(type0) && !pathEl.value) {
        pathEl.value = defaultFolder(type0);
    }
    nameEl.value = btnData?.name || btnData?.label || '';
    setEditVolumeFromLinear(btnData?.vol ?? 1.0);
    document.getElementById('edit-bg-color').value   = btnData?.color_bg   ?? '#444444';
    document.getElementById('edit-text-color').value = btnData?.color_text ?? '#ffffff';
    document.getElementById('edit-shortcut').value   = btnData?.shortcut   ?? '';
    await attachPalette(
        document.getElementById('edit-bg-color'),
        document.getElementById('edit-text-color'),
        'button',
    );
    await refreshColorInputs();
    _applyPathHint(type0, pathEl);
    syncEditVolumeControl(type0);
    if (!btnData) _applySuggestedStyle();

    modal.classList.remove('hidden');
    nameEl.focus();

    // Cambiar tipo (LFA): restaura la ruta original si vuelve al tipo del
    // botón, si no la limpia; las locuciones proponen su nombre estándar
    typeEl.onchange = () => {
        const sel = typeEl.value;
        pathEl.value = sel === type0
            ? (isFolderType(sel) ? (btnData?.folder ?? '') : (btnData?.path ?? ''))
            : '';
        if (isLocution(sel)) {
            pathEl.value = pathEl.value || defaultFolder(sel);
            nameEl.value = t(labelKey(sel));
            if (!pathEl.value) appAlert(t(`edit_modal.missing_${sel}`));
        }
        setEditVolumeFromLinear(sel === type0 ? (btnData?.vol ?? 1.0) : 1.0);
        _applyPathHint(sel, pathEl);
        syncEditVolumeControl(sel);
    };

    // "..." → audio: explorador de ARCHIVOS filtrado a audio (vía Rust/rfd);
    //          locuciones: explorador de CARPETAS
    document.getElementById('btn-select-file').onclick = async () => {
        try {
            if (isFolderType(typeEl.value)) {
                const folder = await invoke('pick_named_folder');
                if (folder?.path) {
                    pathEl.value = folder.path;
                    if (typeEl.value === 'random_folder' && _shouldAutoname(nameEl, btnData)) {
                        nameEl.value = folder.name || t('edit_modal.type_random_folder');
                    }
                }
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
                await refreshColorInputs();
            }
        } catch (_) { /* Usuario canceló el diálogo */ }
    };

    // Pre-escucha: solo aplica a archivos de audio
    document.getElementById('btn-prelisten').onclick = () => {
        if (!canPrelisten(typeEl.value) || !pathEl.value) return;
        const vol = currentEditVolumeLinear(typeEl.value);
        import('./prelisten.js').then(m =>
            m.openPrelisten(pathEl.value, nameEl.value, vol, btnData?.duration ?? 0));
    };

    // Editor de pista (onda + cue + normalizador): solo archivos de audio
    document.getElementById('btn-edit-track').onclick = () => {
        if (!canPrelisten(typeEl.value) || !pathEl.value) return;
        import('./trackEditor.js').then(m =>
            m.openPreferredTrackEditor(pathEl.value, nameEl.value, onSave));
    };

    // Guardar: crea o actualiza el botón en Rust (upsert)
    document.getElementById('btn-save-edit').onclick = async () => {
        const sel   = typeEl.value;
        const isFolder = isFolderType(sel);
        try {
            if (isLocution(sel) && !pathEl.value.trim()) {
                await appAlert(t(`edit_modal.missing_${sel}`));
                return;
            }
            await invokeShortcutSave('update_button_data', {
                index,
                label:     nameEl.value.trim() || btnData?.label || String(index),
                colorBg:   document.getElementById('edit-bg-color').value,
                colorText: document.getElementById('edit-text-color').value,
                btnType:   sel,
                path:      isFolder ? '' : pathEl.value,
                folder:    isFolder ? pathEl.value : '',
                vol:       currentEditVolumeLinear(sel),
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

async function _applySuggestedStyle() {
    try {
        const style = await invoke('suggest_button_style');
        document.getElementById('edit-bg-color').value = style.color_bg;
        document.getElementById('edit-text-color').value = _themeTextColor();
    } catch (_) { /* Mantiene los valores por defecto si Rust no responde. */ }
}

function _applyPathHint(type, pathEl) {
    pathEl.placeholder = t(placeholderKey(type));
}

async function _loadTypeState(type) {
    const state = await invoke('get_edit_button_types', { currentType: type ?? null });
    setTypeState(state);
}

function _shouldAutoname(nameEl, btnData) {
    const current = nameEl.value.trim();
    return !current || current === (btnData?.label ?? '') || current === String(btnData?.index ?? '');
}

function _themeTextColor() {
    return (document.documentElement.dataset.theme || 'dark') === 'light' ? '#111111' : '#ffffff';
}

function _syncTypeOptions(typeEl) {
    typeEl.innerHTML = '';
    typeOptions().forEach(opt => {
        const option = document.createElement('option');
        option.value = opt.id;
        option.textContent = t(opt.label_key);
        typeEl.appendChild(option);
    });
}
