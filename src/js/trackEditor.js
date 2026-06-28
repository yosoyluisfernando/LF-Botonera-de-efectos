import { invoke } from './api.js';
import { t } from './i18n.js';
import { createWaveform } from './waveformCanvas.js';
import { bindTransport, play, playInicio, stop, halt, onCursorMark, refreshPreviewGain } from './trackTransport.js';
import { dockIn, openPreferred, popOut, syncButton } from './trackEditorWindow.js';
import * as normConfig from './normConfig.js';

const BUCKETS = 4000;

let _wave = null, _path = '', _name = '', _meta = null, _onSaved = null, _wired = false;

export async function openTrackEditor(path, name, onSaved, options = {}) {
    if (!path) return;
    _path = path; _name = name || ''; _onSaved = onSaved;
    _wireOnce();
    const initialZoom = _zoomValue(options.zoom);
    document.getElementById('te-zoom').value = initialZoom;
    document.getElementById('te-name').textContent = _name ? `${t('track_editor.title_separator')}${_name}` : '';
    document.getElementById('track-editor-modal').classList.remove('hidden');
    _setStatus(t('track_editor.loading'));
    try {
        const r = await invoke('analyze_track', { path, buckets: BUCKETS });
        _meta = r.meta;
        _meta.norm_enabled = true;
        _wave = _wave || _makeWave();
        _wave.setZoom(initialZoom);
        _sanitizeCue(r.duration_s);
        bindTransport(_wave, _meta, _path);
        _wave.setData({ duration: r.duration_s, peaks: r.waveform });
        _wave.setMarkers(_meta.cue_start_s, _meta.cue_end_s);
        _wave.setGain(_gainLinear());
        _fillControls(r);
        _applyDetectedCue(r);
        syncButton();
        _setStatus(null);
        const cfg = await invoke('get_config');
        if (!cfg.norm_prompted) normConfig.open(cfg.norm || {}, cfg.cue_detect || {}, { firstTime: true });
    } catch (e) {
        console.error('Error al analizar pista:', e);
        _setStatus(t('track_editor.error'));
    }
}

export async function openPreferredTrackEditor(path, name, onSaved) {
    return openPreferred(path, name, onSaved, openTrackEditor);
}

function _makeWave() {
    const els = {
        container: document.getElementById('te-wave-container'),
        inner: document.getElementById('te-wave-inner'),
        canvas: document.getElementById('te-canvas'),
        cursor: document.getElementById('te-cursor'),
        timeText: document.getElementById('te-time-text'),
    };
    return createWaveform(els, {
        onCursorChange: onCursorMark,
        onMarkerChange: (s, e) => { _meta.cue_start_s = s; _meta.cue_end_s = e; _updateCueReadout(); },
        onZoom: dir => _stepZoom(dir),
    });
}

function _gainLinear() { return Math.pow(10, _effectiveGainDb() / 20); }

function _normalGainDb() { return _meta.norm_gain_db || 0; }

function _effectiveGainDb() { return _normalGainDb() + (_meta.gain_db || 0); }

function _sanitizeCue(duration) {
    _meta.cue_start_s = Math.min(Math.max(0, _meta.cue_start_s || 0), Math.max(0, duration - 0.01));
    if (_meta.cue_end_s != null && (_meta.cue_end_s <= _meta.cue_start_s || _meta.cue_end_s > duration)) {
        _meta.cue_end_s = null;
    }
}

function _fillControls(r) {
    document.getElementById('te-norm-enabled').checked = !!_meta.norm_enabled;
    document.getElementById('te-lufs').textContent = r.lufs != null ? `${r.lufs.toFixed(1)} LUFS` : '—';
    document.getElementById('te-peak').textContent = r.peak_db != null ? `${r.peak_db.toFixed(1)} dBFS` : '—';
    _syncGainSliderToMeta();
    _updateGainReadout();
    _updateCueReadout();
}

function _applyDetectedCue(r) {
    const hasDetected = r.detected_start_s != null || r.detected_end_s != null;
    const isDefault = (_meta.cue_start_s || 0) < 0.01 && _meta.cue_end_s == null;
    if (!hasDetected || !isDefault) return;
    if (r.detected_start_s != null) _meta.cue_start_s = r.detected_start_s;
    if (r.detected_end_s != null) _meta.cue_end_s = r.detected_end_s;
    _wave?.setMarkers(_meta.cue_start_s, _meta.cue_end_s);
    _updateCueReadout();
}

function _updateGainReadout() {
    document.getElementById('te-gain-readout').textContent =
        `${parseFloat(document.getElementById('te-gain').value).toFixed(1)} dB`;
}

function _syncGainSliderToMeta() { document.getElementById('te-gain').value = _effectiveGainDb(); }

function _updateCueReadout() {
    const dur = _meta.cue_end_s ?? _meta.duration_s;
    document.getElementById('te-cue-start').textContent = `${(_meta.cue_start_s || 0).toFixed(2)}s`;
    document.getElementById('te-cue-end').textContent = _meta.cue_end_s != null ? `${_meta.cue_end_s.toFixed(2)}s` : '—';
    document.getElementById('te-duration').textContent = `${Math.max(0, dur - (_meta.cue_start_s || 0)).toFixed(2)}s`;
}

function _applyGainToWave() {
    _meta.gain_db = parseFloat(document.getElementById('te-gain').value) - _normalGainDb();
    _updateGainReadout();
    _wave?.setGain(_gainLinear());
    refreshPreviewGain();
}

function _stepZoom(dir) {
    const el = document.getElementById('te-zoom');
    el.value = Math.max(1, Math.min(30, parseFloat(el.value) + dir));
    _wave?.setZoom(parseFloat(el.value));
}

function _zoomValue(value) {
    const n = parseFloat(value);
    return Number.isFinite(n) ? Math.max(1, Math.min(30, n)) : 1;
}

function _wireOnce() {
    if (_wired) return;
    _wired = true;
    const on = (id, ev, fn) => document.getElementById(id).addEventListener(ev, fn);
    on('te-gain', 'input', _applyGainToWave);
    on('te-normalize', 'click', async () => {
        try {
            const g = await invoke('recalculate_norm', { path: _path });
            _meta.norm_gain_db = g;
            _meta.gain_db = 0;
            _syncGainSliderToMeta();
            _updateGainReadout();
            _wave?.setGain(_gainLinear());
            refreshPreviewGain();
        } catch (e) { console.error('recalculate_norm:', e); }
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
    on('te-popout', 'click', _toggleWindowMode);
    on('te-norm-settings', 'click', async () => {
        const c = await invoke('get_config');
        normConfig.open(c.norm || {}, c.cue_detect || {});
    });
}

function _toggleWindowMode() {
    const zoom = _zoomValue(document.getElementById('te-zoom').value);
    const closeModal = () => {
        halt();
        document.getElementById('track-editor-modal').classList.add('hidden');
    };
    if (document.body.classList.contains('editor-window-mode')) dockIn(_path, _name, halt, zoom);
    else popOut(_path, _name, closeModal, zoom);
}

async function _save() {
    try {
        await invoke('set_track_cue', { path: _path, startS: _meta.cue_start_s || 0, endS: _meta.cue_end_s });
        await invoke('set_track_gain', { path: _path, gainDb: _meta.gain_db || 0 });
        await invoke('set_track_normalization', { path: _path, enabled: true });
        await _close();
        _onSaved?.();
    } catch (e) { console.error('Error al guardar pista:', e); }
}

async function _close() {
    halt();
    if (document.body.classList.contains('editor-window-mode')) {
        await invoke('set_editor_mode', { mode: 'window' }).catch(console.error);
        window.__TAURI__?.window?.getCurrentWindow?.().close();
    } else {
        await invoke('set_editor_mode', { mode: 'modal' }).catch(console.error);
        document.getElementById('track-editor-modal').classList.add('hidden');
    }
}

function _setStatus(text) {
    const el = document.getElementById('te-status');
    if (!text) { el.classList.add('hidden'); return; }
    el.textContent = text;
    el.classList.remove('hidden');
}
