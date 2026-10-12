import { t } from '../util/i18n.js';

export function micSettingsMarkup() {
    const number = (id, max, min = 0) => `<label for="mic-${id}">${t('mic.' + id)}</label>
        <input id="mic-${id}" type="number" min="${min}" max="${max}" step="1" required>`;
    return `<div class="modal-content" role="dialog" aria-modal="true" aria-labelledby="mic-settings-title">
        <div class="modal-header"><h3 id="mic-settings-title">${t('mic.settings')}</h3>
            <button type="button" class="close-btn" aria-label="${t('mic.close')}">×</button></div>
        <form id="mic-form"><div class="modal-body">
            <label for="mic-mode">${t('mic.mode')}</label>
            <select id="mic-mode">${['hold', 'toggle', 'auto'].map(mode =>
                `<option value="${mode}">${t('mic.' + mode)}</option>`).join('')}</select>
            <fieldset><legend>${t('mic.targets')}</legend>${['main', 'panel', 'player'].map(id =>
                `<label class="checkbox-line"><input id="mic-${id}" type="checkbox">${t('mic.' + id)}</label>`).join('')}</fieldset>
            <div class="mic-fields"><div>${number('level', 100)}</div>
                <div>${number('down_ms', 5000)}</div><div>${number('up_ms', 10000)}</div></div>
            <p class="hint">${t('mic.level_hint')}</p>
            <div id="mic-auto-fields">
                <label for="mic-input">${t('mic.input')}</label><select id="mic-input"></select>
                <div class="mic-fields"><div>${number('threshold_db', 0, -80)}</div>
                    <div>${number('detect_ms', 2000)}</div><div>${number('silence_ms', 10000, 50)}</div></div>
                <p class="hint">${t('mic.auto_hint')}</p>
                <label for="mic-level-meter">${t('mic.input_level')}</label>
                <meter id="mic-level-meter" min="-100" max="0" value="-100"></meter>
            </div>
            <p id="mic-live-status" role="status"></p><p id="mic-settings-error" role="alert"></p>
        </div><div class="modal-footer"><button type="button" id="mic-cancel" class="btn-dark">${t('mic.cancel')}</button>
            <button type="submit" class="btn-blue">${t('mic.save')}</button></div></form></div>`;
}
