/**
 * Archivo: countdownFormat.js
 * Propósito: Formatea el contador según la duración original del audio.
 */

/** Devuelve texto estable para el contador de la barra inferior. */
export function formatCountdown(remaining, duration) {
    const rem = Math.max(0, Number(remaining) || 0);
    const total = Math.max(0, Number(duration) || 0);
    if (total >= 3600) return _hours(rem);
    if (total >= 60) return _minutes(rem);
    return `${rem.toFixed(1)}s`;
}

function _minutes(seconds) {
    const mins = Math.floor(seconds / 60);
    const secs = seconds - mins * 60;
    return `${mins}:${_pad(secs, 4)}`;
}

function _hours(seconds) {
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds - hrs * 3600 - mins * 60;
    return `${hrs}:${String(mins).padStart(2, '0')}:${_pad(secs, 4)}`;
}

function _pad(value, size) {
    return value.toFixed(1).padStart(size, '0');
}
