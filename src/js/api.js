/**
 * Archivo: api.js
 * Propósito: Wrapper seguro sobre las APIs globales de Tauri (window.__TAURI__).
 * Cumple Regla 4: Punto central que aísla la comunicación IPC del resto de módulos.
 *
 * CRÍTICO: window.__TAURI__ se resuelve en el CUERPO de cada función, nunca en tiempo
 * de módulo. En producción (WebView2), el objeto global puede no estar inyectado todavía
 * cuando el módulo se parsea por primera vez. Capturarlo al nivel del módulo lo congela
 * como undefined permanentemente y ningún evento ni comando IPC llega jamás.
 */

/**
 * Envía un comando IPC al motor Rust.
 * Si el backend no está disponible, devuelve datos de fallback para no bloquear la UI.
 * @param {string} cmd  Nombre del comando Tauri/Rust.
 * @param {object} [args] Argumentos opcionales.
 * @returns {Promise<any>}
 */
export function invoke(cmd, args) {
    if (window.__TAURI__?.core?.invoke)
        return window.__TAURI__.core.invoke(cmd, args);

    console.warn(`[Sin Backend Tauri] invoke('${cmd}') — usando datos de fallback.`);
    if (cmd === 'get_config') {
        return Promise.resolve({
            is_first_boot: true,
            theme: 'dark',
            audio_device: 'default',
            weather_module_enabled: false,
            lf_automatizador_link: false,
        });
    }
    if (cmd === 'get_grid_state') {
        return Promise.resolve({ columns: 5, rows: 5, buttons: [] });
    }
    return Promise.resolve(null);
}

/**
 * Espera a que window.__TAURI__ esté disponible (lo inyecta WebView2 de forma asíncrona).
 * Resuelve en cuanto el objeto aparece o al cabo de timeoutMs (modo navegador / fallback).
 * Debe llamarse al inicio de initApp() antes de cualquier invoke() o listen().
 * @param {number} [timeoutMs=5000]
 * @returns {Promise<void>}
 */
export function waitForTauri(timeoutMs = 5000) {
    if (window.__TAURI__?.event?.listen) return Promise.resolve();
    return new Promise(resolve => {
        const start = Date.now();
        const id = setInterval(() => {
            if (window.__TAURI__?.event?.listen || Date.now() - start > timeoutMs) {
                clearInterval(id);
                resolve();
            }
        }, 50);
    });
}

/**
 * Suscribe un handler a un evento emitido por el motor Rust.
 * @param {string}   event   Nombre del evento.
 * @param {Function} handler Callback que recibe el evento.
 * @returns {Promise<Function>} Función para cancelar la suscripción.
 */
export function listen(event, handler) {
    if (window.__TAURI__?.event?.listen)
        return window.__TAURI__.event.listen(event, handler);
    console.warn(`[Sin Backend Tauri] listen('${event}') — evento ignorado.`);
    return Promise.resolve(() => {});
}
