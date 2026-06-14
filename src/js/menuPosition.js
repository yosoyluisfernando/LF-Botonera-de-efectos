/**
 * Archivo: menuPosition.js
 * Propósito: Posiciona menús contextuales dentro de los límites de la ventana.
 * Si el menú se saldría por la derecha o por abajo, se desplaza hacia dentro
 * para que nunca quede cortado.
 */

const MARGIN = 4; // Separación mínima respecto al borde de la ventana (px)

/**
 * Muestra el menú en (x, y) ajustando la posición para que quepa en pantalla.
 * @param {HTMLElement} menu Elemento del menú (se le quita la clase .hidden).
 * @param {number}      x    Coordenada X deseada (ej. cursor).
 * @param {number}      y    Coordenada Y deseada (ej. cursor).
 */
export function placeMenu(menu, x, y) {
    if (!menu) return;
    // Debe estar visible para poder medir su tamaño real
    menu.classList.remove('hidden');
    const rect = menu.getBoundingClientRect();

    const maxX = window.innerWidth  - rect.width  - MARGIN;
    const maxY = window.innerHeight - rect.height - MARGIN;

    menu.style.left = `${Math.max(MARGIN, Math.min(x, maxX))}px`;
    menu.style.top  = `${Math.max(MARGIN, Math.min(y, maxY))}px`;
}
