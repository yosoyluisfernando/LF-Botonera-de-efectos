/**
 * Archivo: audioDeviceRecovery.js
 * Proposito: modal de recuperacion cuando una salida de audio configurada no
 * esta disponible. La decision real se aplica en Rust; aqui solo se muestra UI.
 */
import '../../css/audioDeviceRecovery.css';
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

let _modal = null;

export async function checkAudioDevicesOnStartup() {
    const status = await invoke('get_audio_device_status').catch(() => null);
    if (!status || _statusOk(status)) {
        if (status) await invoke('apply_configured_audio_devices').catch(console.error);
        return;
    }
    await _show(status);
}

async function _show(status) {
    _ensureModal();
    _modal.querySelector('.audio-device-body').textContent = _message(status);
    _fillSelect(status);
    _modal.classList.remove('hidden');
}

function _ensureModal() {
    if (_modal) return;
    _modal = document.createElement('div');
    _modal.className = 'modal-overlay hidden';
    _modal.style.zIndex = '4800';
    _modal.innerHTML = `
      <div class="modal-content modal-sm audio-device-modal">
        <div class="modal-header">
          <h3>${t('audio_device.title')}</h3>
        </div>
        <div class="modal-body">
          <p class="audio-device-body confirm-text"></p>
          <select class="audio-device-select audio-select"></select>
        </div>
        <div class="modal-footer">
          <button class="btn-dark audio-device-retry">${t('audio_device.retry')}</button>
          <button class="btn-dark audio-device-default">${t('audio_device.use_default')}</button>
          <button class="btn-blue audio-device-apply">${t('audio_device.use_selected')}</button>
        </div>
      </div>`;
    _modal.querySelector('.audio-device-retry').addEventListener('click', _retry);
    _modal.querySelector('.audio-device-default').addEventListener('click', _useDefault);
    _modal.querySelector('.audio-device-apply').addEventListener('click', _useSelected);
    document.body.appendChild(_modal);
}

function _message(status) {
    const missing = [];
    if (!status.main_available) missing.push(`${t('audio_device.main')}: ${status.main}`);
    if (!status.pre_available) missing.push(`${t('audio_device.pre')}: ${status.pre}`);
    return `${t('audio_device.body')} ${missing.join(' / ')}`;
}

function _fillSelect(status) {
    const select = _modal.querySelector('.audio-device-select');
    select.innerHTML = '';
    (status.devices || ['default']).forEach(device => {
        const opt = document.createElement('option');
        opt.value = device;
        opt.textContent = device;
        select.appendChild(opt);
    });
}

async function _retry() {
    const status = await invoke('apply_configured_audio_devices');
    if (_statusOk(status)) _modal.classList.add('hidden');
    else await _show(status);
}

async function _useDefault() {
    const status = await invoke('get_audio_device_status');
    if (!status.main_available) await invoke('set_audio_device', { deviceName: 'default' });
    if (!status.pre_available) await invoke('set_pre_device', { deviceName: 'default' });
    _modal.classList.add('hidden');
}

async function _useSelected() {
    const deviceName = _modal.querySelector('.audio-device-select').value || 'default';
    const status = await invoke('get_audio_device_status');
    if (!status.main_available) await invoke('set_audio_device', { deviceName });
    if (!status.pre_available) await invoke('set_pre_device', { deviceName });
    _modal.classList.add('hidden');
}

function _statusOk(status) {
    return status.main_available && status.pre_available;
}
