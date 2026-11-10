/**
 * Archivo: main.js
 * Propósito: Punto de entrada mínimo. El contrato de arranque vive en startup.js.
 */

import { initTouchControls } from './touchControls.js';
import { loadCapabilities } from '../bridge/platform.js';
import { startApp } from './startup.js';

document.addEventListener('DOMContentLoaded', async () => {
    initTouchControls();
    try { await loadCapabilities(); } catch (error) { console.error(error); }
    await startApp();
});
