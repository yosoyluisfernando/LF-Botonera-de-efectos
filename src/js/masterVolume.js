/**
 * Archivo: masterVolume.js
 * Proposito: pinta el control VOL y envia cambios al backend Rust.
 */

import { invoke } from './api.js';
import { t } from './i18n.js';
import { placeMenu } from './menuPosition.js';
import { appConfirm } from './appDialog.js';

let _state = { volume: 1, remember: false, boost: false, max: 1 };
let _wired = false;
let _menu = null;

export function initMasterVolume() {
    refreshMasterVolume();
    if (_wired) return;
    _wired = true;
    _button()?.addEventListener('click', _togglePanel);
    _button()?.addEventListener('contextmenu', _showMenu);
    _slider()?.addEventListener('input', _onSlide);
    document.addEventListener('click', _closeMenuOutside, { capture: true });
}

export function refreshMasterVolume() {
    invoke('get_master_volume_state')
        .then(state => _applyState(state ?? _state))
        .catch(console.error);
}

function _togglePanel(event) {
    event.stopPropagation();
    _panel()?.classList.toggle('hidden');
    _button()?.classList.toggle('active', !_panel()?.classList.contains('hidden'));
}

function _onSlide(event) {
    const volume = Number(event.target.value) / 100;
    invoke('set_master_volume', { volume })
        .then(state => _applyState(state ?? _state))
        .catch(console.error);
}

function _showMenu(event) {
    event.preventDefault();
    event.stopPropagation();
    if (!_menu) {
        _menu = document.createElement('div');
        _menu.className = 'master-volume-menu hidden';
        document.body.appendChild(_menu);
    }
    _menu.innerHTML = '';
    _menu.appendChild(_menuButton(_state.remember, t('playback.remember_volume'), () => {
        _setOptions(!_state.remember, _state.boost);
    }));
    _menu.appendChild(_menuButton(_state.boost, t('playback.allow_boost'), async () => {
        if (!_state.boost && !await appConfirm(t('playback.boost_warning'))) return;
        _setOptions(_state.remember, !_state.boost);
    }));
    _menu.classList.remove('hidden');
    placeMenu(_menu, event.clientX, event.clientY);
}

function _setOptions(remember, boost) {
    invoke('set_master_volume_options', { remember, boost })
        .then(state => _applyState(state ?? _state))
        .catch(console.error);
}

function _menuButton(active, label, onClick) {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.textContent = `${active ? '✓ ' : ''}${label}`;
    btn.addEventListener('click', event => {
        event.stopPropagation();
        _hideMenu();
        onClick();
    });
    return btn;
}

function _applyState(state) {
    _state = {
        volume: Number(state.volume ?? 1),
        remember: !!state.remember,
        boost: !!state.boost,
        max: Number(state.max ?? 1),
    };
    const slider = _slider();
    const percent = Math.round(_state.volume * 100);
    if (slider) {
        slider.max = Math.round(_state.max * 100);
        slider.value = String(percent);
        slider.style.setProperty('--vol-fill', `${(percent / Number(slider.max)) * 100}%`);
    }
    const readout = document.getElementById('master-volume-readout');
    if (readout) readout.textContent = `${percent}%`;
}

function _closeMenuOutside(event) {
    if (!_menu?.contains(event.target)) _hideMenu();
}

function _hideMenu() {
    _menu?.classList.add('hidden');
}

const _button = () => document.getElementById('master-volume-btn');
const _panel = () => document.getElementById('master-volume-panel');
const _slider = () => document.getElementById('master-volume-slider');
