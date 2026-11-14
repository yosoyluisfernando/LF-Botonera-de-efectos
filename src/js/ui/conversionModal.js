/** Modal ocultable: ocultar nunca envía una orden de cancelación a Rust. */
import { t } from '../util/i18n.js';
import { isConverting } from './conversionState.js';
import { conversionLabel } from './conversionButtons.js';
let modal, list, launcher, suppressed = false, latestJobs = [];
function init() {
    if (modal) return;
    modal = document.createElement('div'); modal.className = 'modal-overlay hidden';
    modal.id = 'conversion-modal'; modal.setAttribute('role', 'dialog');
    modal.setAttribute('aria-modal', 'true'); modal.setAttribute('aria-labelledby', 'conversion-title');
    const box = document.createElement('div'); box.className = 'modal-content conversion-dialog';
    const head = document.createElement('div'); head.className = 'modal-header';
    const title = document.createElement('span'); title.id = 'conversion-title';
    title.dataset.i18n = 'conversion.title'; title.textContent = t('conversion.title');
    const hide = document.createElement('button'); hide.className = 'btn-dark';
    hide.dataset.i18n = 'conversion.hide'; hide.textContent = t('conversion.hide');
    hide.onclick = () => { modal.classList.add('hidden'); suppressed = true; launcher.focus(); };
    head.append(title, hide);
    const body = document.createElement('div'); body.className = 'modal-body';
    const hint = document.createElement('p'); hint.className = 'hint';
    hint.dataset.i18n = 'conversion.hide_hint'; hint.textContent = t('conversion.hide_hint');
    list = document.createElement('div'); list.className = 'conversion-list';
    body.append(hint, list); box.append(head, body); modal.append(box);
    modal.addEventListener('keydown', e => {
        if (e.key === 'Escape') { e.stopPropagation(); hide.click(); }
        if (e.key === 'Tab') { e.preventDefault(); hide.focus(); }
    });
    launcher = document.createElement('button'); launcher.className = 'btn-dark conversion-launcher hidden';
    launcher.id = 'conversion-launcher'; launcher.dataset.i18n = 'conversion.title';
    launcher.textContent = t('conversion.title'); launcher.onclick = () => openConversionModal();
    document.body.append(modal, launcher);
}
export function openConversionModal() {
    init(); suppressed = false; modal.classList.remove('hidden');
    renderConversionModal(latestJobs); modal.querySelector('button').focus();
}
export function renderConversionModal(jobs, newWork = false) {
    init(); latestJobs = jobs;
    const visible = jobs.filter(job => job.status !== 'native');
    launcher.classList.toggle('hidden', visible.length === 0);
    if (newWork && !suppressed && modal.classList.contains('hidden')) { openConversionModal(); return; }
    if (modal.classList.contains('hidden')) {
        if (!jobs.some(isConverting)) suppressed = false;
        return;
    }
    list.replaceChildren();
    for (const job of visible) {
        const row = document.createElement('div'); row.className = 'conversion-row';
        const name = document.createElement('strong'); name.textContent = job.path.split(/[\\/]/).pop();
        const status = document.createElement('span');
        status.textContent = isConverting(job) ? conversionLabel(job)
            : t(job.status === 'ready' ? 'conversion.ready' : `errors.${job.error || 'audio_convert_failed'}`);
        const progress = document.createElement('progress'); progress.max = 100;
        progress.setAttribute('aria-label', name.textContent);
        if (job.percent != null) progress.value = job.percent;
        if (job.status === 'failed') progress.hidden = true;
        row.append(name, status, progress); list.append(row);
    }
    // Al terminar el lote, la siguiente importación puede volver a presentar su modal.
    if (!jobs.some(isConverting)) suppressed = false;
}
