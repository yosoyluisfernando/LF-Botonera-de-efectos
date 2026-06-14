/**
 * Archivo: bottomBar.js
 * Propósito: Inicialización DOM de la barra inferior.
 * No registra listeners de eventos Rust — eso lo hace main.js (único punto de escucha).
 */

import { initClockWidget }   from './clockWidget.js';
import { initPlaybackModes } from './playbackModes.js';

export function initBottomBar() {
    initClockWidget();
    initPlaybackModes();
}
