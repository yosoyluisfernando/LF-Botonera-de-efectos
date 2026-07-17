/**
 * Archivo: consoleView.js
 * Propósito: pinta las tiras de canal de la consola y manda los movimientos del
 * fader. Solo dibuja (Regla 4): los buses, sus topes y si suman en el programa
 * los decide Rust en `get_console_view`; el nivel y los dB llegan en audio-tick.
 * No registra listeners de audio — main.js es el único punto de escucha.
 */
import '../../css/console.css';
import '../../css/consoleMeter.css';
import '../../css/consoleFader.css';
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

/** Los faders vivos, por bus, para no rehacer el DOM en cada tick. */
const _strips = new Map();

export async function renderConsole() {
    const view = await invoke('get_console_view').catch(() => null);
    if (!view) return null;
    _strips.clear();
    _fill('console-strips', view.strips);
    _fill('console-program', [view.program]);
    _fill('console-cue', [view.cue]);
    return view;
}

/** Con la consola cerrada no hay nada que pintar: el tick sigue llegando diez
 *  veces por segundo y no tiene sentido tocar un DOM que nadie ve. */
export function clearConsole() {
    _strips.clear();
}

/** Actualiza vúmetros y dB. Llamar desde el handler de audio-tick en main.js. */
export function updateConsoleTick(payload) {
    if (!_strips.size) return;
    const buses = payload.buses ?? {};
    _paint('efectos', buses.efectos);
    _paint('panel', buses.panel);
    _paint('reproductor', buses.reproductor);
    _paint('cue', buses.cue);
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
    // Un bus con tarjeta propia no obedece al máster; la tira tiene que decirlo.
    else if (!strip.in_program) el.classList.add('is-direct');
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
    _wireFader(el.querySelector('.fader'), strip.bus);
    _strips.set(strip.bus, el);
    return el;
}

/**
 * El vúmetro se marca `aria-hidden`: cambia diez veces por segundo y no hay
 * forma de leerlo en voz alta que no sea una tortura. Lo que sí es legible es la
 * cifra en dB, que está fuera y se lee al llegar a ella.
 */
function _meter(ch) {
    return `<div class="meter" aria-hidden="true">
              <span class="meter-ch">${ch}</span>
              <div class="meter-bg"><div class="meter-mask" data-ch="${ch}"></div></div>
            </div>`;
}

/**
 * Mientras se arrastra se aplica pero NO se guarda: aplicar es un atómico, y
 * guardar en cada píxel sería una tormenta de escrituras a disco. Al soltar, se
 * persiste. Es el mismo cuidado que ya tenían el máster y el reproductor.
 */
function _wireFader(fader, bus) {
    if (!fader) return;
    fader.addEventListener('input', () => _send(bus, fader.value, false));
    fader.addEventListener('change', () => _send(bus, fader.value, true));
}

function _send(bus, value, persist) {
    invoke('set_bus_fader', { bus, value: Number(value), persist }).catch(console.error);
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
    if (mask) mask.style.height = `${((1 - Math.min(level, 1)) * 100).toFixed(1)}%`;
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
