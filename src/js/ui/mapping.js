/**
 * Archivo: mapping.js
 * Propósito: Modo Mapeo (Fase 5). Replica la maqueta: al activarlo se muestra
 * un banner, el cursor cambia y el siguiente clic sobre un botón o pestaña
 * abre el modal de captura de tecla. ESC sale del modo en cualquier momento.
 * La persistencia del atajo se delega a Rust (Regla 4).
 */

import { t } from '../util/i18n.js';
import { invoke, listen } from '../bridge/api.js';
import { initKeyInputs } from '../util/keyInputs.js';
import { invokeShortcutSave } from './shortcutSave.js';
import { getMidiField, initMidiField, setMidiField, cancelMidiCapture, hasActiveMidiCapture } from './midiControls.js';

let _onRefresh = null;
let _target    = null; // ButtonData o PaletaData según _type
let _type      = '';   // 'button' | 'tab'
let _profileId = '';
let _wired     = false;
let _midiRequest = 0;

/** Inicializa el botón de entrada y el modal de captura. Llamar una vez. */
export function initMapping(onRefresh) {
    _onRefresh = onRefresh;
    if (_wired) return;
    _wired = true;

    document.getElementById('btn-enter-mapping').addEventListener('click', () => {
        document.getElementById('settings-modal').classList.add('hidden');
        document.body.classList.add('mapping-mode');
        document.getElementById('mapping-banner').classList.remove('hidden');
    });

    document.getElementById('btn-cancel-capture').addEventListener('click', () => {
        exitMapping();
    });

    document.getElementById('btn-save-capture').addEventListener('click', _saveCapture);
    initMidiField('capture-midi-input', 'btn-capture-midi', 'btn-clear-midi');
    const refresh = () => {
        if (!document.getElementById('capture-modal').classList.contains('hidden')) void _refreshMidi();
    };
    listen('midi-devices-changed', refresh).catch(console.error);
    window.addEventListener('focus', refresh);
    document.getElementById('capture-midi-section').addEventListener('toggle', event => {
        if (!event.currentTarget.open) void cancelMidiCapture();
    });

    // ESC sale del modo mapeo con prioridad máxima (fase de captura)
    document.addEventListener('keydown', e => {
        if (e.key === 'Escape' && isMapping()) { e.stopPropagation(); exitMapping(); }
    }, true);
}

/** ¿Está activo el modo mapeo? Lo consultan grid.js y tabs.js en sus clics. */
export function isMapping() {
    return document.body.classList.contains('mapping-mode');
}

/** Abre la captura de tecla para un botón de la cuadrícula. */
export function captureButton(btnData) {
    _target = btnData; _type = 'button';
    _openCapture(`${t('mapping.target_button')} ${btnData.index}: ${btnData.name || btnData.label}`,
                 btnData.shortcut);
}

/** Abre la captura de tecla para una pestaña. */
export function captureTab(paleta, profileId) {
    _target = paleta; _type = 'tab'; _profileId = profileId;
    _openCapture(`${t('mapping.target_tab')}: ${paleta.nombre}`, paleta.shortcut);
}

function _openCapture(label, current) {
    document.getElementById('capture-target-name').textContent = label;
    const input = document.getElementById('capture-key-input');
    input.value = current || '';
    setMidiField('capture-midi-input', _target?.midi);
    initKeyInputs(document.getElementById('capture-modal'));
    document.getElementById('capture-modal').classList.remove('hidden');
    const section = document.getElementById('capture-midi-section');
    section.open = false; section.classList.add('hidden');
    void _refreshMidi();
    input.focus();
}

async function _saveCapture() {
    if (hasActiveMidiCapture()) return;
    const key = document.getElementById('capture-key-input').value;
    try {
        if (_type === 'button') {
            await invokeShortcutSave('update_button_data', {
                index: _target.index, label: _target.label,
                colorBg: _target.color_bg, colorText: _target.color_text,
                shortcut: key,
                midi: getMidiField('capture-midi-input'),
            });
        } else if (_type === 'tab') {
            await invokeShortcutSave('update_paleta_meta', {
                profileId: _profileId, paletaId: _target.id,
                nombre: _target.nombre, rows: _target.rows, cols: _target.cols,
                shortcut: key,
                midi: getMidiField('capture-midi-input'),
            });
        }
        exitMapping();
        _onRefresh?.();
    } catch (e) { console.error('Error al guardar atajo:', e); }
}

/** Sale del modo mapeo y cierra el modal de captura. */
export function exitMapping() {
    ++_midiRequest;
    void cancelMidiCapture();
    _target = null; _type = '';
    document.body.classList.remove('mapping-mode');
    document.getElementById('mapping-banner').classList.add('hidden');
    document.getElementById('capture-modal').classList.add('hidden');
}

/** Rust decide disponibilidad: MIDI habilitado y al menos una entrada conectada. */
async function _refreshMidi() {
    const request = ++_midiRequest;
    let available = false;
    try { available = Boolean((await invoke('get_shortcut_editor_state')).midi_available); }
    catch (error) { console.error('Error al consultar MIDI:', error); }
    if (request !== _midiRequest) return;
    const section = document.getElementById('capture-midi-section');
    section.classList.toggle('hidden', !available);
    const capture = document.getElementById('btn-capture-midi');
    capture.dataset.unavailable = String(!available);
    capture.disabled = !available || hasActiveMidiCapture();
    if (!available) { section.open = false; await cancelMidiCapture(); }
}
