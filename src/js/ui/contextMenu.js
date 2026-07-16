/**
 * Archivo: contextMenu.js
 * Propósito: Gestiona el menú contextual (clic derecho sobre botón).
 * Usa el elemento estático #context-menu del HTML.
 * Checkboxes de loop/overlap/stop_other delegan toggles a Rust (Regla 4).
 */

import { openEditModal } from './editModal.js';
import { invoke } from '../bridge/api.js';
import { placeMenu } from '../util/menuPosition.js';
import { hasSelection, paintSelection } from './buttonSelection.js';

let _cleanupFn = null;

/**
 * Muestra el menú contextual.
 * @param {number}      x         Coordenada X del cursor.
 * @param {number}      y         Coordenada Y del cursor.
 * @param {number}      index     Índice del botón.
 * @param {object|null} btnData   Datos del botón (null = celda vacía).
 * @param {Function}    onUpdate  Callback tras cualquier cambio de estado.
 * @param {string}      group     'grid' (cuadrícula) | 'fixed' (panel fijo).
 */
export function showContextMenu(x, y, index, btnData, onUpdate, group = 'grid') {
    const menu = document.getElementById('context-menu');

    // Con botones seleccionados (Ctrl+clic) el menú se reduce a una sola opción:
    // pintarlos. Lo demás no aplica a varios a la vez, y dejarlo visible haría
    // creer que "Editar" o "Limpiar" actúan sobre toda la selección.
    if (_showColorOnly(menu, onUpdate)) {
        placeMenu(menu, x, y);
        setTimeout(() => document.addEventListener('click', _hideOnClickOutside), 10);
        return;
    }

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
    _cleanupFn = _wireMenuActions(menu, index, btnData, onUpdate, group);

    // Cerrar al hacer clic fuera
    setTimeout(() => {
        document.addEventListener('click', _hideOnClickOutside);
    }, 10);
}

/** Devuelve true si el menú quedó en modo "solo color" (hay selección). */
function _showColorOnly(menu, onUpdate) {
    const multi = hasSelection();
    menu.querySelectorAll('li, hr').forEach(el => {
        el.classList.toggle('hidden', multi && el.id !== 'menu-color-selected');
    });
    document.getElementById('menu-color-selected')?.classList.toggle('hidden', !multi);
    if (!multi) return false;
    if (_cleanupFn) _cleanupFn();
    const item = document.getElementById('menu-color-selected');
    const onClick = () => {
        _hide();
        paintSelection(onUpdate);
    };
    item.addEventListener('click', onClick);
    _cleanupFn = () => item.removeEventListener('click', onClick);
    return true;
}

function _wireMenuActions(menu, index, btnData, onUpdate, group) {
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
        openEditModal(index, btnData, onUpdate, group).catch(console.error);
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
        const clearCommand = group === 'fixed' ? 'clear_fixed_button' : 'clear_button';
        try { await invoke(clearCommand, { index }); onUpdate?.(); }
        catch (e) { console.error('Error al limpiar botón:', e); }
    };
    const onBucle = () => _toggleButtonFlag(index, btnData, 'loop_mode', onUpdate, group);
    const onOverlap = () => _toggleButtonFlag(index, btnData, 'overlap', onUpdate, group);
    const onDetener = () => _toggleButtonFlag(index, btnData, 'stop_other', onUpdate, group);
    const onRestart = () => _toggleButtonFlag(index, btnData, 'restart', onUpdate, group);

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

async function _toggleButtonFlag(index, btnData, flag, onUpdate, group) {
    if (!btnData) return;
    _hide();
    const toggleCommand = group === 'fixed' ? 'toggle_fixed_button_flag' : 'toggle_button_flag';
    try {
        await invoke(toggleCommand, { index, flag });
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
