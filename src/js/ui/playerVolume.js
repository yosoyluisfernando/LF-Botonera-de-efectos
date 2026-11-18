/**
 * Archivo: playerVolume.js
 * Propósito: el volumen propio del reproductor desde el panel. El panel es
 * estrecho, así que el control se despliega ENCIMA del botón, no a los lados.
 * El recorrido es 0–100 % y comparte su estado con la consola.
 */

import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { registerVolumeView } from './volumeSync.js';
import { bindVolumeSlider, paintVolumeSlider } from './volumeSlider.js';
import { placeMenu } from '../util/menuPosition.js';

let _panel = null;
let _percent = 100;

export function initPlayerVolume() {
    registerVolumeView('player', state => paintPlayerVolume(state.player_volume));
    const button = document.getElementById('player-volume-btn');
    button.addEventListener('click', _togglePanel);
    document.addEventListener('click', e => {
        if (!_panel || _panel.classList.contains('hidden')) return;
        if (!_panel.contains(e.target) && e.target !== button) _close();
    });
}

/** El estado común de Rust actualiza icono, deslizador y porcentaje. */
export function paintPlayerVolume(volume) {
    const next = Math.round(Math.min(Number(volume) || 0, 1) * 100);
    _percent = next;
    _paintButton();
    const slider = document.getElementById('player-volume-slider');
    paintVolumeSlider(slider, _percent, 100);
    const readout = document.getElementById('player-volume-value');
    if (readout) readout.textContent = `${_percent}%`;
}

function _paintButton() {
    const button = document.getElementById('player-volume-btn');
    if (!button) return;
    button.textContent = _percent === 0 ? '🔇' : _percent < 50 ? '🔉' : '🔊';
    button.title = `${t('player.volume')} ${_percent}%`;
}

function _togglePanel(event) {
    event.stopPropagation();
    if (_panel && !_panel.classList.contains('hidden')) return _close();
    if (!_panel) _panel = _build();
    _panel.classList.remove('hidden');
    // ENCIMA del botón: el panel es estrecho y no hay sitio a los lados.
    const r = event.currentTarget.getBoundingClientRect();
    const h = _panel.getBoundingClientRect().height;
    placeMenu(_panel, r.left, r.top - h - 4);
}

function _build() {
    const el = document.createElement('div');
    el.className = 'player-volume-panel hidden';
    el.innerHTML = `<input id="player-volume-slider" type="range" min="0" max="100"
        step="1" value="${_percent}">
        <span id="player-volume-value" class="player-volume-readout">${_percent}%</span>`;
    document.body.appendChild(el);

    const slider = el.querySelector('#player-volume-slider');
    // Mientras se arrastra se aplica pero NO se guarda: aplicar es instantáneo,
    // guardar en cada píxel sería una escritura a disco por movimiento.
    bindVolumeSlider(slider, (value, persist) => _send(Number(value), persist));
    return el;
}

function _send(percent, persist) {
    _percent = percent;
    _paintButton();
    const value = document.getElementById('player-volume-value');
    if (value) value.textContent = `${percent}%`;
    return invoke('player_set_volume', { volume: percent / 100, persist }).catch(console.error);
}

function _close() {
    _panel?.classList.add('hidden');
}
