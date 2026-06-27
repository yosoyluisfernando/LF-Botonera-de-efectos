/**
 * Archivo: trackTransport.js
 * Propósito: transporte del editor de pista (Play/Pausa/Stop + cursor‑guía).
 * Responsabilidad separada del orquestador (trackEditor.js). El cursor es un
 * reloj `performance.now()` movido por requestAnimationFrame (traducido del LFA,
 * sin depender de ticks). Solo invoca al motor Rust y pinta el cursor (Regla 4).
 */
import { invoke } from './api.js';

const PLAY_ID = '__track_preview__';

let _wave = null, _meta = null, _path = '';
let _playing = false, _paused = false, _mark = 0, _startClock = 0, _raf = 0;

/** Conecta el transporte con la pista recién abierta/analizada (resetea estado). */
export function bindTransport(wave, meta, path) {
    _wave = wave;
    _meta = meta;
    _path = path;
    _playing = false;
    _paused = false;
    _mark = 0;
}

/** Botón Play: si está en pausa reanuda; si no, (re)inicia desde la marca
 *  (y si ya sonaba, reinicia desde la marca, sin duplicar). */
export function play() {
    const origin = _paused ? (_wave ? _wave.getCursor() : 0) : _mark;
    _playFrom(origin);
}

/** ▶ junto al inicio: reproduce desde el cue de inicio; repetir reinicia ahí. */
export function playInicio() {
    _mark = _meta?.cue_start_s || 0;
    _paused = false;
    _playFrom(_mark);
}

/** Clic en la onda: fija la marca; si algo suena, reinicia desde ahí. */
export function onCursorMark(t) {
    _mark = t;
    _paused = false;
    if (_playing) _playFrom(t);
}

/** Stop cíclico: 1º pausa · 2º vuelve a la marca · 3º vuelve a 0:00. */
export function stop() {
    if (_playing) {
        halt();
        _paused = true;
        return;
    }
    const g = _wave ? _wave.getCursor() : 0;
    if (_mark > 0.001 && Math.abs(g - _mark) > 0.001) {
        _wave?.setCursor(_mark, false);
    } else {
        _mark = 0;
        _wave?.setCursor(0, false);
    }
    _paused = false;
}

/** Detiene audio y bucle, dejando la guía donde esté (uso interno y al cerrar). */
export function halt() {
    invoke('stop_audio', { id: PLAY_ID });
    _playing = false;
    if (_raf) {
        cancelAnimationFrame(_raf);
        _raf = 0;
    }
    document.getElementById('te-play').classList.remove('te-active');
}

async function _playFrom(origin) {
    _wave?.setCursor(origin, false);
    try {
        await invoke('play_audio', {
            id: PLAY_ID, path: _path, volume: 1.0,
            loopMode: false, stopOther: false, overlap: false, restart: true,
            cueStartS: origin, cueEndS: null, gainDb: 0,
        });
        _playing = true;
        _paused = false;
        document.getElementById('te-play').classList.add('te-active');
        _startClock = performance.now() / 1000 - origin;
        if (_raf) cancelAnimationFrame(_raf);
        _loop();
    } catch (e) { console.error('Error al reproducir:', e); }
}

function _loop() {
    if (!_playing) return;
    const t = performance.now() / 1000 - _startClock;
    const dur = _meta?.duration_s || 0;
    if (dur > 0 && t >= dur) {
        halt();
        _paused = false;
        _mark = 0;
        _wave?.setCursor(0, false);
        return;
    }
    _wave?.setCursor(t, true);
    _raf = requestAnimationFrame(_loop);
}
