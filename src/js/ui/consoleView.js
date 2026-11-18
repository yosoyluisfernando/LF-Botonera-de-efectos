/**
 * Archivo: consoleView.js
 * Propósito: pinta las tiras de canal de la consola y manda los movimientos del
 * fader. Solo dibuja (Regla 4): los buses, sus topes y si suman en el programa
 * los decide Rust en `get_console_view`; el nivel y los dB llegan en `meter-tick`.
 * No registra listeners de audio — main.js es el único punto de escucha.
 */
import '../../css/console.css';
import '../../css/consoleMeter.css';
import '../../css/consoleFader.css';
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { wireConsoleFader, paintConsoleFaders } from './consoleFaders.js';
import { registerVolumeView, refreshVolumeControls } from './volumeSync.js';
import { activeMeterTransition } from './meterBallistics.js';

/** Los faders vivos, por bus, para no rehacer el DOM en cada tick. */
const _strips = new Map();
registerVolumeView('console', state => paintConsoleFaders(_strips, state));

export async function renderConsole() {
    const view = await invoke('get_console_view').catch(() => null);
    if (!view) return null;
    _strips.clear();
    document.querySelector('.console-body')?.classList.toggle('has-tab-outputs', view.strips.some(s => s.device));
    _fill('console-strips', view.strips);
    _fill('console-program', [view.program]);
    _fill('console-cue', [view.cue]);
    refreshVolumeControls();
    return view;
}

export function refreshConsoleOutputs() {
    if (_strips.size) renderConsole().catch(console.error);
}

/** Con la consola cerrada no hay nada que pintar: el tick sigue llegando cincuenta
 *  veces por segundo y no tiene sentido tocar un DOM que nadie ve. */
export function clearConsole() {
    _strips.clear();
}

/** Actualiza vúmetros y dB. Llamar desde el handler de `meter-tick`. */
export function updateConsoleTick(payload) {
    if (!_strips.size) return;
    const buses = payload.buses ?? {};
    _paint('efectos', buses.efectos);
    _paint('panel', buses.panel);
    _paint('reproductor', buses.reproductor);
    _paint('cue', buses.cue);
    (buses.salidas ?? []).forEach((level, i) => _paint(`salida${i}`, level));
    // El programa no viaja en `buses`: es master_level_*, donde el vúmetro de la
    // barra inferior lo lee desde siempre.
    _paint('programa', {
        l: payload.master_level_l ?? 0,
        r: payload.master_level_r ?? 0,
        db: _dbOf(payload.master_level_l, payload.master_level_r, payload.idle),
    });
}

function _fill(containerId, strips) {
    const box = document.getElementById(containerId);
    if (!box) return;
    box.innerHTML = '';
    strips.forEach(strip => box.appendChild(_buildStrip(strip)));
}

function _buildStrip(strip) {
    const master = strip.bus === 'programa';
    const el = document.createElement('div');
    el.className = 'console-strip';
    if (master) el.classList.add('is-master');
    // Aviso de que este bus tiene tarjeta propia y no obedece al máster. Solo
    // cuando es una DECISIÓN del usuario: la pre-escucha nunca suma en el
    // programa, y marcarla siempre no informaría de nada — sería ruido fijo.
    else if (strip.bus !== 'cue' && !strip.in_program && !strip.follows_master) el.classList.add('is-direct');
    el.innerHTML = `
      <div class="strip-stage">
        <div class="strip-meters">
          ${_meter('L')}${_meter('R')}
        </div>
        <div class="fader-track">
          <input type="range" class="fader${master ? ' is-master' : ''}"
                 min="0" max="${strip.max}" step="0.01" value="${strip.fader}"
                 aria-label="${t(`console.bus.${strip.bus}`)}">
        </div>
      </div>
      <span class="strip-label">${t(`console.bus.${strip.bus}`)}</span>
      <span class="strip-dest">${t(`console.dest.${strip.bus}`)}</span>
      <span class="strip-db">${_fmtDb(null)}</span>`;
    if (strip.device) {
        el.classList.toggle('is-unavailable', !strip.available);
        const label = t('tab_outputs.channel');
        const deviceName = strip.device_name || strip.device;
        el.querySelector('.strip-label').textContent = label;
        el.querySelector('.fader').setAttribute('aria-label', `${label}: ${deviceName}`);
        const dest = el.querySelector('.strip-dest'); dest.classList.add('strip-device');
        dest.textContent = `${deviceName} · ${t(strip.available ? 'tab_outputs.controlled' : 'tab_outputs.unavailable')}`;
    }
    wireConsoleFader(el.querySelector('.fader'), strip.bus, strip.device);
    _strips.set(strip.bus, el);
    return el;
}

/**
 * El vúmetro se marca `aria-hidden`: cambia cincuenta veces por segundo y no hay
 * forma de leerlo en voz alta que no sea una tortura. Lo que sí es legible es la
 * cifra en dB, que está fuera y se lee al llegar a ella.
 */
function _meter(ch) {
    return `<div class="meter" aria-hidden="true">
              <span class="meter-ch">${ch}</span>
              <div class="meter-bg"><div class="meter-mask" data-ch="${ch}"></div></div>
            </div>`;
}

function _paint(bus, level) {
    const el = _strips.get(bus);
    if (!el || !level) return;
    _mask(el, 'L', level.l ?? 0);
    _mask(el, 'R', level.r ?? 0);
    const db = el.querySelector('.strip-db');
    if (db) db.textContent = _fmtDb(level.db);
}

function _mask(el, ch, level) {
    const mask = el.querySelector(`.meter-mask[data-ch="${ch}"]`);
    // La máscara tapa desde arriba: cuanto más baja, más escala se ve.
    if (mask) {
        mask.style.transitionDuration = `${activeMeterTransition(mask, level)}ms`;
        mask.style.height = `${((1 - Math.min(level, 1)) * 100).toFixed(1)}%`;
    }
}

/**
 * El dB lo calcula Rust y llega en el tick (regla 4). El del programa es la
 * excepción: su nivel viaja como `master_level_*` desde antes de que la consola
 * existiera, así que aquí se hace la misma cuenta en vez de meter otro campo al
 * evento solo para esta tira.
 */
function _dbOf(l, r, idle) {
    const peak = Math.max(l ?? 0, r ?? 0);
    if (idle || peak <= 0) return null;
    return 20 * Math.log10(peak);
}

/** `null` es el silencio: en decibelios el cero no existe. */
function _fmtDb(db) {
    if (db === null || db === undefined) return t('console.silence');
    return `${db.toFixed(1)} dB`;
}
