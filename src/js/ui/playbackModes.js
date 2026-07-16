import { createPlaybackModeController } from './playbackModeController.js';

const controller = createPlaybackModeController({
    getState: 'get_playback_state', setMode: 'set_playback_mode',
    setSolo: 'set_solo_mode', stop: 'stop_all_audio',
    element: mode => document.getElementById(`pb-btn-${mode}`),
    soloElement: () => document.getElementById('pb-btn-stop_others'),
    stopElement: () => document.getElementById('pb-btn-stop-all'),
});

export function getCurrentMode() { return controller.getMode(); }
export function initPlaybackModes() { return controller.init(); }
export function refreshPlaybackModes() { return controller.refresh(); }
