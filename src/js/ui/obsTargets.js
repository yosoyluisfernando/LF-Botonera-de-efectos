// Listas de presentación: las identidades y los estados siempre proceden de Rust.
import { t } from '../util/i18n.js';

export function initObsTargets(state, saved, onChange) {
    const target = document.getElementById('obs-target');
    const scene = document.getElementById('obs-scene');
    let entries = [];
    let current = null;
    const selected = () => entries[Number(target.value)];
    const fill = () => {
        target.replaceChildren();
        entries = current.target === 'filters'
            ? state.filters.filter(f => f.source === scene.value)
            : current.target === 'inputs'
            ? state.inputs.map(input => ({ name: input.name }))
            : current.target === 'scene_items'
                ? state.scene_items.filter(item => item.scene === scene.value)
                : (state[current.target] || []).map(name => ({ name }));
        const retained = current.target === 'filters'
            ? saved?.target === scene.value && !!saved?.filter
            : current.target === 'scene_items'
            ? saved?.scene === scene.value && saved?.scene_item_id != null
            : saved?.kind === current.kind;
        const savedName = current.target === 'filters' ? saved?.filter : saved?.target;
        if (retained && savedName && !entries.some(e => e.name === savedName
            && (current.target !== 'scene_items' || e.id === saved.scene_item_id))) {
            entries.unshift({ name: savedName, id: saved.scene_item_id });
        }
        for (const [index, entry] of entries.entries()) {
            const option = document.createElement('option');
            option.value = String(index);
            option.textContent = current.kind === 'quick_transition'
                ? t('obs.quick_number').replace('{id}', entry.name.split('.').at(-1))
                : current.target === 'scene_items' ? `${entry.name} (#${entry.id})` : entry.name;
            target.append(option);
        }
        if (retained) {
            const index = entries.findIndex(e => e.name === savedName
                && (current.target !== 'scene_items' || e.id === saved.scene_item_id));
            if (index >= 0) target.value = String(index);
        }
        onChange();
    };
    scene.onchange = fill;
    target.onchange = onChange;
    return {
        render(item) {
            current = item;
            document.getElementById('obs-target-row').classList.toggle('hidden', !item.target);
            const filter = item.target === 'filters';
            document.getElementById('obs-scene-row').classList.toggle('hidden', item.target !== 'scene_items' && !filter);
            document.querySelector('label[for="obs-scene"]').textContent = t(filter ? 'obs.source' : 'obs.scene_group');
            document.querySelector('label[for="obs-target"]').textContent = t(item.target === 'inputs'
                ? 'obs.input' : filter ? 'obs.filter' : item.target === 'scene_items' ? 'obs.element' : 'obs.target');
            scene.replaceChildren();
            const scenes = [...new Set(filter ? state.filters.map(f => f.source)
                : [...(state.scenes || []), ...(state.scene_items || []).map(i => i.scene)])];
            const savedSource = filter ? saved?.filter && saved.target : saved?.scene;
            if (savedSource && !scenes.includes(savedSource)) scenes.unshift(savedSource);
            for (const value of scenes) {
                const option = document.createElement('option');
                option.value = value; option.textContent = value; scene.append(option);
            }
            if (savedSource) scene.value = savedSource;
            const hint = document.getElementById('obs-resource-hint');
            const key = filter ? 'filter_hint' : item.target === 'inputs' ? 'input_hint' : item.target === 'scene_items'
                ? 'element_hint' : item.kind.startsWith('replay_') ? 'replay_hint'
                : item.kind === 'record_split' ? 'split_hint'
                : ['record_pause', 'record_resume', 'record_pause_toggle'].includes(item.kind) ? 'pause_hint' : '';
            hint.classList.toggle('hidden', !key);
            hint.textContent = key ? t('obs.' + key) : '';
            fill();
        },
        label() {
            if (current?.target === 'filters') return selected() ? `${scene.value} · ${selected().name}` : '';
            return current?.kind === 'quick_transition' ? target.selectedOptions[0]?.textContent || ''
                : selected()?.name || '';
        },
        draft(item) {
            return { target: item.target === 'filters' ? scene.value : item.target ? selected()?.name || '' : '',
                filter: item.target === 'filters' ? selected()?.name || '' : '',
                scene: item.target === 'scene_items' ? scene.value : '',
                scene_item_id: item.target === 'scene_items' ? selected()?.id ?? null : null };
        },
    };
}
