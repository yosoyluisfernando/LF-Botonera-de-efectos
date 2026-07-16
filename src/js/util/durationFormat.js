/**
 * Archivo: durationFormat.js
 * Proposito: formatear duraciones enteras de la lista del reproductor.
 * `mmss` para cada pista (04:08) y `hhmmss` para el total (01:16:57).
 */

/** Duracion de una pista: mm:ss (o hh:mm:ss si pasa de una hora). */
export function mmss(seconds) {
    const total = Math.max(0, Math.round(Number(seconds) || 0));
    if (total >= 3600) return hhmmss(total);
    return `${_pad(Math.floor(total / 60))}:${_pad(total % 60)}`;
}

/** Duracion acumulada: hh:mm:ss. */
export function hhmmss(seconds) {
    const total = Math.max(0, Math.round(Number(seconds) || 0));
    const hrs = Math.floor(total / 3600);
    const mins = Math.floor((total % 3600) / 60);
    return `${_pad(hrs)}:${_pad(mins)}:${_pad(total % 60)}`;
}

function _pad(value) {
    return String(value).padStart(2, '0');
}
