/**
 * Archivo: buttonStyle.js
 * Propósito: pintar la presentación avanzada de un botón (`style` en Rust):
 * tamaño de texto propio o de la pestaña, colores exactos y degradados.
 * Solo dibuja: los valores se validan y guardan en Rust (model/button_style.rs).
 */
import { paintAdaptive } from '../util/colorAdapter.js';

export const FONT_MIN = 8;
export const FONT_MAX = 48;

const DIRECTIONS = {
    horizontal: 'to right',
    vertical: 'to bottom',
    diagonal: 'to bottom right',
    diagonal_inv: 'to top right',
};

/** CSS de un degradado, o '' si es color sólido. */
export function gradientCss(direction, from, to) {
    const dir = DIRECTIONS[direction];
    return dir && to ? `linear-gradient(${dir}, ${from}, ${to})` : '';
}

/**
 * Pinta colores y tamaño. Sin estilo propio usa el color adaptado al tema de
 * siempre (paintAdaptive); con color exacto o degradado pinta tal cual.
 * @param {HTMLElement} el      Celda (.grid-item o .fixed-panel-item).
 * @param {object}      data    ButtonData.
 * @param {number}      tabFont Tamaño predeterminado de la pestaña (0 = global).
 */
export function paintButtonStyle(el, data, tabFont = 0) {
    const style = data?.style ?? {};
    const exact = Boolean(style.exact || style.gradient);
    el.classList.toggle('style-exact', exact);
    if (exact) {
        // Sin data-base-bg: el repintado por cambio de tema no lo toca.
        delete el.dataset.baseBg;
        delete el.dataset.colorRole;
        el.style.backgroundColor = data.color_bg || '#444444';
        el.style.color = data.color_text || '#ffffff';
        const bg = gradientCss(style.gradient, data.color_bg, style.bg2);
        if (bg) el.style.setProperty('--btn-gradient', bg);
        else el.style.removeProperty('--btn-gradient');
    } else {
        el.style.removeProperty('--btn-gradient');
        paintAdaptive(el, data.color_bg, data.color_text, 'button');
    }
    const label = el.querySelector('.label');
    const textGradient = gradientCss(style.gradient || 'horizontal', data.color_text, style.text2);
    label?.classList.toggle('text-gradient', Boolean(exact && style.text2));
    if (label && exact && style.text2) label.style.setProperty('--btn-text-gradient', textGradient);
    applyFontSize(el, style.font_size || tabFont || 0);
}

/** 0 = deja el tamaño global de Ajustes. */
export function applyFontSize(el, size) {
    const valid = size >= FONT_MIN && size <= FONT_MAX;
    el.classList.toggle('has-font-size', valid);
    if (valid) el.style.setProperty('--btn-font-size', `${size}px`);
    else el.style.removeProperty('--btn-font-size');
}

/**
 * Encoge, solo en pantalla, el texto que no cabe en su botón: el tamaño
 * guardado no cambia. Una pasada por celda al dibujar o redimensionar; con
 * rejillas de decenas de botones son unas pocas mediciones.
 */
export function fitLabels(root) {
    root?.querySelectorAll('.grid-item[data-id]').forEach(cell => {
        const label = cell.querySelector('.label');
        if (!label) return;
        label.style.fontSize = '';
        let size = parseFloat(getComputedStyle(cell).fontSize) || 14;
        const fits = () => label.scrollHeight <= cell.clientHeight - 4 &&
            label.scrollWidth <= cell.clientWidth - 2;
        while (!fits() && size > FONT_MIN) {
            size -= 1;
            label.style.fontSize = `${size}px`;
        }
    });
}

let _observer = null;
/** Reajusta al cambiar el tamaño de la ventana o del panel (agrupado por frame). */
export function watchFit(root) {
    if (_observer || !root || typeof ResizeObserver === 'undefined') return;
    let pending = false;
    _observer = new ResizeObserver(() => {
        if (pending) return;
        pending = true;
        requestAnimationFrame(() => { pending = false; fitLabels(root); });
    });
    _observer.observe(root);
}
