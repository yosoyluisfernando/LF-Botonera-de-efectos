/**
 * Archivo: playbackModes.js
 * Proposito: pinta los modos base y el toggle SOLO; Rust persiste la logica.
 */

import { invoke } from '../bridge/api.js';

const BASE_MODES = ['normal', 'loop', 'overlap', 'restart'];

let _currentMode = 'normal';
let _solo = false;
let _wired = false;

export function getCurrentMode() { return _currentMode; }

export function initPlaybackModes() {
    refreshPlaybackModes();
    if (_wired) return;
    _wired = true;

    BASE_MODES.forEach(mode => {
        document.getElementById(`pb-btn-${mode}`)?.addEventListener('click', () => {
            _currentMode = mode;
            _updateUI();
            invoke('set_playback_mode', { mode }).catch(console.error);
        });
    });

    document.getElementById('pb-btn-stop_others')?.addEventListener('click', () => {
        _solo = !_solo;
        _updateUI();
        invoke('set_solo_mode', { enabled: _solo }).catch(console.error);
    });

    document.getElementById('pb-btn-stop-all')?.addEventListener('click', () => {
        invoke('stop_all_audio').catch(console.error);
    });
}

export function refreshPlaybackModes() {
    invoke('get_playback_state')
        .then(state => {
            _currentMode = state?.mode || 'normal';
            _solo = !!state?.solo;
            _updateUI();
        })
        .catch(() => {
            invoke('get_playback_mode')
                .then(mode => { _currentMode = mode || 'normal'; _updateUI(); })
                .catch(console.error);
        });
}

function _updateUI() {
    BASE_MODES.forEach(mode => {
        document.getElementById(`pb-btn-${mode}`)
            ?.classList.toggle('active', mode === _currentMode);
    });
    document.getElementById('pb-btn-stop_others')
        ?.classList.toggle('active', _solo);
}
