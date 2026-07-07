/**
 * Archivo: settingsLocutions.js
 * Propósito: Control DOM del panel "Hora y Clima".
 * La UI es un control remoto tonto: no geocodifica, no calcula coordenadas ni
 * cachea nada. Toda la lógica ciudad → coordenadas → clima vive en Rust.
 */

import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { locutionsTemplate } from './settingsLocutionsTemplate.js';

/** Construye el panel y conecta sus controles. Llamar una sola vez. */
export function initLocutionsPanel() {
    document.getElementById('s-locuciones').innerHTML = locutionsTemplate();

    _onToggle('loc-module',     'loc-blocks');
    _onToggle('loc-time-on',    'loc-time-fields');
    _onToggle('loc-weather-on', 'loc-weather-fields');
    _wireBrowse('loc-time-browse', 'loc-time-folder');
    _wireBrowse('loc-temp-browse', 'loc-temp-folder');
    _wireBrowse('loc-hum-browse',  'loc-hum-folder');
    _wireCitySearch();

    document.getElementById('loc-time-test').addEventListener('click', () => {
        invoke('play_time_locution', { id: '__loc_test__' }).catch(console.error);
    });
    document.getElementById('loc-weather-test').addEventListener('click', _previewWeather);
    document.getElementById('loc-unit').addEventListener('change', _clearWeather);
}

/** Actualiza el clima desde el listener central de startup.js. */
export function updateWeatherPanel(payload) {
    _paintWeather(payload);
}

/** Rellena el panel con la configuración actual al abrir el modal. */
export function loadLocutionsPanel(config) {
    const l = config.locutions ?? {};

    _set('loc-module',     config.weather_module_enabled, true);
    _set('loc-time-on',    l.time_enabled, true);
    _set('loc-weather-on', l.weather_enabled, true);
    _set('loc-time-folder', l.time_folder);
    _set('loc-temp-folder', l.temp_folder);
    _set('loc-hum-folder',  l.hum_folder);
    _set('loc-city',        l.weather_city);
    _set('loc-unit',        l.weather_unit || 'metric');

    _sync('loc-module',     'loc-blocks');
    _sync('loc-time-on',    'loc-time-fields');
    _sync('loc-weather-on', 'loc-weather-fields');
    _clearWeather();

    if (config.weather_module_enabled && l.weather_enabled) _showSaved();
}

/** Persiste el panel en Rust. Lo llama el botón Guardar del modal.
 *  No envía coordenadas: Rust las resuelve desde la ciudad. */
export function saveLocutions() {
    return invoke('set_locution_config', {
        moduleEnabled:  document.getElementById('loc-module').checked,
        timeEnabled:    document.getElementById('loc-time-on').checked,
        timeFolder:     document.getElementById('loc-time-folder').value,
        weatherEnabled: document.getElementById('loc-weather-on').checked,
        tempFolder:     document.getElementById('loc-temp-folder').value,
        humFolder:      document.getElementById('loc-hum-folder').value,
        weatherCity:    document.getElementById('loc-city').value.trim(),
        weatherUnit:    document.getElementById('loc-unit').value,
    });
}

/** Botón "Comprobar": prueba la ciudad escrita sin guardar ni exigir carpetas. */
async function _previewWeather() {
    const city = document.getElementById('loc-city').value.trim();
    if (!city) { _status(t('settings_loc.err_no_city'), 'error'); return; }
    _paintWeather(null);
    _status(t('settings_loc.searching'), '');
    try {
        const w = await invoke('preview_weather', { city, unit: _unit() });
        _paintWeather(w);
        _status(`✓ ${w.label}`, 'ok');
    } catch (e) {
        _paintWeather(null);
        _status(_errMessage(e), 'error');
    }
}

/** Carga el clima guardado al abrir el panel (respeta el estado persistido). */
async function _showSaved() {
    try {
        _paintWeather(await invoke('get_weather_now', { force: false }));
    } catch (_) { /* Sin red o sin ciudad: el usuario puede Comprobar. */ }
}

function _paintWeather(w) {
    const temp = document.getElementById('loc-weather-temp');
    const hum  = document.getElementById('loc-weather-hum');
    if (!temp || !hum) return;
    const sym = _unit() === 'imperial' ? '°F' : '°C';
    temp.textContent = w ? `🌡️ ${w.temp} ${sym}` : `🌡️ -- ${sym}`;
    hum.textContent  = w ? `💧 ${w.hum} %`        : '💧 -- %';
}

function _clearWeather() {
    _paintWeather(null);
    _status('', '');
}

/** Pinta la línea de estado; `kind` ('', 'ok', 'error') colorea vía CSS. */
function _status(msg, kind) {
    const el = document.getElementById('loc-weather-status');
    if (!el) return;
    el.textContent = msg;
    el.dataset.kind = kind;
}

function _unit() {
    return document.getElementById('loc-unit')?.value || 'metric';
}

/** Traduce la clave de error de Rust; si no hay traducción, muestra el texto. */
function _errMessage(e) {
    const key = `settings_loc.err_${e}`;
    const msg = t(key);
    return msg === key ? String(e) : msg;
}

function _wireCitySearch() {
    let timer;
    document.getElementById('loc-city').addEventListener('input', e => {
        clearTimeout(timer);
        const query = e.target.value.trim();
        if (query.length < 3) return;
        timer = setTimeout(async () => {
            try {
                const results = await invoke('search_city', { query });
                const list = document.getElementById('loc-city-list');
                list.innerHTML = '';
                (results ?? []).forEach(r => {
                    const opt = document.createElement('option');
                    opt.value = r.label;
                    list.appendChild(opt);
                });
            } catch (_) { /* Sin red: el usuario puede reintentar o Comprobar. */ }
        }, 500);
    });
}

function _wireBrowse(btnId, inputId) {
    document.getElementById(btnId).addEventListener('click', async () => {
        try {
            const folder = await invoke('pick_named_folder');
            if (folder?.path) document.getElementById(inputId).value = folder.path;
        } catch (_) { /* Usuario canceló. */ }
    });
}

function _onToggle(checkId, blockId) {
    document.getElementById(checkId).addEventListener('change', () => _sync(checkId, blockId));
}

function _sync(checkId, blockId) {
    const on = document.getElementById(checkId).checked;
    document.getElementById(blockId).classList.toggle('hidden', !on);
}

function _set(id, value, isCheck = false) {
    const el = document.getElementById(id);
    if (isCheck) el.checked = !!value;
    else         el.value   = value ?? '';
}
