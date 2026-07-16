/**
 * Archivo: colorTheme.js
 * Propósito: cómo se ve un color de la paleta en el tema actual. Rust envía las
 * dos versiones de cada color (clara y oscura) ya calculadas; aquí solo se elige
 * la que toca.
 */

/** Tema activo, leído del DOM: es donde lo deja `theme.js`. */
export function currentTheme() {
    return document.documentElement.dataset.theme === 'light' ? 'light' : 'dark';
}

export function colorBg(opt) {
    return currentTheme() === 'light' ? opt.lightBg : opt.darkBg;
}

export function colorText(opt) {
    return currentTheme() === 'light' ? opt.lightText : opt.darkText;
}
