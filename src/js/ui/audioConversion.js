/** Presentación de estados de conversión calculados por Rust. */
import { t } from '../util/i18n.js';
import { invoke } from '../bridge/api.js';
import { showNotice } from './notice.js';
import { rememberConversion, conversionJobs } from './conversionState.js';
import { paintConversionButtons } from './conversionButtons.js';
import { renderConversionModal } from './conversionModal.js';
let refreshTimer;
export async function loadConversions() {
    for (const job of await invoke('get_audio_conversions')) rememberConversion(job);
    renderConversionModal(conversionJobs(), conversionJobs().some(j => ['preparing', 'converting'].includes(j.status))); paintConversionButtons();
}
export async function paintConversion(payload, onRefresh) {
    if (!rememberConversion(payload)) return;
    const { path, status, error } = payload;
    renderConversionModal(conversionJobs(), ['preparing', 'converting'].includes(status));
    paintConversionButtons();
    if (status === 'failed') {
        const key = `errors.${error}`;
        showNotice(`${t(key) === key ? t('errors.audio_convert_failed') : t(key)} — ${String(path).split(/[\\/]/).pop()}`, 'error', 8000);
    }
    if (status === 'ready') {
        clearTimeout(refreshTimer);
        refreshTimer = setTimeout(async () => {
            try { await invoke('player_resync'); await onRefresh?.(); }
            catch (error) { console.error(error); }
        }, 150);
    }
}
