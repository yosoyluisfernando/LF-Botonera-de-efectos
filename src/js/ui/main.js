/**
 * Archivo: main.js
 * Propósito: Punto de entrada mínimo. El contrato de arranque vive en startup.js.
 */

import { initTouchControls } from './touchControls.js';
import { loadCapabilities } from '../bridge/platform.js';
import { initAndroidBack } from './androidBack.js';
import { startApp } from './startup.js';

document.addEventListener('DOMContentLoaded', async () => {
    performance.mark('lf-startup-dom');
    initTouchControls();
    try { await loadCapabilities(); } catch (error) { console.error(error); }
    performance.mark('lf-startup-capabilities');
    try { await initAndroidBack(); } catch (error) { console.error(error); }
    performance.mark('lf-startup-back');
    await startApp();
});
