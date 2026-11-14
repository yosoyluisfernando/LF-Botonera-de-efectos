/**
 * Archivo: editStyle.js
 * Propósito: controles avanzados del editor de botón: códigos HEX, degradado
 * de dos colores (fondo y texto), color exacto, tamaño de texto propio y vista
 * previa en vivo. Rust valida y guarda (`style` en update_button_data).
 */
import { t } from '../util/i18n.js';
import { refreshColorInputs } from './colorPalette.js';
import { paintButtonStyle } from './buttonStyle.js';
import { bindFontControl, globalFontPx } from './fontSizeControl.js';

const $ = id => document.getElementById(id);
const PAIRS = [
    ['edit-bg-color', 'edit-bg-hex', true],
    ['edit-text-color', 'edit-text-hex', true],
    ['edit-bg2-color', 'edit-bg2-hex', false],
    ['edit-text2-color', 'edit-text2-hex', false],
];

let _font = null;
let _tabFont = 0;
let _wired = false;

/** Normaliza lo que se escribe o pega: `ff0000`, `#F00` → `#FF0000`. */
export function normalizeHex(raw) {
    let v = String(raw || '').trim().replace(/^#/, '');
    if (/^[0-9a-f]{3}$/i.test(v)) v = v.split('').map(c => c + c).join('');
    return /^[0-9a-f]{6}$/i.test(v) ? `#${v.toUpperCase()}` : null;
}

/**
 * Carga el estilo del botón en el formulario.
 * @param {object} btnData  ButtonData (o null para una celda vacía).
 * @param {number} tabFont  Tamaño predeterminado de la pestaña (0 = global).
 */
export function initEditStyle(btnData, tabFont = 0) {
    _wire();
    _tabFont = tabFont;
    const style = btnData?.style ?? {};
    $('edit-gradient').value = style.gradient || '';
    $('edit-bg2-color').value = style.bg2 || '#000000';
    $('edit-text-gradient').checked = Boolean(style.text2);
    $('edit-text2-color').value = style.text2 || '#FFD700';
    $('edit-color-exact').checked = Boolean(style.exact);
    _font.set(style.font_size || 0);
    syncEditStyleFromInputs();
}

/** Tras cambiar los colores desde fuera (archivo nuevo, color sugerido). */
export function syncEditStyleFromInputs() {
    PAIRS.forEach(([color, hex]) => {
        $(hex).value = normalizeHex($(color).value) ?? '';
        $(hex).classList.remove('invalid');
    });
    _toggleRows();
    _preview();
}

/** El `style` a enviar a Rust. */
export function currentEditStyle() {
    const gradient = $('edit-gradient').value;
    const textGradient = $('edit-text-gradient').checked;
    return {
        font_size: _font?.get() ?? 0,
        gradient,
        bg2: gradient ? normalizeHex($('edit-bg2-color').value) ?? '' : '',
        text2: textGradient ? normalizeHex($('edit-text2-color').value) ?? '' : '',
        exact: $('edit-color-exact').checked || Boolean(gradient) || textGradient,
    };
}

/** null si todo es válido; si no, el mensaje a mostrar. */
export function editStyleError() {
    const bad = PAIRS.some(([, hex]) => {
        const el = $(hex);
        return el.offsetParent !== null && el.value && !normalizeHex(el.value);
    });
    return bad ? t('colors.hex_invalid') : null;
}

function _wire() {
    if (_wired) return;
    _wired = true;
    _font = bindFontControl('edit', () => _tabFont || globalFontPx(),
        () => (_tabFont ? 'font_size.from_tab' : 'font_size.from_global'), () => _preview());
    PAIRS.forEach(([color, hex, palette]) => {
        $(hex).addEventListener('input', () => _onHex(color, hex, palette));
        $(color).addEventListener('input', () => {
            $(hex).value = normalizeHex($(color).value) ?? '';
            $(hex).classList.remove('invalid');
            // Elegir de la paleta es elegir el color adaptado al tema.
            if (palette) $('edit-color-exact').checked = false;
            _preview();
        });
    });
    $('edit-gradient').addEventListener('change', () => { _toggleRows(); _preview(); });
    $('edit-text-gradient').addEventListener('change', () => { _toggleRows(); _preview(); });
    $('edit-color-exact').addEventListener('change', _preview);
    $('edit-name').addEventListener('input', _preview);
}

function _onHex(color, hex, palette) {
    const value = normalizeHex($(hex).value);
    $(hex).classList.toggle('invalid', !value && $(hex).value.trim() !== '');
    if (!value) return;
    $(color).value = value;
    // Quien escribe un HEX quiere ESE color, no su versión adaptada al tema.
    if (palette) {
        $('edit-color-exact').checked = true;
        refreshColorInputs();
    }
    _preview();
}

function _toggleRows() {
    const gradient = Boolean($('edit-gradient').value);
    $('edit-bg2-col').style.visibility = gradient ? 'visible' : 'hidden';
    $('edit-text2-pair').style.display = $('edit-text-gradient').checked ? '' : 'none';
    const exact = $('edit-color-exact');
    exact.disabled = gradient || $('edit-text-gradient').checked;
    if (exact.disabled) exact.checked = true;
}

function _preview() {
    const el = $('edit-style-preview');
    if (!el) return;
    el.querySelector('.button-visual-text').textContent = $('edit-name').value || 'ABC';
    paintButtonStyle(el, {
        color_bg: $('edit-bg-color').value,
        color_text: $('edit-text-color').value,
        style: currentEditStyle(),
    }, _tabFont);
    const size = currentEditStyle().font_size || _tabFont || globalFontPx();
    el.style.fontSize = `${size}px`;
}
