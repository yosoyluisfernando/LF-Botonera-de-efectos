/** Solo controles de favoritos; Rust valida nombres, colores y persistencia. */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { appConfirm } from './appDialog.js';
import { alertIpcError } from './ipcError.js';
const el = id => document.getElementById(id);
let callbacks, favorites = [], wired = false;
export async function initColorFavorites(options) {
    callbacks = options;
    if (!wired) {
        wired = true;
        el('edit-favorite-list').addEventListener('change', async () => {
            const chosen = favorites.find(f => f.name === el('edit-favorite-list').value);
            el('edit-favorite-remove').disabled = !chosen;
            if (chosen) { el('edit-favorite-name').value = chosen.name; await callbacks.apply(chosen); }
        });
        el('edit-favorite-add').addEventListener('click', () => save().catch(alertIpcError));
        el('edit-favorite-remove').addEventListener('click', () => remove().catch(alertIpcError));
    }
    el('edit-favorite-name').value = '';
    el('edit-favorite-save').open = false;
    render(await invoke('get_color_favorites'));
}
function render(items, selected = '') {
    favorites = items;
    const select = el('edit-favorite-list'); select.replaceChildren();
    const placeholder = document.createElement('option'); placeholder.value = '';
    placeholder.textContent = t(items.length ? 'colors.favorite_choose' : 'colors.favorites_empty');
    select.append(placeholder);
    for (const favorite of items) {
        const option = document.createElement('option'); option.value = favorite.name;
        option.textContent = favorite.name; select.append(option);
    }
    select.value = selected;
    el('edit-favorite-remove').disabled = !selected;
}
async function save() {
    if (await callbacks.invalid()) return;
    const favorite = { ...callbacks.read(), name: el('edit-favorite-name').value };
    const items = await invoke('save_color_favorite', { favorite });
    render(items, favorite.name.trim());
    el('edit-favorite-save').open = false;
}
async function remove() {
    const name = el('edit-favorite-list').value;
    if (!name || !await appConfirm(t('colors.favorite_remove_confirm').replace('{name}', name))) return;
    render(await invoke('remove_color_favorite', { name }));
    el('edit-favorite-name').value = '';
}
