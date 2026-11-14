/** Editor unificado de colores. Base: editStyle de Danilo Wenner (7c1f1ae).
 * JS sincroniza el borrador visual; Rust valida y persiste al aceptar. */
import { invoke } from '../bridge/api.js';
import { initColorFavorites } from './colorFavorites.js';
import { appAlert } from './appDialog.js';
import { t } from '../util/i18n.js';
import { paintButtonStyle } from './buttonStyle.js';
import { paintButtonContent } from './buttonVisual.js';
import { currentEditVisual } from './visualSelector.js';
import { refreshColorInputs } from './colorPalette.js';
const $ = id => document.getElementById(id);
const PAIRS = ['bg', 'text', 'bg2', 'text2'];
let wired = false;
let mode = 'preset';
let preset = {};
export function normalizeHex(raw) {
    let v = String(raw || '').trim().replace(/^#/, '');
    if (/^[0-9a-f]{3}$/i.test(v)) v = v.split('').map(c => c + c).join('');
    return /^[0-9a-f]{6}$/i.test(v) ? `#${v.toUpperCase()}` : null;
}
export async function initEditStyle(data) {
    wire();
    await loadStyle(data);
    await initColorFavorites({
        read: () => ({ color_bg: $('edit-bg-color').value, color_text: $('edit-text-color').value, style: currentEditStyle() }),
        invalid: async () => { const error = editStyleError(); if (error) await appAlert(error); return Boolean(error); },
        apply: async favorite => {
            $('edit-bg-color').value = favorite.color_bg; $('edit-text-color').value = favorite.color_text;
            await loadStyle(favorite);
            $('edit-fill').value = 'favorites'; toggleRows();
        },
    });
}
async function loadStyle(data) {
    const style = data?.style ?? {};
    mode = style.gradient || (style.exact || style.text2 ? 'solid' : 'preset');
    $('edit-fill').value = mode;
    $('edit-custom-bg2').value = style.bg2 || '#000000';
    $('edit-custom-text2').value = style.text2 || '#FFD700';
    $('edit-text-gradient').checked = Boolean(style.text2);
    preset = { bg: $('edit-bg-color').value, text: $('edit-text-color').value };
    if (mode !== 'preset') {
        const palette = await invoke('get_color_palette');
        const match = palette.find(c => c.base.toLowerCase() === preset.bg.toLowerCase()) || palette[0];
        if (match) preset = { bg: match.base, text: match.darkText || '#FFFFFF' };
    }
    syncEditStyleFromInputs();
    await refreshColorInputs();
}
export function syncEditStyleFromInputs() {
    for (const key of ['bg', 'text']) {
        $('edit-custom-' + key).value = $('edit-' + key + '-color').value;
    }
    for (const key of PAIRS) {
        $('edit-' + key + '-hex').value = $('edit-custom-' + key).value.toUpperCase();
        $('edit-' + key + '-hex').setAttribute('aria-invalid', 'false');
    }
    toggleRows(); preview();
}
export function currentEditStyle() {
    if (mode === 'preset') return {};
    return {
        gradient: mode === 'solid' ? '' : mode,
        bg2: mode === 'solid' ? '' : $('edit-custom-bg2').value,
        text2: $('edit-text-gradient').checked ? $('edit-custom-text2').value : '',
        exact: true,
    };
}
export function editStyleError() {
    if (mode === 'preset') return null;
    const keys = ['bg', 'text'];
    if (mode !== 'solid') keys.push('bg2');
    if ($('edit-text-gradient').checked) keys.push('text2');
    for (const key of keys) {
        const field = $('edit-' + key + '-hex');
        const value = normalizeHex(field.value);
        if (!value) { field.setAttribute('aria-invalid', 'true'); field.focus(); return t('colors.hex_invalid'); }
    }
    return null;
}
function wire() {
    if (wired) return;
    wired = true;
    $('edit-fill').addEventListener('change', () => {
        if ($('edit-fill').value === 'favorites') { toggleRows(); return; }
        if (mode === 'preset') preset = { bg: $('edit-bg-color').value, text: $('edit-text-color').value };
        mode = $('edit-fill').value;
        for (const key of ['bg', 'text']) {
            $('edit-' + key + '-color').value = mode === 'preset' ? preset[key] : $('edit-custom-' + key).value;
        }
        refreshColorInputs(); toggleRows(); preview();
    });
    for (const key of ['bg', 'text']) $('edit-' + key + '-color').addEventListener('input', syncEditStyleFromInputs);
    for (const key of PAIRS) {
        const picker = $('edit-custom-' + key), hex = $('edit-' + key + '-hex');
        picker.addEventListener('input', () => {
            hex.value = picker.value.toUpperCase(); hex.setAttribute('aria-invalid', 'false');
            setBase(key, picker.value); preview();
        });
        hex.addEventListener('input', () => {
            const value = normalizeHex(hex.value);
            hex.setAttribute('aria-invalid', String(!value));
            if (!value) return;
            picker.value = value; setBase(key, value); preview();
        });
    }
    $('edit-text-gradient').addEventListener('change', () => { toggleRows(); preview(); });
    $('edit-name').addEventListener('input', preview);
    document.addEventListener('lf-edit-visual-change', preview);
    document.addEventListener('lf-theme-change', preview);
}
function setBase(key, value) {
    if (key === 'bg' || key === 'text') $('edit-' + key + '-color').value = value;
}
function toggleRows() {
    const browsing = $('edit-fill').value === 'favorites';
    $('edit-favorite-choices').classList.toggle('hidden', !browsing);
    $('edit-preset-colors').classList.toggle('hidden', browsing || mode !== 'preset');
    $('edit-custom-colors').classList.toggle('hidden', browsing || mode === 'preset');
    $('edit-second-bg').classList.toggle('hidden', mode === 'solid' || mode === 'preset');
    $('edit-second-text').classList.toggle('hidden', !$('edit-text-gradient').checked);
}
function preview() {
    const el = $('edit-style-preview');
    if (!el || !wired) return;
    const data = {
        name: $('edit-name').value || t('colors.preview_sample'),
        color_bg: $('edit-bg-color').value, color_text: $('edit-text-color').value,
        style: currentEditStyle(), visual: currentEditVisual(),
    };
    paintButtonContent(el.querySelector('.label'), data);
    paintButtonStyle(el, data);
}
