/**
 * Archivo: theme.js
 * Propósito: Aplica el tema ANTES del primer pintado (Regla 7) y gestiona
 * el listener de cambio de tema del SO para el modo "system".
 * La fuente de verdad es Rust (AppConfig.theme); localStorage es caché de pintado.
 */

const MQ = window.matchMedia('(prefers-color-scheme: dark)');
let _systemListener = null;

/** Devuelve 'dark' o 'light' para un valor de tema ('system' incluido). */
function _resolve(value) {
    return value === 'system' ? (MQ.matches ? 'dark' : 'light') : value;
}

/** Elimina el listener de SO si existe. */
function _teardown() {
    if (_systemListener) {
        MQ.removeEventListener('change', _systemListener);
        _systemListener = null;
    }
}

/**
 * Aplica un valor de tema ('dark', 'light', 'system'), persiste en localStorage
 * y, cuando es 'system', escucha cambios del SO en caliente sin reiniciar.
 */
export function applyTheme(value) {
    localStorage.setItem('lf-botonera-theme', value);
    _teardown();
    document.documentElement.setAttribute('data-theme', _resolve(value));
    if (value === 'system') {
        _systemListener = e => {
            document.documentElement.setAttribute('data-theme', e.matches ? 'dark' : 'light');
        };
        MQ.addEventListener('change', _systemListener);
    }
}

/** Inyecta el tema cacheado inmediatamente para evitar parpadeo blanco (Regla 7). */
function initializeTheme() {
    applyTheme(localStorage.getItem('lf-botonera-theme') || 'system');
}

initializeTheme();
export { initializeTheme };
