/**
 * Archivo: settingsLocutionsTemplate.js
 * Propósito: Plantilla traducida del panel Hora y Clima.
 */

import { t } from './i18n.js';

/** Devuelve el HTML del panel de locuciones con textos i18n. */
export function locutionsTemplate() {
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
