/**
 * Archivo: colorAdapter.js
 * Propósito: Derivar colores visibles por tema sin modificar los datos guardados.
 */

const DEFAULTS = {
    buttonBg: '#444444',
    tabBg: '#3a3f44',
    profileBg: '#008c3a',
    text: '#ffffff',
};

/** Aplica color adaptado por tema a un elemento con valores base de usuario. */
export function paintAdaptive(el, bg, text = DEFAULTS.text, role = 'button') {
    if (!el) return;
    el.dataset.baseBg = bg || DEFAULTS[`${role}Bg`] || DEFAULTS.buttonBg;
    el.dataset.baseText = text || DEFAULTS.text;
    el.dataset.colorRole = role;
    _paint(el);
}

/** Repinta todos los elementos adaptativos tras un cambio de tema. */
export function repaintAdaptiveColors(root = document) {
    root.querySelectorAll('[data-base-bg][data-color-role]').forEach(_paint);
}

function _paint(el) {
    const bg = adaptColor(el.dataset.baseBg, el.dataset.colorRole);
    const text = readableText(bg, el.dataset.baseText, el.dataset.colorRole);
    el.style.backgroundColor = bg;
    el.style.color = text;
}

function adaptColor(hex, role) {
    const hsl = rgbToHsl(hexToRgb(hex));
    if (!hsl) return hex || DEFAULTS.buttonBg;

    const theme = document.documentElement.dataset.theme || 'dark';
    const darkBoost = role === 'tab' ? 0.92 : 0.82;
    const lightSat = role === 'tab' ? 0.82 : 0.9;

    if (theme === 'light') {
        hsl.s = clamp(Math.max(hsl.s * 1.35, lightSat), 0, 1);
        hsl.l = clamp(Math.max(hsl.l, 0.5), 0.5, 0.68);
    } else {
        hsl.s = clamp(hsl.s * darkBoost, 0.28, 0.66);
        hsl.l = clamp(Math.min(hsl.l, role === 'tab' ? 0.34 : 0.3), 0.16, 0.34);
    }
    return rgbToHex(hslToRgb(hsl));
}

function readableText(bgHex, preferredHex, role) {
    const bg = hexToRgb(bgHex);
    const preferred = hexToRgb(preferredHex);
    if ((document.documentElement.dataset.theme || 'dark') === 'dark' &&
        (role === 'button' || role === 'tab')) {
        return '#ffffff';
    }
    if (isExplicitTextColor(preferredHex)) return normalizeTextColor(preferredHex);
    if (bg && preferred && contrast(bg, preferred) >= 4.5) return preferredHex;
    return contrast(bg, { r: 0, g: 0, b: 0 }) > contrast(bg, { r: 255, g: 255, b: 255 })
        ? '#111111'
        : '#ffffff';
}

function isExplicitTextColor(hex) {
    const value = String(hex || '').toUpperCase();
    return value === '#FFFFFF' || value === '#111111';
}

function normalizeTextColor(hex) {
    return String(hex).toUpperCase() === '#111111' ? '#111111' : '#ffffff';
}

function hexToRgb(hex) {
    const m = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex || '');
    return m ? { r: parseInt(m[1], 16), g: parseInt(m[2], 16), b: parseInt(m[3], 16) } : null;
}

function rgbToHex({ r, g, b }) {
    return `#${[r, g, b].map(v => Math.round(v).toString(16).padStart(2, '0')).join('')}`;
}

function rgbToHsl(rgb) {
    if (!rgb) return null;
    let { r, g, b } = rgb;
    r /= 255; g /= 255; b /= 255;
    const max = Math.max(r, g, b), min = Math.min(r, g, b);
    let h = 0, s = 0;
    const l = (max + min) / 2;
    if (max !== min) {
        const d = max - min;
        s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
        h = max === r ? (g - b) / d + (g < b ? 6 : 0)
          : max === g ? (b - r) / d + 2
          : (r - g) / d + 4;
        h /= 6;
    }
    return { h, s, l };
}

function hslToRgb({ h, s, l }) {
    if (s === 0) return { r: l * 255, g: l * 255, b: l * 255 };
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    return {
        r: hueToRgb(p, q, h + 1 / 3) * 255,
        g: hueToRgb(p, q, h) * 255,
        b: hueToRgb(p, q, h - 1 / 3) * 255,
    };
}

function hueToRgb(p, q, t) {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1 / 6) return p + (q - p) * 6 * t;
    if (t < 1 / 2) return q;
    if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
    return p;
}

function contrast(a, b) {
    const l1 = luminance(a) + 0.05;
    const l2 = luminance(b) + 0.05;
    return Math.max(l1, l2) / Math.min(l1, l2);
}

function luminance({ r, g, b }) {
    const f = v => (v /= 255) <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
    return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
}

function clamp(n, min, max) {
    return Math.min(max, Math.max(min, n));
}

document.addEventListener('lf-theme-change', () => repaintAdaptiveColors());
