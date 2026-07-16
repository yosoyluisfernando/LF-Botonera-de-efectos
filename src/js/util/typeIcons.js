/**
 * Archivo: typeIcons.js
 * Proposito: iconos visuales para tipos de boton.
 */

const ICONS = {
    audio: '🎵',
    random_folder: '📁',
    time: '🕐',
    temperature: '🌡️',
    humidity: '💧',
};

/** Devuelve markup seguro para pintar un icono de tipo. */
export function typeIcon(type) {
    if (!type) return '';
    const key = ICONS[type] ? type : 'audio';
    return `<span class="type-icon type-icon-${key}" aria-hidden="true">${ICONS[key]}</span>`;
}
