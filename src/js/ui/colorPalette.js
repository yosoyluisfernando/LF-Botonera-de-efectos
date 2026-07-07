/**
 * Archivo: colorPalette.js
 * Proposito: selector modal de colores entregados por Rust.
 */

import { invoke } from '../bridge/api.js';

let _palette = null;
let _target = null;
let _choice = null;
let _manualText = false;
const _registered = [];

export function attachPalette(bgInput, textInput, role = 'button') {
    if (!bgInput || bgInput.dataset.paletteReady) return;
    bgInput.dataset.paletteReady = '1';
    bgInput.classList.add('managed-color-input');
    if (textInput) textInput.classList.add('managed-color-input');
    _registered.push({ bgInput, textInput, role });
    _wireColorInput(bgInput, bgInput, textInput, role);
    if (textInput) _wireColorInput(textInput, bgInput, textInput, role);
    refreshColorInputs();
}

export async function refreshColorInputs() {
    const palette = await _loadPalette();
    _registered.forEach(r => _paintInputs(r, palette));
}

function _wireColorInput(input, bgInput, textInput, role) {
    input.readOnly = true;
    input.addEventListener('pointerdown', e => {
        e.preventDefault();
        openColorPicker(bgInput, textInput, role);
    });
    input.addEventListener('keydown', e => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            openColorPicker(bgInput, textInput, role);
        }
    });
}

export async function openColorPicker(bgInput, textInput, role = 'button') {
    _target = { bgInput, textInput, role };
    _choice = null;
    _manualText = false;
    const palette = await _loadPalette();
    _paintInputs(_target, palette);
    _renderPalette(palette);
    _selectByBase(bgInput.value, palette[0]);
    _syncTextOptions();
    document.getElementById('color-picker-modal')?.classList.remove('hidden');
}

export function initColorPicker() {
    document.getElementById('color-picker-apply')?.addEventListener('click', _apply);
    document.getElementById('color-picker-cancel')?.addEventListener('click', _hide);
    document.getElementById('color-picker-modal')?.addEventListener('click', e => {
        if (e.target?.id === 'color-picker-modal') _hide();
    });
    document.querySelectorAll('input[name="color-picker-text"]').forEach(r => {
        r.addEventListener('change', () => {
            if (!_choice) return;
            _manualText = true;
            _choice.text = r.value;
            _paintPreview();
        });
    });
}

async function _loadPalette() {
    if (_palette) return _palette;
    _palette = await invoke('get_color_palette').catch(() => []);
    return _palette;
}

function _renderPalette(palette) {
    const grid = document.getElementById('color-picker-grid');
    if (!grid) return;
    grid.innerHTML = '';
    palette.forEach(opt => {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'color-choice';
        btn.dataset.base = opt.base;
        btn.style.backgroundColor = _bg(opt);
        btn.style.color = _text(opt);
        btn.textContent = 'A';
        btn.addEventListener('click', () => _select(opt));
        grid.appendChild(btn);
    });
}

function _selectByBase(base, fallback) {
    const found = _palette?.find(c => c.base.toLowerCase() === String(base).toLowerCase());
    _select(found || fallback);
}

function _select(opt) {
    if (!opt) return;
    _choice = { base: opt.base, bg: _bg(opt), text: _textForSelection(opt), option: opt };
    document.querySelectorAll('.color-choice').forEach(btn => {
        btn.classList.toggle('selected', btn.dataset.base === opt.base);
    });
    _paintPreview();
}

function _syncTextOptions() {
    const dark = _theme() === 'dark' && ['button', 'tab'].includes(_target?.role);
    const black = document.getElementById('color-text-black');
    if (!black) return;
    black.disabled = dark;
    document.getElementById('color-text-white').checked = dark || _choice?.text !== '#111111';
    black.checked = !dark && _choice?.text === '#111111';
}

function _paintPreview() {
    if (!_choice) return;
    const preview = document.getElementById('color-picker-preview');
    if (preview) {
        preview.style.backgroundColor = _choice.bg;
        preview.style.color = _choice.text;
    }
    _syncTextOptions();
}

function _apply() {
    if (!_choice || !_target) return;
    _target.bgInput.value = _choice.base;
    if (_target.textInput) _target.textInput.value = _choice.text;
    _paintInputs(_target, _palette || []);
    _hide();
}

function _hide() {
    document.getElementById('color-picker-modal')?.classList.add('hidden');
}

function _bg(opt) {
    return _theme() === 'light' ? opt.lightBg : opt.darkBg;
}

function _text(opt) {
    return _theme() === 'light' ? opt.lightText : opt.darkText;
}

function _textForSelection(opt) {
    if (_theme() === 'dark' && ['button', 'tab'].includes(_target?.role)) return opt.darkText;
    if (_manualText) return _selectedTextValue();
    return _text(opt);
}

function _selectedTextValue() {
    const checked = document.querySelector('input[name="color-picker-text"]:checked');
    return checked?.value || '#FFFFFF';
}

function _theme() {
    return document.documentElement.dataset.theme || 'dark';
}

function _paintInputs(target, palette) {
    const opt = palette.find(c => c.base.toLowerCase() === String(target.bgInput.value).toLowerCase());
    const bg = opt ? _bg(opt) : target.bgInput.value;
    target.bgInput.style.backgroundColor = bg;
    target.bgInput.style.color = 'transparent';
    if (target.textInput) {
        target.textInput.style.backgroundColor = target.textInput.value || '#ffffff';
        target.textInput.style.color = 'transparent';
    }
}

document.addEventListener('lf-theme-change', () => {
    refreshColorInputs();
    if (_target && !document.getElementById('color-picker-modal')?.classList.contains('hidden')) {
        openColorPicker(_target.bgInput, _target.textInput, _target.role);
    }
});
