/**
 * Archivo: ipcError.js
 * Proposito: mostrar un error devuelto por Rust. El motor envia una clave
 * (p. ej. "button_not_found"); si no hay traduccion, se muestra tal cual.
 */

import { appAlert } from './appDialog.js';
import { t } from '../util/i18n.js';

/** Traduce la clave de error; devuelve el texto crudo si no hay traduccion. */
export function ipcErrorMessage(err) {
    const key = `errors.${err}`;
    const message = t(key);
    return message === key ? String(err) : message;
}

export async function alertIpcError(err) {
    await appAlert(ipcErrorMessage(err));
}
