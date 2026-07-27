/**
 * Archivo: runtimeEvents.js
 * Propósito: suscribe los eventos que emite Rust y los reparte a los módulos que
 * pintan. Estado y progreso llegan en "audio-tick"/"player-tick" a 10 Hz; los
 * niveles usan el evento ligero "meter-tick" a 50 FPS.
 */

import { listen } from '../bridge/api.js';
import { updateTabPlayback } from './tabs.js';
import { paintAudioTick } from './gridPlayback.js';
import { updatePlaybackProgress } from './playbackProgressBar.js';
import { updateClockTick, updateAudioTick } from './clockWidget.js';
import { updateVuMeter } from './vuMeter.js';
import { updateWeatherPanel } from './settingsLocutions.js';
import { paintFixedAudioTick } from './fixedPanel.js';
import { paintPlayerTick } from './playerView.js';
import { updateConsoleTick } from './consoleView.js';
import { openModal as openConsoleModal } from './consoleWindow.js';
import { executeLibraryAction } from './libraryActionDispatch.js';

let _wired = false;

/** Cablea los eventos en vivo. `onRefresh` y `onDockEditor` los pone el arranque. */
export async function wireRuntimeEvents({ onRefresh, onDockEditor }) {
    if (_wired) return;
    await Promise.all([
        listen('clock-tick', e => updateClockTick(e.payload ?? {})),
        listen('audio-tick', e => _paintAudio(e.payload ?? {})),
        listen('meter-tick', e => _paintMeters(e.payload ?? {})),
        // El reproductor tiene motor propio y por tanto su propio pulso: no puede
        // colgar de "audio-tick", que no se emite si no hay efectos sonando.
        listen('player-tick', e => paintPlayerTick(e.payload ?? {})),
        listen('weather-updated', e => updateWeatherPanel(e.payload)),
        listen('global-shortcut-refresh', () => onRefresh()),
        listen('track-editor-dock', e => onDockEditor(e.payload ?? {})),
        listen('console-dock', () => openConsoleModal()),
        listen('library-window-action', e => {
            const payload = e.payload ?? {};
            return executeLibraryAction(payload.action, payload.selection ?? []);
        }),
    ]);
    _wired = true;
}

function _paintAudio(payload) {
    paintAudioTick(payload);
    paintFixedAudioTick(payload);
    updatePlaybackProgress(payload);
    updateAudioTick(payload);
    updateTabPlayback(payload);
    window.dispatchEvent(new CustomEvent('lf-audio-tick', { detail: payload }));
}

function _paintMeters(payload) {
    updateVuMeter(payload);
    updateConsoleTick(payload);
}
