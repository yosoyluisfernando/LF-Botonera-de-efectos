/**
 * Archivo: settingsLocutions.js
 * Propósito: Panel "Hora y Clima" del modal de Configuración (Fase 6).
 * Inyecta su contenido en #s-locuciones (patrón del wizard) y delega toda
 * la lógica (carpetas, API de clima, locuciones) al motor Rust (Regla 4).
 * Dos bloques independientes (hora / clima) para ahorrar recursos.
 */

import { invoke, listen } from './api.js';
import { t } from './i18n.js';

let _coords     = { lat: 0, lon: 0 }; // Coordenadas de la ciudad elegida
let _cityMap    = new Map();          // label → {lat, lon} de las sugerencias
let _loadedCity = '';                 // Ciudad tal como estaba al abrir el panel

/** Construye el panel y conecta sus controles. Llamar una sola vez. */
export function initLocutionsPanel() {
    document.getElementById('s-locuciones').innerHTML = _template();

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

    // Probar guarda PRIMERO lo escrito en el panel (ciudad/unidad/carpetas):
    // sin esto, la prueba leía la configuración vieja y siempre fallaba con
    // "ciudad no configurada" hasta que el usuario pulsara Guardar.
    document.getElementById('loc-weather-test')
        .addEventListener('click', async () => {
            const out = document.getElementById('loc-weather-now');
            out.textContent = '...';
            try { await saveLocutions(); } catch (e) { out.textContent = String(e); return; }
            _showWeather(true);
        });

    // El hilo Rust refresca cada 15 min: reflejar el valor nuevo si llega
    listen('weather-updated', e => _paintWeather(e.payload));
}

/** Consulta el clima (caché o forzado) y lo pinta en el panel. */
async function _showWeather(force) {
    const out = document.getElementById('loc-weather-now');
    out.textContent = '...';
    try {
        _paintWeather(await invoke('get_weather_now', { force }));
    } catch (e) {
        // Rust devuelve códigos de error traducibles (Regla 6)
        const key = `settings_loc.err_${e}`;
        const msg = t(key);
        out.textContent = msg === key ? String(e) : msg;
    }
}

function _paintWeather(w) {
    if (!w) return;
    const sym = document.getElementById('loc-unit').value === 'imperial' ? '°F' : '°C';
    document.getElementById('loc-weather-now').textContent =
        `🌡️ ${w.temp} ${sym}  💧 ${w.hum} %`;
}

/** Rellena el panel con la configuración actual (al abrir el modal). */
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

    // Mostrar el clima actual al abrir (usa caché; no fuerza la red)
    if (config.weather_module_enabled && l.weather_enabled) _showWeather(false);
}

/** Persiste el panel en Rust. Lo llama el botón Guardar del modal. */
export function saveLocutions() {
    const city = document.getElementById('loc-city').value.trim();
    if (_cityMap.has(city)) {
        _coords = _cityMap.get(city);
    } else if (city !== _loadedCity) {
        // Ciudad escrita a mano y distinta a la guardada: invalidar las
        // coordenadas viejas para que Rust geocodifique la nueva
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

// ─── Internos ─────────────────────────────────────────────────────────────────

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
                (results ?? []).forEach(r => {
                    const opt = document.createElement('option');
                    opt.value = r.label;
                    _cityMap.set(r.label, { lat: r.lat, lon: r.lon });
                    list.appendChild(opt);
                });
            } catch (_) { /* Sin red: el usuario puede reintentar */ }
        }, 500);
    });
}

function _wireBrowse(btnId, inputId) {
    document.getElementById(btnId).addEventListener('click', async () => {
        try {
            const folder = await invoke('pick_folder');
            if (folder) document.getElementById(inputId).value = folder;
        } catch (_) { /* Usuario canceló */ }
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

function _template() {
    return `
    <label><input type="checkbox" id="loc-module"> <b>${t('settings_loc.module')}</b></label>
    <p class="hint">${t('settings_loc.module_hint')}</p>
    <div id="loc-blocks" class="hidden">
      <hr class="modal-divider">
      <label><input type="checkbox" id="loc-time-on"> <b>🕐 ${t('settings_loc.time_block')}</b></label>
      <div id="loc-time-fields" class="hidden" style="margin-top:8px">
        <label>${t('settings_loc.time_folder')}</label>
        <div class="file-input-group">
          <input type="text" id="loc-time-folder" readonly>
          <button id="loc-time-browse">...</button>
          <button id="loc-time-test" title="${t('settings_loc.test')}">▶</button>
        </div>
        <p class="hint">${t('settings_loc.time_hint')}</p>
      </div>
      <hr class="modal-divider">
      <label><input type="checkbox" id="loc-weather-on"> <b>🌡️ ${t('settings_loc.weather_block')}</b></label>
      <div id="loc-weather-fields" class="hidden" style="margin-top:8px">
        <label>${t('settings_loc.city')}</label>
        <input type="text" id="loc-city" list="loc-city-list" autocomplete="off"
               spellcheck="false" style="margin-bottom:8px">
        <datalist id="loc-city-list"></datalist>
        <div class="row">
          <div class="col">
            <label>${t('settings_loc.unit')}</label>
            <select id="loc-unit"><option value="metric">°C</option><option value="imperial">°F</option></select>
          </div>
          <div class="col">
            <button id="loc-weather-test" class="btn-dark" style="margin-top:18px">${t('settings_loc.fetch')}</button>
          </div>
        </div>
        <p id="loc-weather-now" class="hint"></p>
        <label>${t('settings_loc.temp_folder')}</label>
        <div class="file-input-group">
          <input type="text" id="loc-temp-folder" readonly>
          <button id="loc-temp-browse">...</button>
        </div>
        <label>${t('settings_loc.hum_folder')}</label>
        <div class="file-input-group">
          <input type="text" id="loc-hum-folder" readonly>
          <button id="loc-hum-browse">...</button>
        </div>
        <p class="hint">${t('settings_loc.weather_hint')}</p>
      </div>
    </div>`;
}
