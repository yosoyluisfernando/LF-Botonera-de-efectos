/** Colores y degradados adaptados de Danilo Wenner (7c1f1ae).
 * Solo representación; Rust valida y conserva el estilo. */
import { paintAdaptive } from '../util/colorAdapter.js';
const DIRECTIONS = {
    horizontal: 'to right', vertical: 'to bottom',
    diagonal: 'to bottom right', diagonal_inv: 'to top right',
};
export function gradientCss(direction, from, to) {
    const dir = DIRECTIONS[direction];
    return dir && to ? `linear-gradient(${dir}, ${from}, ${to})` : '';
}
export function paintButtonStyle(el, data) {
    const style = data?.style ?? {};
    const exact = Boolean(style.exact || style.gradient || style.text2);
    el.classList.toggle('style-exact', exact);
    el.style.removeProperty('--btn-gradient');
    if (exact) {
        delete el.dataset.baseBg; delete el.dataset.baseText; delete el.dataset.colorRole;
        el.style.backgroundColor = data.color_bg || '#444444';
        el.style.color = data.color_text || '#ffffff';
        const bg = gradientCss(style.gradient, data.color_bg, style.bg2);
        if (bg) el.style.setProperty('--btn-gradient', bg);
    } else paintAdaptive(el, data.color_bg, data.color_text, 'button');
    const label = el.querySelector('.label, .fixed-panel-name');
    const text = gradientCss(style.gradient || 'horizontal', data.color_text, style.text2);
    label?.classList.toggle('text-gradient', Boolean(exact && style.text2));
    label?.style.removeProperty('--btn-text-gradient');
    if (label && exact && style.text2) label.style.setProperty('--btn-text-gradient', text);
}
