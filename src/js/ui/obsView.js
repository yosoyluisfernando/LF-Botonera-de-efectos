// Los indicadores de cada botón llegan resueltos desde Rust.
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { paintObsSettings } from './settingsObs.js';
import { applyObsAppearance } from './obsAppearance.js';
let latest = null;
export function paintObsState(state) {
    latest = state;
    applyObsAppearance(state.button_view);
    paintObsSettings(state);
    for (const button of state.buttons ?? []) {
        const cell = [...document.querySelectorAll('#main-grid [data-obs="true"]')]
            .find(el => el.dataset.id === button.id);
        if (!cell) continue;
        cell.dataset.obsState = button.state;
        const badge = cell.querySelector('.timer');
        if (badge) badge.textContent = t('obs.button_state.' + button.state);
        cell.title = button.error ? t('errors.' + button.error) : t('obs.button_state.' + button.state);
    }
}
export async function refreshObsState() {
    if (latest) paintObsState(latest);
    paintObsState(await invoke('obs_get'));
}
