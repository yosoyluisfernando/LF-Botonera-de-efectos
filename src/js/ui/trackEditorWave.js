/**
 * Construye la onda del editor con sus referencias DOM estables.
 * La interacción concreta se recibe como hooks del orquestador.
 */
import { isAndroid } from '../bridge/platform.js';
import { createWaveform } from './waveformCanvas.js';

export function createEditorWave(hooks) {
    return createWaveform({
        container: document.getElementById('te-wave-container'),
        inner: document.getElementById('te-wave-inner'),
        canvas: document.getElementById('te-canvas'),
        cursor: document.getElementById('te-cursor'),
        timeText: document.getElementById('te-time-text'),
    }, hooks, { touch: isAndroid() });
}

export function wireEditorSteps() {
    document.querySelectorAll('[data-te-step]').forEach(button => {
        button.addEventListener('click', () => {
            const slider = document.getElementById(button.dataset.teStep);
            slider.stepUp(Number(button.dataset.step));
            slider.dispatchEvent(new Event('input'));
        });
    });
}
