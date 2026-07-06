/**
 * Modulo: playbackProgressBar.js
 * Proposito: dibujar la barra global de progreso y enviar seek a Rust.
 */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import '../../css/playbackProgress.css';

let _cfg = { enabled: false, seek_step_s: 10 };
let _panel = null;
let _fill = null;
let _time = null;
let _back = null;
let _forward = null;
let _bar = null;
let _last = { duration: 0, remaining: 0 };
let _dragging = false;

export function initPlaybackProgressBar() {
    if (_panel) return;
    const bottom = document.getElementById('bottom-bar');
    if (!bottom) return;
    _panel = document.createElement('div');
    _panel.id = 'playback-progress-panel';
    _panel.className = 'playback-progress-panel hidden';
    _panel.innerHTML = `
      <button id="progress-back" class="progress-seek-btn" type="button">‹</button>
      <button id="progress-forward" class="progress-seek-btn" type="button">›</button>
      <div id="main-progress-bar" class="main-progress-bar" role="slider">
        <div id="main-progress-fill" class="main-progress-fill"></div>
      </div>
      <span id="main-progress-time" class="main-progress-time">00:00 / 00:00</span>`;
    bottom.parentNode.insertBefore(_panel, bottom);
    _back = document.getElementById('progress-back');
    _forward = document.getElementById('progress-forward');
    _bar = document.getElementById('main-progress-bar');
    _fill = document.getElementById('main-progress-fill');
    _time = document.getElementById('main-progress-time');
    _wire();
    refreshPlaybackProgressBar();
}

export async function refreshPlaybackProgressBar() {
    const config = await invoke('get_config').catch(() => null);
    const progress = config?.playback_progress || {};
    _cfg = {
        enabled: !!progress.enabled,
        seek_step_s: progress.seek_step_s || 10,
    };
    _applyVisibility();
    _paint();
}

export function updatePlaybackProgress(payload = {}) {
    _last = {
        duration: payload.display_duration || 0,
        remaining: payload.display_remaining || 0,
    };
    _paint();
}

function _wire() {
    _back.addEventListener('click', () => _seekDelta(-_cfg.seek_step_s));
    _forward.addEventListener('click', () => _seekDelta(_cfg.seek_step_s));
    _bar.addEventListener('pointerdown', e => {
        _dragging = true;
        _bar.setPointerCapture(e.pointerId);
        _seekFromEvent(e);
    });
    _bar.addEventListener('pointermove', e => {
        if (_dragging) _seekFromEvent(e);
    });
    _bar.addEventListener('pointerup', e => {
        _dragging = false;
        _bar.releasePointerCapture(e.pointerId);
    });
}

function _applyVisibility() {
    _panel?.classList.toggle('hidden', !_cfg.enabled);
    _back.title = t('playback_progress.back');
    _forward.title = t('playback_progress.forward');
    _bar.title = t('playback_progress.seek');
}

function _paint() {
    if (!_panel) return;
    const dur = _last.duration;
    const pos = dur > 0 ? Math.max(0, dur - _last.remaining) : 0;
    const pct = dur > 0 ? Math.min(100, (pos / dur) * 100) : 0;
    _fill.style.width = `${pct}%`;
    _time.textContent = `${_fmt(pos)} / ${_fmt(dur)}`;
    const disabled = !_cfg.enabled || dur <= 0;
    _back.disabled = disabled;
    _forward.disabled = disabled;
    _bar.classList.toggle('disabled', disabled);
}

function _seekDelta(delta) {
    if (_last.duration <= 0) return;
    invoke('seek_active_playback', { deltaS: delta, positionS: null }).catch(console.error);
}

function _seekFromEvent(e) {
    if (_last.duration <= 0) return;
    const rect = _bar.getBoundingClientRect();
    const ratio = Math.min(1, Math.max(0, (e.clientX - rect.left) / rect.width));
    invoke('seek_active_playback', {
        deltaS: null,
        positionS: ratio * _last.duration,
    }).catch(console.error);
}

function _fmt(seconds) {
    const total = Math.max(0, Math.floor(seconds));
    const m = String(Math.floor(total / 60)).padStart(2, '0');
    const s = String(total % 60).padStart(2, '0');
    return `${m}:${s}`;
}
