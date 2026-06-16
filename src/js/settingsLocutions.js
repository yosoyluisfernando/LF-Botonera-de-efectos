/**
 * Archivo: settingsLocutions.js
 * Propósito: Control DOM del panel "Hora y Clima".
 * La lógica de carpetas, clima y locuciones vive en Rust.
 */

import { invoke } from './api.js';
import { t } from './i18n.js';
import { locutionsTemplate } from './settingsLocutionsTemplate.js';

let _coords     = { lat: 0, lon: 0 };
let _cityMap    = new Map();
let _loadedCity = '';

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

    document.getElementById('loc-weather-test')
        .addEventListener('click', async () => {
            const out = document.getElementById('loc-weather-now');
            out.textContent = '...';
            try { await saveLocutions(); } catch (e) { out.textContent = String(e); return; }
            _showWeather(true);
        });
}

/** Actualiza el clima desde el listener central de startup.js. */
export function updateWeatherPanel(payload) {
    _paintWeather(payload);
}

/** Rellena el panel con la configuración actual al abrir el modal. */
export function loadLocutionsPanel(config) {
    const l = config.locutions ?? {};
    _coords     = { lat: l.weather_lat ?? 0, lon: l.weather_lon ?? 0 };
    _loadedCity = l.weather_city ?? '';

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

    if (config.weather_module_enabled && l.weather_enabled) _showWeather(false);
}

/** Persiste el panel en Rust. Lo llama el botón Guardar del modal. */
export function saveLocutions() {
    const city = document.getElementById('loc-city').value.trim();
    if (_cityMap.has(city)) {
        _coords = _cityMap.get(city);
    } else if (city !== _loadedCity) {
        _coords = { lat: 0, lon: 0 };
    }
    _loadedCity = city;
    return invoke('set_locution_config', {
        moduleEnabled:  document.getElementById('loc-module').checked,
        timeEnabled:    document.getElementById('loc-time-on').checked,
        timeFolder:     document.getElementById('loc-time-folder').value,
        weatherEnabled: document.getElementById('loc-weather-on').checked,
        tempFolder:     document.getElementById('loc-temp-folder').value,
        humFolder:      document.getElementById('loc-hum-folder').value,
        weatherCity:    city,
        weatherLat:     _coords.lat,
        weatherLon:     _coords.lon,
        weatherUnit:    document.getElementById('loc-unit').value,
    });
}

async function _showWeather(force) {
    const out = document.getElementById('loc-weather-now');
    out.textContent = '...';
    try {
        _paintWeather(await invoke('get_weather_now', { force }));
    } catch (e) {
        const key = `settings_loc.err_${e}`;
        const msg = t(key);
        out.textContent = msg === key ? String(e) : msg;
    }
}

function _paintWeather(w) {
    const out = document.getElementById('loc-weather-now');
    if (!w || !out) return;
    const sym = document.getElementById('loc-unit')?.value === 'imperial' ? '°F' : '°C';
    out.textContent = `🌡️ ${w.temp} ${sym}  💧 ${w.hum} %`;
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
                _cityMap.clear();
                (results ?? []).forEach(r => _appendCity(list, r));
            } catch (_) { /* Sin red: el usuario puede reintentar. */ }
        }, 500);
    });
}

function _appendCity(list, r) {
    const opt = document.createElement('option');
    opt.value = r.label;
    _cityMap.set(r.label, { lat: r.lat, lon: r.lon });
    list.appendChild(opt);
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
