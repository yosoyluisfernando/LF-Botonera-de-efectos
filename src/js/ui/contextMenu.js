/**
 * Archivo: contextMenu.js
 * Propósito: Gestiona el menú contextual (clic derecho sobre botón).
 * Usa el elemento estático #context-menu del HTML.
 * Checkboxes de loop/overlap/stop_other delegan toggles a Rust (Regla 4).
 */

import { openEditModal } from './editModal.js';
import { invoke } from '../bridge/api.js';
import { placeMenu } from '../util/menuPosition.js';

let _cleanupFn = null;

/**
 * Muestra el menú contextual.
 * @param {number}      x         Coordenada X del cursor.
 * @param {number}      y         Coordenada Y del cursor.
 * @param {number}      index     Índice del botón.
 * @param {object|null} btnData   Datos del botón (null = celda vacía).
 * @param {Function}    onUpdate  Callback tras cualquier cambio de estado.
 */
export function showContextMenu(x, y, index, btnData, onUpdate) {
    const menu = document.getElementById('context-menu');

    // Actualizar estado de checkboxes desde los datos del botón
    document.getElementById('check-bucle').textContent   = btnData?.loop_mode  ? '✓' : '';
    document.getElementById('check-overlap').textContent = btnData?.overlap    ? '✓' : '';
    document.getElementById('check-detener').textContent = btnData?.stop_other ? '✓' : '';
    document.getElementById('check-restart').textContent = btnData?.restart    ? '✓' : '';

    // Deshabilitar opciones que requieren un botón con archivo
    _toggleDisabled('menu-limpiar', !btnData);
    _toggleDisabled('menu-previa',  !btnData?.can_prelisten);
    _toggleDisabled('menu-editar-pista', !btnData?.can_prelisten);

    placeMenu(menu, x, y); // Muestra el menú sin salirse de la ventana

    // Limpiar listeners anteriores antes de reasignar
    if (_cleanupFn) _cleanupFn();
    _cleanupFn = _wireMenuActions(menu, index, btnData, onUpdate);

    // Cerrar al hacer clic fuera
    setTimeout(() => {
        document.addEventListener('click', _hideOnClickOutside);
    }, 10);
}

function _wireMenuActions(menu, index, btnData, onUpdate) {
    const editarEl  = document.getElementById('menu-editar');
    const limpiarEl = document.getElementById('menu-limpiar');
    const bucleEl   = document.getElementById('menu-bucle');
    const overlapEl = document.getElementById('menu-overlap');
    const detenerEl = document.getElementById('menu-detener');
    const restartEl = document.getElementById('menu-restart');
    const previaEl  = document.getElementById('menu-previa');
    const editTrackEl = document.getElementById('menu-editar-pista');

    const onEditar = () => {
        _hide();
        openEditModal(index, btnData, onUpdate).catch(console.error);
    };
    const onPrevia  = () => {
        if (!btnData?.can_prelisten) return;
        _hide();
        import('./prelisten.js').then(m =>
            m.openPrelisten(btnData.path, btnData.name || btnData.label,
                            btnData.vol ?? 1.0, btnData.duration ?? 0));
    };
    const onEditTrack = () => {
        if (!btnData?.can_prelisten) return;
        _hide();
        import('./trackEditor.js').then(m =>
            m.openPreferredTrackEditor(btnData.path, btnData.name || btnData.label, onUpdate));
    };
    const onLimpiar = async () => {
        if (!btnData) return;
        _hide();
        try { await invoke('clear_button', { index }); onUpdate?.(); }
        catch (e) { console.error('Error al limpiar botón:', e); }
    };
    const onBucle = () => _toggleButtonFlag(index, btnData, 'loop_mode', onUpdate);
    const onOverlap = () => _toggleButtonFlag(index, btnData, 'overlap', onUpdate);
    const onDetener = () => _toggleButtonFlag(index, btnData, 'stop_other', onUpdate);
    const onRestart = () => _toggleButtonFlag(index, btnData, 'restart', onUpdate);

    editarEl.addEventListener('click', onEditar);
    limpiarEl.addEventListener('click', onLimpiar);
    bucleEl.addEventListener('click', onBucle);
    overlapEl.addEventListener('click', onOverlap);
    detenerEl.addEventListener('click', onDetener);
    restartEl.addEventListener('click', onRestart);
    previaEl.addEventListener('click', onPrevia);
    editTrackEl.addEventListener('click', onEditTrack);

    return () => {
        editarEl.removeEventListener('click', onEditar);
        limpiarEl.removeEventListener('click', onLimpiar);
        bucleEl.removeEventListener('click', onBucle);
        overlapEl.removeEventListener('click', onOverlap);
        detenerEl.removeEventListener('click', onDetener);
        restartEl.removeEventListener('click', onRestart);
        previaEl.removeEventListener('click', onPrevia);
        editTrackEl.removeEventListener('click', onEditTrack);
    };
}

function _hide() {
    document.getElementById('context-menu')?.classList.add('hidden');
    document.removeEventListener('click', _hideOnClickOutside);
}

function _hideOnClickOutside(e) {
    if (!document.getElementById('context-menu')?.contains(e.target)) _hide();
}

async function _toggleButtonFlag(index, btnData, flag, onUpdate) {
    if (!btnData) return;
    _hide();
    try {
        await invoke('toggle_button_flag', { index, flag });
        onUpdate?.();
    } catch (e) {
        console.error(e);
    }
}

function _toggleDisabled(id, disabled) {
    const el = document.getElementById(id);
    if (disabled) el.classList.add('disabled');
    else          el.classList.remove('disabled');
}
