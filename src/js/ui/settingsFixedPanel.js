/**
 * Archivo: settingsFixedPanel.js
 * Propósito: pestaña Panel fijo de Ajustes. Es la dueña de la pestaña, así que
 * compone también la sección Reproductor (`settingsPlayer.js`), igual que el
 * panel de Reproducción compone la de fundidos.
 */

import { invoke } from '../bridge/api.js';
import { appConfirm } from './appDialog.js';
import { t } from '../util/i18n.js';
import { initPlayerSettings, loadPlayerSettings, savePlayerSettings } from './settingsPlayer.js';

let _loadedScope = 'global';

export function initFixedPanelSettings() {
    document.getElementById('fixed-view').addEventListener('change', _syncButtonsOnly);
    document.getElementById('fixed-row-mode').addEventListener('change', _syncRows);
    initPlayerSettings();
}

/** `config` y `devices` vienen del modal, que ya los pidió al abrirse. */
export async function loadFixedPanelSettings(config, devices) {
    const { settings } = await invoke('get_fixed_panel');
    document.getElementById('fixed-scope').value = settings.scope;
    _loadedScope = settings.scope;
    document.getElementById('fixed-view').value = settings.view;
    document.getElementById('fixed-side').value = settings.side;
    document.getElementById('fixed-columns').value = settings.columns;
    document.getElementById('fixed-row-mode').value = settings.row_mode;
    document.getElementById('fixed-rows').value = settings.rows;
    document.getElementById('fixed-modes-position').value = settings.modes_position;
    document.getElementById('fixed-visible').checked = settings.visible;
    document.getElementById('fixed-show-start').checked = settings.show_on_start;
    // Despues de fijar `fixed-view`: la seccion Reproductor se muestra u oculta
    // segun esa presentacion, asi que necesita el valor ya puesto.
    loadPlayerSettings(config, devices);
    _syncButtonsOnly();
    _syncRows();
}

export async function saveFixedPanelSettings() {
    const scope = document.getElementById('fixed-scope').value;
    if (scope !== _loadedScope && await appConfirm(t('fixed_panel.delete_previous'))) {
        await invoke('clear_fixed_scope', { scope: _loadedScope });
    }
    const result = await invoke('set_fixed_panel_settings', {
        scope,
        view: document.getElementById('fixed-view').value,
        side: document.getElementById('fixed-side').value,
        visible: document.getElementById('fixed-visible').checked,
        showOnStart: document.getElementById('fixed-show-start').checked,
        columns: Number(document.getElementById('fixed-columns').value),
        rowMode: document.getElementById('fixed-row-mode').value,
        rows: Number(document.getElementById('fixed-rows').value),
        width: (await invoke('get_fixed_panel')).settings.width,
        modesPosition: document.getElementById('fixed-modes-position').value,
    });
    _loadedScope = scope;
    await savePlayerSettings();
    return result;
}

/** ¿La presentación es la rejilla de botones fijos? */
function _isButtonsView() {
    return document.getElementById('fixed-view').value === 'buttons';
}

/** Columnas y Filas son la CAPACIDAD de la rejilla de botones fijos
 *  (`columnas × filas` limita cuántos caben, ver `cmd_fixed_panel::set_fixed_panel_settings`).
 *  En modo reproductor no pintan nada, así que el bloque entero se oculta: la
 *  cola tiene desplazamiento propio y no se organiza por filas ni columnas. */
function _syncRows() {
    const limited = _isButtonsView()
        && document.getElementById('fixed-row-mode').value === 'limited';
    document.getElementById('fixed-row-mode-row').classList.toggle('hidden', !_isButtonsView());
    document.getElementById('fixed-rows-row').classList.toggle('hidden', !limited);
}

/** Oculta lo que solo aplica a la rejilla de botones fijos. Los "controles de
 *  reproducción" son los modos de esos BOTONES (loop, multi, solo, stop…): el
 *  reproductor tiene su propio transporte, así que ahí la opción no pinta nada. */
function _syncButtonsOnly() {
    const buttons = _isButtonsView();
    document.getElementById('fixed-columns-row').classList.toggle('hidden', !buttons);
    document.getElementById('fixed-modes-row').classList.toggle('hidden', !buttons);
}
