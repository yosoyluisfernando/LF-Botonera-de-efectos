import { createPlaybackModeController } from './playbackModeController.js';

const controller = createPlaybackModeController({
    getState: 'get_fixed_playback_state', setMode: 'set_fixed_playback_mode',
    setSolo: 'set_fixed_solo_mode', stop: 'stop_fixed_audio',
    element: mode => document.querySelector(`[data-fixed-mode="${mode}"]`),
    soloElement: () => document.querySelector('[data-fixed-solo]'),
    stopElement: () => document.querySelector('[data-fixed-stop]'),
});

export function initFixedPlaybackModes() { return controller.init(); }
export function refreshFixedPlaybackModes() { return controller.refresh(); }
