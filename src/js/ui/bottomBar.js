/**
 * Archivo: bottomBar.js
 * Propósito: Inicialización DOM de la barra inferior.
 * No registra listeners de eventos Rust — eso lo hace main.js (único punto de escucha).
 */

import { initClockWidget }   from './clockWidget.js';
import { initMasterVolume, refreshMasterVolume } from './masterVolume.js';
import { initPlaybackModes, refreshPlaybackModes } from './playbackModes.js';
import { initPlaybackProgressBar, refreshPlaybackProgressBar } from './playbackProgressBar.js';

export function initBottomBar() {
    initPlaybackProgressBar();
    initClockWidget();
    initPlaybackModes();
    initMasterVolume();
}

export function refreshBottomBar() {
    refreshPlaybackModes();
    refreshMasterVolume();
    refreshPlaybackProgressBar();
}
