/** Indicador visual de conversión separado del progreso de reproducción. */
import { t } from '../util/i18n.js';
import { conversionFor, isConverting } from './conversionState.js';
export function conversionLabel(job) {
    if (job?.status === 'queued') return t('conversion.queued');
    return job?.percent == null ? t('conversion.preparing')
        : t('conversion.percent').replace('{percent}', String(job.percent));
}
export function paintConversionButtons() {
    document.querySelectorAll('[data-audio-path]').forEach(cell => {
        const job = conversionFor(cell.dataset.audioPath);
        const active = isConverting(job);
        cell.classList.toggle('converting', active);
        cell.setAttribute('aria-busy', String(active));
        let label = cell.querySelector('.conversion-status');
        if (!active) { label?.remove(); return; }
        if (!label) {
            label = document.createElement('span');
            label.className = 'conversion-status';
            const text = document.createElement('span'); text.className = 'conversion-text';
            const progress = document.createElement('progress'); progress.max = 100;
            label.append(text, progress); cell.append(label);
        }
        label.querySelector('.conversion-text').textContent = conversionLabel(job);
        const progress = label.querySelector('progress');
        progress.setAttribute('aria-label', conversionLabel(job));
        if (job.percent == null) progress.removeAttribute('value');
        else progress.value = job.percent;
    });
}
