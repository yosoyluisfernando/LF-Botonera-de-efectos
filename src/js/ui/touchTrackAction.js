/** Acción explícita de cola para no depender de doble toque. */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { alertIpcError } from './ipcError.js';
export function addTouchTrackAction(row, index) {
    if (!matchMedia('(any-pointer: coarse)').matches) return;
    const button = document.createElement('button');
    button.type = 'button';
    button.textContent = '▶';
    button.setAttribute('aria-label', t('mobile.activate_track'));
    button.addEventListener('click', event => {
        event.stopPropagation();
        invoke('player_activate_index', { index }).catch(alertIpcError);
    });
    row.appendChild(button);
}

export function initTouchAudioImport(onUpdate) {
    const button = document.getElementById('player-add-audio');
    button.addEventListener('click', async () => {
        button.disabled = true;
        try {
            const path = await invoke('pick_audio_file');
            if (path) { await invoke('player_add_track', { path }); await onUpdate(); }
        } catch (error) { await alertIpcError(error); }
        finally { button.disabled = false; }
    });
}
