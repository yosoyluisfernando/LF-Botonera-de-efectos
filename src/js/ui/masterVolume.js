/**
 * Archivo: masterVolume.js
 * Proposito: pinta el control VOL y envia cambios al backend Rust.
 */

import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { placeMenu } from '../util/menuPosition.js';
import { registerVolumeView, refreshVolumeControls } from './volumeSync.js';
import { bindVolumeSlider, paintVolumeSlider } from './volumeSlider.js';
import { appConfirm } from './appDialog.js';

let _state = { volume: 1, remember: false, boost: false, max: 1 };
let _wired = false;
let _menu = null;

export function initMasterVolume() {
    refreshMasterVolume();
    if (_wired) return;
    _wired = true;
    registerVolumeView('master', state => _applyState(state.master));
    _button()?.addEventListener('click', _togglePanel);
    _button()?.addEventListener('contextmenu', _showMenu);
    if (_slider()) bindVolumeSlider(_slider(), _onSlide);
    document.addEventListener('click', _closeMenuOutside, { capture: true });
}

export function refreshMasterVolume() {
    return refreshVolumeControls();
}

function _togglePanel(event) {
    event.stopPropagation();
    _panel()?.classList.toggle('hidden');
    const open = !_panel()?.classList.contains('hidden');
    _button()?.classList.toggle('active', open);
    _button()?.setAttribute('aria-expanded', String(open));
}

function _onSlide(value) {
    const volume = Number(value) / 100;
    return invoke('set_master_volume', { volume });
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
        .catch(console.error).finally(refreshVolumeControls);
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
        const max = Math.round(_state.max * 100);
        paintVolumeSlider(slider, percent, max);
        const fill = (percent / max) * 100;
        const safe = (100 / max) * 100;
        slider.style.setProperty('--vol-fill', `${fill}%`);
        slider.style.setProperty('--vol-safe', `${safe}%`);
        slider.style.setProperty('--vol-safe-fill', `${Math.min(fill, safe)}%`);
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
