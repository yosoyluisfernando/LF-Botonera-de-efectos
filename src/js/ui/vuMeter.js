/**
 * Archivo: vuMeter.js
 * Propósito: Pinta el vúmetro estéreo de la barra inferior.
 * Técnica visual: clip-path revela el gradiente proporcional al nivel recibido de Rust.
 * No registra listeners — main.js es el único punto de escucha de audio-tick.
 * La transición CSS maneja el decaimiento visual cuando Rust deja de emitir (Regla 4).
 */

/** Actualiza los dos canales VU. Llamar desde el handler de audio-tick en main.js. */
export function updateVuMeter(payload) {
    // `idle` lo decide Rust: es el último tick antes del silencio. NO se deduce
    // de que no haya botones — la música del reproductor suma en el mismo bus y
    // no aparece en esa lista, así que deducirlo daba cada tick por final, le
    // ponía el decaimiento de 0.8 s y la aguja nunca alcanzaba el nivel real.
    const isFinal = payload.idle ?? false;
    _setLevel('vu-left',  payload.master_level_l ?? 0, isFinal);
    _setLevel('vu-right', payload.master_level_r ?? 0, isFinal);
}

// isFinal=true → último tick antes del silencio: transición larga para decaimiento suave.
// isFinal=false → audio activo: transición corta para respuesta inmediata.
function _setLevel(id, level, isFinal) {
    const el = document.getElementById(id);
    if (!el) return;
    el.style.transition = isFinal ? 'clip-path 0.8s ease-out' : 'clip-path 0.07s linear';
    el.style.clipPath   = `inset(0 ${((1 - Math.min(level, 1)) * 100).toFixed(1)}% 0 0)`;
}
