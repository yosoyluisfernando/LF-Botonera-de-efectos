/** Presentación de estados de conversión calculados por Rust. */
import { t } from '../util/i18n.js';
import { invoke } from '../bridge/api.js';
import { showNotice } from './notice.js';
let refreshTimer;
export async function paintConversion(payload, onRefresh) {
    const { path, status, error } = payload ?? {};
    const name = String(path || '').split(/[\\/]/).pop();
    if (status === 'preparing') showNotice(`${t('conversion.preparing')} — ${name}`);
    if (status === 'failed') {
        const key = `errors.${error}`;
        showNotice(`${t(key) === key ? t('errors.audio_convert_failed') : t(key)} — ${name}`, 'error', 8000);
    }
    if (status === 'ready') {
        clearTimeout(refreshTimer);
        refreshTimer = setTimeout(async () => {
            try { await invoke('player_resync'); await onRefresh?.(); }
            catch (error) { console.error(error); }
        }, 150);
    }
}
