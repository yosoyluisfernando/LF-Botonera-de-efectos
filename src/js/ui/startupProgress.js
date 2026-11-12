/**
 * Mantiene visible y actualizable el estado real del arranque. Cada cambio
 * cede un cuadro al WebView para que Windows pueda repintar la ventana.
 */

import { t } from '../util/i18n.js';

export async function startupStage(key) {
    performance.mark(`lf-startup-${key}`);
    const target = document.getElementById('startup-stage');
    const translationKey = `startup.${key}`;
    const translated = t(translationKey);
    if (target && translated !== translationKey) target.textContent = translated;
    await nextPaint();
}

export function startupReady() {
    const screen = document.getElementById('loading-screen');
    screen?.classList.add('hidden');
    document.getElementById('app-section')?.classList.remove('hidden');
}

function nextPaint() {
    return new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
}

export function startupError(msg) {
    const screen = document.getElementById('loading-screen');
    screen.innerHTML = `
        <div style="text-align:center;padding:40px">
            <h2 style="color:var(--error-color)">${t('errors.fatal_ipc')}</h2>
            <p style="color:var(--text-secondary);margin-top:10px;font-size:12px">
                ${msg ?? ''}
            </p>
        </div>`;
}
