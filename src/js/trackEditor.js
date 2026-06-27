/**
 * Archivo: trackEditor.js
 * Propósito: Editor de pista (modal grande). Orquesta el análisis, el cue
 * Inicio/Fin, el volumen en dB (que gobierna la amplitud de la onda) y la
 * normalización. El transporte (Play/Pausa/Stop + cursor) vive en
 * trackTransport.js; el dibujo en waveformCanvas.js. No hace DSP ni SQL: invoca
 * al motor Rust (Regla 4); los textos en es.json (Regla 6).
 */
import { invoke } from './api.js';
import { t } from './i18n.js';
import { createWaveform } from './waveformCanvas.js';
import { bindTransport, play, playInicio, stop, halt, onCursorMark } from './trackTransport.js';

const BUCKETS = 4000;

let _wave = null, _path = '', _meta = null, _onSaved = null, _wired = false;

/** Abre el editor para un archivo de audio. */
export async function openTrackEditor(path, name, onSaved) {
    if (!path) return;
    _path = path; _onSaved = onSaved;
    _wireOnce();
    document.getElementById('te-name').textContent = name || '';
    document.getElementById('track-editor-modal').classList.remove('hidden');
    _setStatus(t('track_editor.loading'));
    try {
        const r = await invoke('analyze_track', { path, buckets: BUCKETS });
        _meta = r.meta;
        _wave = _wave || _makeWave();
        _sanitizeCue(r.duration_s);
        bindTransport(_wave, _meta, _path);
        _wave.setData({ duration: r.duration_s, peaks: r.waveform });
        _wave.setMarkers(_meta.cue_start_s, _meta.cue_end_s);
        _wave.setGain(_gainLinear());
        _fillControls(r);
        _setStatus(null);
    } catch (e) {
        console.error('Error al analizar pista:', e);
        _setStatus(t('track_editor.error'));
    }
}

function _makeWave() {
    return createWaveform({
        container: document.getElementById('te-wave-container'),
        inner: document.getElementById('te-wave-inner'),
        canvas: document.getElementById('te-canvas'),
        cursor: document.getElementById('te-cursor'),
        timeText: document.getElementById('te-time-text'),
    }, {
        onCursorChange: onCursorMark,
        onMarkerChange: (s, e) => { _meta.cue_start_s = s; _meta.cue_end_s = e; _updateCueReadout(); },
        onZoom: dir => _stepZoom(dir),
    });
}

function _gainLinear() {
    const norm = _meta.norm_enabled ? (_meta.norm_gain_db || 0) : 0;
    return Math.pow(10, (norm + (_meta.gain_db || 0)) / 20);
}

function _sanitizeCue(duration) {
    _meta.cue_start_s = Math.min(Math.max(0, _meta.cue_start_s || 0), Math.max(0, duration - 0.01));
    if (_meta.cue_end_s != null && (_meta.cue_end_s <= _meta.cue_start_s || _meta.cue_end_s > duration)) {
        _meta.cue_end_s = null;
    }
}

function _fillControls(r) {
    document.getElementById('te-gain').value = _meta.gain_db ?? 0;
    document.getElementById('te-norm-enabled').checked = !!_meta.norm_enabled;
    document.getElementById('te-lufs').textContent = r.lufs != null ? `${r.lufs.toFixed(1)} LUFS` : '—';
    document.getElementById('te-peak').textContent = r.peak_db != null ? `${r.peak_db.toFixed(1)} dBFS` : '—';
    _updateGainReadout();
    _updateCueReadout();
}

function _updateGainReadout() {
    document.getElementById('te-gain-readout').textContent =
        `${parseFloat(document.getElementById('te-gain').value).toFixed(1)} dB`;
}

function _updateCueReadout() {
    const dur = _meta.cue_end_s ?? _meta.duration_s;
    document.getElementById('te-cue-start').textContent = `${(_meta.cue_start_s || 0).toFixed(2)}s`;
    document.getElementById('te-cue-end').textContent = _meta.cue_end_s != null ? `${_meta.cue_end_s.toFixed(2)}s` : '—';
    document.getElementById('te-duration').textContent = `${Math.max(0, dur - (_meta.cue_start_s || 0)).toFixed(2)}s`;
}

function _applyGainToWave() {
    _meta.gain_db = parseFloat(document.getElementById('te-gain').value);
    _updateGainReadout();
    _wave?.setGain(_gainLinear());
}

function _stepZoom(dir) {
    const el = document.getElementById('te-zoom');
    el.value = Math.max(1, Math.min(30, parseFloat(el.value) + dir));
    _wave?.setZoom(parseFloat(el.value));
}

function _wireOnce() {
    if (_wired) return;
    _wired = true;
    const on = (id, ev, fn) => document.getElementById(id).addEventListener(ev, fn);
    on('te-gain', 'input', _applyGainToWave);
    on('te-norm-enabled', 'change', e => { _meta.norm_enabled = e.target.checked; _wave?.setGain(_gainLinear()); });
    on('te-normalize', 'click', () => {
        document.getElementById('te-norm-enabled').checked = true;
        _meta.norm_enabled = true;
        _wave?.setGain(_gainLinear());
    });
    on('te-zoom', 'input', e => _wave?.setZoom(parseFloat(e.target.value)));
    on('te-fijar-inicio', 'click', () => { _wave?.fijar('start'); _updateCueReadout(); });
    on('te-fijar-fin', 'click', () => { _wave?.fijar('end'); _updateCueReadout(); });
    on('te-clear-end', 'click', () => { _wave?.clearEnd(); _updateCueReadout(); });
    on('te-play', 'click', play);
    on('te-play-inicio', 'click', playInicio);
    on('te-stop', 'click', stop);
    on('te-save', 'click', _save);
    on('te-close', 'click', _close);
    on('te-close-x', 'click', _close);
    on('te-popout', 'click', _popOut);
}

/** Saca el editor a su propia ventana (carga index.html en modo editor) y
 *  cierra el modal. La ventana reutiliza los mismos módulos. */
function _popOut() {
    const name = document.getElementById('te-name').textContent || '';
    const url = `index.html?editor=${encodeURIComponent(_path)}&name=${encodeURIComponent(name)}`;
    try {
        new window.__TAURI__.webviewWindow.WebviewWindow('track-editor', {
            url, title: 'Editor de pista', width: 1100, height: 720, resizable: true,
        });
        halt();
        document.getElementById('track-editor-modal').classList.add('hidden');
    } catch (e) {
        console.error('Error al abrir el editor en ventana:', e);
    }
}

async function _save() {
    try {
        await invoke('set_track_cue', { path: _path, startS: _meta.cue_start_s || 0, endS: _meta.cue_end_s });
        await invoke('set_track_gain', { path: _path, gainDb: _meta.gain_db || 0 });
        await invoke('set_track_normalization', { path: _path, enabled: !!_meta.norm_enabled });
        _close();
        _onSaved?.();
    } catch (e) { console.error('Error al guardar pista:', e); }
}

function _close() {
    halt();
    // En modo ventana, "Cerrar/Cancelar" cierra la ventana; si no, oculta el modal.
    if (document.body.classList.contains('editor-window-mode')) {
        window.__TAURI__?.window?.getCurrentWindow?.().close();
    } else {
        document.getElementById('track-editor-modal').classList.add('hidden');
    }
}

function _setStatus(text) {
    const el = document.getElementById('te-status');
    if (!text) { el.classList.add('hidden'); return; }
    el.textContent = text;
    el.classList.remove('hidden');
}
