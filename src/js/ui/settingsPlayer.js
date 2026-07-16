/**
 * Archivo: settingsPlayer.js
 * Propósito: los ajustes del reproductor en el modal de Ajustes. El modo y el
 * volumen viven en la pestaña **Panel fijo**; la **salida** vive en *Principal*,
 * junto a la principal y la de pre-escucha, para tener todas las salidas de
 * audio en un mismo sitio. Este módulo los gestiona todos porque son del mismo
 * dueño, estén en la pestaña que estén.
 *
 * La lógica vive en Rust (Regla 4): aquí solo se lee lo guardado y se envían los
 * comandos, al pulsar Guardar como el resto del modal.
 */

import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

// El backend admite hasta 1.5, pero ese margen extra solo existe en la app con
// el modo boost del master. Aquí el recorrido es 0-100%, la convención del resto.
const MAX_PERCENT = 100;

let _loaded = { mode: 'normal', percent: 100, device: '', largeFolder: 'ask' };

export function initPlayerSettings() {
    document.getElementById('player-volume')
        ?.addEventListener('input', e => _paintVolume(e.target.value));
    document.getElementById('fixed-view')
        ?.addEventListener('change', syncPlayerSection);
}

/** Modo y volumen solo aplican con la presentación Reproductor: si no, se ocultan
 *  (mismo criterio que "Columnas" y "Filas", que solo se ven en Botones). La
 *  salida NO se oculta: está en Principal y el reproductor puede seguir sonando
 *  aunque el panel esté enseñando los botones fijos. */
export function syncPlayerSection() {
    document.getElementById('player-settings-section')?.classList.toggle(
        'hidden', document.getElementById('fixed-view').value !== 'player');
}

/** `config` y `devices` los trae el modal, que ya los pidió: sin IPC extra. */
export function loadPlayerSettings(config, devices) {
    const player = config?.player ?? {};
    const mode = player.playback_mode || 'normal';
    const device = player.output_device ?? '';
    const raw = Number(player.volume);
    const percent = Math.min(
        Math.round((Number.isFinite(raw) ? raw : 1) * 100), MAX_PERCENT);

    const largeFolder = player.large_folder_action || 'ask';
    _loaded = { mode, percent, device, largeFolder };
    _value('player-large-folder', largeFolder);
    _value('player-mode', mode);
    _fillDevices(devices, device);
    _value('player-volume', percent);
    _paintVolume(percent);
    syncPlayerSection();
}

export async function savePlayerSettings() {
    const mode = document.getElementById('player-mode').value;
    if (mode !== _loaded.mode) await invoke('player_set_mode', { mode });

    const percent = Number(document.getElementById('player-volume').value);
    if (percent !== _loaded.percent) {
        await invoke('player_set_volume', { volume: percent / 100 });
    }

    // Re-aplicar el mismo dispositivo reabre la salida y CORTA la música: solo
    // se envía si cambió de verdad (mismo cuidado que la tarjeta principal).
    const device = document.getElementById('player-device').value;
    if (device !== _loaded.device) await invoke('player_set_device', { device });

    // Deja cambiar lo que se respondió en el aviso de carpeta grande, por si se
    // marcó "recordar siempre" sin querer.
    const largeFolder = document.getElementById('player-large-folder').value;
    if (largeFolder !== _loaded.largeFolder) {
        await invoke('player_set_large_folder_action', { action: largeFolder });
    }

    _loaded = { mode, percent, device, largeFolder };
}

/** La primera opción, vacía, es "el mismo de los efectos": lo que Rust entiende
 *  como cadena vacía y resuelve al dispositivo principal del perfil. */
function _fillDevices(devices, current) {
    const select = document.getElementById('player-device');
    if (!select) return;
    select.innerHTML = '';
    const options = [{ value: '', label: t('player.device_same') }]
        .concat((devices ?? []).map(d => ({ value: d, label: d })));
    for (const { value, label } of options) {
        const opt = document.createElement('option');
        opt.value = value;
        opt.textContent = label;
        if (value === current) opt.selected = true;
        select.appendChild(opt);
    }
}

function _paintVolume(percent) {
    const readout = document.getElementById('player-volume-readout');
    if (readout) readout.textContent = `${percent}%`;
}

function _value(id, value) {
    const el = document.getElementById(id);
    if (el) el.value = value;
}
