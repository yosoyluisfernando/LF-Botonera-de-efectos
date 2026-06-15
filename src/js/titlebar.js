/**
 * Archivo: titlebar.js
 * Propósito: Conecta los botones de la barra de título personalizada
 * (minimizar, maximizar/restaurar, cerrar) con la API de ventana de Tauri.
 * La ventana se crea sin decoraciones nativas (decorations: false), por lo
 * que estos controles son la única forma visual de gestionar la ventana.
 */

let _wired = false;

/** Inicializa los tres controles de ventana. Llamar una sola vez. */
export function initTitlebar() {
    if (_wired) return;
    const win = window.__TAURI__?.window?.getCurrentWindow?.();
    if (!win) return; // Modo desarrollo sin backend: sin controles
    _wired = true;

    document.getElementById('tb-min')
        ?.addEventListener('click', () => win.minimize());

    document.getElementById('tb-max')
        ?.addEventListener('click', () => win.toggleMaximize());

    document.getElementById('tb-close')
        ?.addEventListener('click', () => win.close());

    // Doble clic sobre la barra = maximizar/restaurar (comportamiento estándar)
    document.querySelector('.titlebar')
        ?.addEventListener('dblclick', e => {
            if (!e.target.closest('.tb-btn')) win.toggleMaximize();
        });
}
