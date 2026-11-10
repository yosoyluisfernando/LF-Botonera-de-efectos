// Preferencia visual y muestras sin acciones; la persistencia y validación son de Rust.
import { t } from '../util/i18n.js';

export function initObsAppearance(panel) {
    const section = document.createElement('fieldset');
    section.className = 'obs-appearance';
    const legend = document.createElement('legend');
    legend.textContent = t('obs.button_view');
    const label = document.createElement('label');
    label.htmlFor = 'obs-button-view';
    label.textContent = t('obs.indicator_style');
    const select = document.createElement('select');
    select.id = 'obs-button-view';
    for (const value of ['border', 'bottom', 'right']) {
        const option = document.createElement('option');
        option.value = value;
        option.textContent = t('obs.view_' + value);
        select.append(option);
    }
    const hint = document.createElement('p');
    hint.textContent = t('obs.view_hint');
    const preview = document.createElement('div');
    preview.id = 'obs-appearance-preview';
    preview.className = 'obs-appearance-preview';
    preview.setAttribute('aria-label', t('obs.view_preview'));
    const samples = [
        ['program', 'obs.sample_scene'], ['preview', 'obs.sample_scene'],
        ['studio_on', 'obs.actions.studio_toggle'], ['recording', 'obs.actions.record_toggle'],
        ['streaming', 'obs.actions.stream_toggle'], ['recording_stopped', 'obs.actions.record_toggle'],
    ];
    for (const [state, key] of samples) {
        const cell = document.createElement('div');
        cell.className = 'grid-item';
        cell.dataset.obs = 'true';
        cell.dataset.obsState = state;
        const name = document.createElement('span');
        name.className = 'label'; name.textContent = t(key);
        const status = document.createElement('span');
        status.className = 'timer'; status.textContent = t('obs.button_state.' + state);
        cell.append(name, status);
        preview.append(cell);
    }
    select.onchange = () => { preview.dataset.obsView = select.value; };
    section.append(legend, label, select, hint, preview);
    panel.append(section);
}
export function loadObsAppearance(view) {
    document.getElementById('obs-button-view').value = view || 'border';
    document.getElementById('obs-appearance-preview').dataset.obsView = obsAppearance();
}
export function obsAppearance() {
    return document.getElementById('obs-button-view').value;
}
export function applyObsAppearance(view) {
    const grid = document.getElementById('main-grid');
    if (grid) grid.dataset.obsView = view || 'border';
}
