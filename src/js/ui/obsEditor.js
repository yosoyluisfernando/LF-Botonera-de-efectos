// Editor de acciones OBS: comparte identidad visual y controles de entrada existentes.
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { appAlert } from './appDialog.js';
import { invokeShortcutSave } from './shortcutSave.js';
import { attachPalette, refreshColorInputs } from './colorPalette.js';
import { initMidiField, setMidiField, getMidiField } from './midiControls.js';
import { initEditVisual, currentEditVisual } from './visualSelector.js';
import { initObsTargets } from './obsTargets.js';

export function setObsEditor(enabled) {
    for (const id of ['edit-type', 'edit-filepath', 'btn-prelisten', 'btn-edit-track', 'edit-volume-col']) {
        const el = document.getElementById(id);
        if (id === 'edit-filepath') {
            el.parentElement.classList.toggle('hidden', enabled);
            el.parentElement.previousElementSibling.classList.toggle('hidden', enabled);
        } else {
            el.classList.toggle('hidden', enabled);
            if (id === 'edit-type') el.previousElementSibling.classList.toggle('hidden', enabled);
        }
    }
    document.getElementById('obs-editor-fields')?.classList.toggle('hidden', !enabled);
}
export async function openObsEditor(index, data, onSave) {
    const state = await invoke('obs_get');
    const modal = document.getElementById('edit-modal');
    setObsEditor(true);
    let fields = document.getElementById('obs-editor-fields');
    if (!fields) {
        fields = document.createElement('div'); fields.id = 'obs-editor-fields';
        fields.innerHTML = `
          <label for="obs-action">${t('obs.action')}</label><select id="obs-action"></select>
          <div id="obs-scene-row"><label for="obs-scene">${t('obs.scene_group')}</label><select id="obs-scene"></select></div>
          <div id="obs-target-row"><label for="obs-target">${t('obs.target')}</label><select id="obs-target"></select></div>
          <div id="obs-duration-row"><label for="obs-duration">${t('obs.duration')}</label>
          <input id="obs-duration" type="number" min="50" max="20000" placeholder="${t('obs.duration_hint')}"></div>
          <p id="obs-quick-hint" class="hidden">${t('obs.quick_hint')}</p>
          <p id="obs-resource-hint" class="hidden"></p>
          <p id="obs-editor-status"></p>`;
        modal.querySelector('.modal-body').prepend(fields);
    }
    fields.classList.remove('hidden');
    modal.querySelector('.modal-header h3').textContent = t('obs.edit_button');
    document.getElementById('edit-type').innerHTML = '';
    const type = document.createElement('option'); type.value = 'obs'; type.textContent = t('obs.title');
    document.getElementById('edit-type').append(type);
    const name = document.getElementById('edit-name');
    name.value = data?.label || '';
    let automaticName = !name.value || [data?.obs?.target,
        data?.obs?.filter ? `${data.obs.target} · ${data.obs.filter}` : null,
        t('obs.actions.' + data?.obs?.kind), String(index),
        data?.obs?.kind === 'quick_transition' ? t('obs.quick_number').replace('{id}', data.obs.target.split('.').at(-1)) : null,
    ].includes(name.value);
    name.oninput = () => { automaticName = !name.value.trim(); };
    document.getElementById('edit-bg-color').value = data?.color_bg || '#444444';
    document.getElementById('edit-text-color').value = data?.color_text || '#ffffff';
    document.getElementById('edit-shortcut').value = data?.shortcut || '';
    initMidiField('edit-midi', 'edit-midi-capture', 'edit-midi-clear');
    setMidiField('edit-midi', data?.midi); initEditVisual(data?.visual);
    await attachPalette(document.getElementById('edit-bg-color'), document.getElementById('edit-text-color'), 'button');
    await refreshColorInputs();
    const action = document.getElementById('obs-action'); action.replaceChildren();
    for (const item of state.actions.filter(item => item.kind !== 'transition_select' || data?.obs?.kind === item.kind)) {
        const option = document.createElement('option'); option.value = item.kind;
        option.textContent = t('obs.actions.' + item.kind); action.append(option);
    }
    action.value = data?.obs?.kind || state.actions[0].kind;
    const completeName = () => {
        if (automaticName) name.value = targets.label() || t('obs.actions.' + action.value);
    };
    const targets = initObsTargets(state, data?.obs, completeName);
    const render = () => {
        const item = state.actions.find(item => item.kind === action.value);
        document.getElementById('obs-quick-hint').classList.toggle('hidden', action.value !== 'quick_transition');
        document.getElementById('obs-duration-row').classList.toggle('hidden', !item.duration);
        targets.render(item);
    };
    action.onchange = render; render();
    document.getElementById('obs-duration').value = data?.obs?.duration_ms ?? '';
    document.getElementById('obs-editor-status').textContent = t('obs.status.' + (state.status || 'disabled'));
    document.getElementById('btn-save-edit').onclick = async () => {
        const save = document.getElementById('btn-save-edit');
        save.disabled = true;
        const item = state.actions.find(item => item.kind === action.value);
        const duration = document.getElementById('obs-duration').value;
        try {
            await invokeShortcutSave('obs_save_button', { index,
                label: document.getElementById('edit-name').value.trim() || t('obs.actions.' + action.value),
                colorBg: document.getElementById('edit-bg-color').value,
                colorText: document.getElementById('edit-text-color').value,
                action: { kind: action.value, ...targets.draft(item),
                    duration_ms: item.duration && duration ? Number(duration) : null },
                shortcut: document.getElementById('edit-shortcut').value,
                midi: getMidiField('edit-midi'), visual: currentEditVisual(),
            });
            modal.classList.add('hidden'); onSave?.();
        } catch (error) {
            if (!String(error).startsWith('shortcut_') && !String(error).startsWith('midi_')) {
                await appAlert(t('errors.' + error));
            }
        } finally { save.disabled = false; }
    };
    document.getElementById('btn-cancel-edit').onclick = () => modal.classList.add('hidden');
    modal.classList.remove('hidden'); document.getElementById('edit-name').focus();
}
