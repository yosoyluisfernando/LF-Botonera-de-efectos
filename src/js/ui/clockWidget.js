/**
 * Archivo: clockWidget.js
 * Propósito: Gestiona el reloj/contador de la barra inferior.
 * No registra listeners — main.js es el único punto de escucha de clock-tick y audio-tick.
 * Expone updateClockTick() y updateAudioTick() para que main.js las llame al recibir eventos.
 * El menú contextual de formato 24/12 h invoca a Rust; la lógica de formato vive allí (Regla 4).
 */

import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { formatCountdown } from '../util/countdownFormat.js';

let _showingAudio  = false;
let _showCountdown = false;
let _clearTimer    = null;
let _is24h         = true;
let _lastDuration  = 0;

const _clockEl = () => document.getElementById('clock-time');
const _dateEl  = () => document.getElementById('clock-date');

// ─── Menú contextual de formato ──────────────────────────────────────────────

let _menu = null;
let _wired = false;

function _buildMenu() {
    _menu = document.createElement('div');
    _menu.className = 'clock-fmt-menu';
    document.body.appendChild(_menu);
    document.addEventListener('click', () => _hideMenu(), { capture: true });
}

function _showMenu(e) {
    e.preventDefault();
    if (!_menu) _buildMenu();
    _menu.innerHTML = `
        <div class="clock-fmt-item${_is24h  ? ' active' : ''}" data-val="24">${t('clock.fmt_24h')}</div>
        <div class="clock-fmt-item${!_is24h ? ' active' : ''}" data-val="12">${t('clock.fmt_12h')}</div>
    `;
    _menu.style.display = 'block';
    _placeMenu(e.clientX, e.clientY);

    _menu.querySelectorAll('.clock-fmt-item').forEach(el => {
        el.addEventListener('click', async (ev) => {
            ev.stopPropagation();
            const want24 = el.dataset.val === '24';
            if (want24 !== _is24h) await invoke('toggle_clock_format');
            _hideMenu();
        });
    });
}

function _hideMenu() {
    if (_menu) _menu.style.display = 'none';
}

function _placeMenu(x, y) {
    const margin = 4;
    const rect = _menu.getBoundingClientRect();
    const maxX = window.innerWidth - rect.width - margin;
    const maxY = window.innerHeight - rect.height - margin;
    _menu.style.left = `${Math.max(margin, Math.min(x, maxX))}px`;
    _menu.style.top = `${Math.max(margin, Math.min(y, maxY))}px`;
}

// ─── Inicialización DOM ───────────────────────────────────────────────────────

/** Conecta el menú contextual. Llamar una vez al inicio (síncrono, sin IPC). */
export function initClockWidget() {
    if (_wired) return;
    _wired = true;
    document.getElementById('clock-widget')
        ?.addEventListener('contextmenu', _showMenu);
}

// ─── Handlers de eventos (llamados desde main.js) ────────────────────────────

/** Actualiza la hora/fecha. Llamar desde el handler de clock-tick en main.js. */
export function updateClockTick(payload) {
    const { time_str, date_str, clock_24h } = payload ?? {};
    _is24h = clock_24h ?? true;
    if (_dateEl() && date_str) _dateEl().textContent = date_str;
    if (!_showingAudio && !_showCountdown && _clockEl() && time_str) {
        _clockEl().textContent = time_str;
    }
}

/** Actualiza el contador regresivo. Llamar desde el handler de audio-tick en main.js. */
export function updateAudioTick(payload) {
    const rem = (payload ?? {}).display_remaining ?? 0;
    const dur = (payload ?? {}).display_duration ?? 0;
    if (rem > 0.005) {
        _showingAudio = true;
        _showCountdown = false;
        _lastDuration = dur || _lastDuration;
        if (_clearTimer) { clearTimeout(_clearTimer); _clearTimer = null; }
        if (_clockEl()) _clockEl().textContent = formatCountdown(rem, _lastDuration);
    } else if (_showingAudio) {
        _showingAudio = false;
        _showCountdown = true;
        if (_clockEl()) _clockEl().textContent = formatCountdown(0, dur || _lastDuration);
        if (_clearTimer) clearTimeout(_clearTimer);
        _clearTimer = setTimeout(() => {
            _showCountdown = false;
            _clearTimer = null;
            _lastDuration = 0;
        }, 3000);
    }
}
