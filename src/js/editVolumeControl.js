/**
 * Archivo: editVolumeControl.js
 * Proposito: control dB del modal de edicion para tipos que aun guardan `vol`.
 */
import { dbToLinear, formatGainDb, linearToDb } from './gainDb.js';
import { isLocution } from './editTypes.js';

/** Inicializa el slider desde el valor lineal persistido. */
export function setEditVolumeFromLinear(vol) {
    const slider = document.getElementById('edit-volume');
    const db = linearToDb(vol ?? 1.0);
    slider.value = db;
    _updateReadout(db);
}

/** Devuelve el multiplicador lineal que entiende Rust para el tipo actual. */
export function currentEditVolumeLinear(type) {
    if (!usesEditVolumeControl(type)) return 1.0;
    return dbToLinear(document.getElementById('edit-volume').value);
}

/** Muestra u oculta el control segun el tipo de boton. */
export function syncEditVolumeControl(type) {
    const col = document.getElementById('edit-volume-col');
    col.classList.toggle('hidden', !usesEditVolumeControl(type));
    _updateReadout(document.getElementById('edit-volume').value);
    document.getElementById('edit-volume').oninput = e => _updateReadout(e.target.value);
}

/** El volumen manual solo aplica a carpetas aleatorias y locuciones. */
export function usesEditVolumeControl(type) {
    return type === 'random_folder' || isLocution(type);
}

function _updateReadout(db) {
    document.getElementById('edit-volume-readout').textContent = formatGainDb(db);
}
