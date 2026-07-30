/** Editor de tags en chips con deduplicación y sugerencias del catálogo. */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

export function createMetadataTags(host, items, initialSuggestions = []) {
    const list = host.querySelector('.lm-tags-list');
    let input = host.querySelector('.lm-tag-input');
    const freshInput = input.cloneNode(true);
    input.replaceWith(freshInput);
    input = freshInput;
    input.value = '';
    const datalist = host.querySelector('.lm-tag-suggestions');
    const base = commonTags(items);
    let current = [...base];
    let timer = null;
    paintSuggestions(datalist, initialSuggestions);
    paint();
    input.addEventListener('keydown', event => {
        if (event.key !== 'Enter' && event.key !== ',') return;
        event.preventDefault();
        commitInput();
    });
    input.addEventListener('input', () => {
        clearTimeout(timer);
        timer = setTimeout(() => suggest(input.value, datalist), 140);
    });

    function paint() {
        list.replaceChildren();
        current.forEach(tag => {
            const chip = document.createElement('span');
            chip.className = 'lm-tag-chip';
            chip.setAttribute('role', 'listitem');
            chip.textContent = tag;
            const remove = document.createElement('button');
            remove.type = 'button';
            remove.textContent = '×';
            remove.setAttribute('aria-label',
                t('metadata_editor.remove_tag').replace('{tag}', tag));
            remove.addEventListener('click', () => {
                current = current.filter(value => key(value) !== key(tag));
                paint();
                input.focus();
            });
            chip.appendChild(remove);
            list.appendChild(chip);
        });
    }

    function commitInput() {
        input.value.split(',').map(value => value.trim()).filter(Boolean)
            .forEach(tag => {
                if (!current.some(value => key(value) === key(tag))) current.push(tag);
            });
        input.value = '';
        paint();
    }

    return {
        changes() {
            commitInput();
            const baseKeys = new Set(base.map(key));
            const currentKeys = new Set(current.map(key));
            return {
                addTags: current.filter(tag => !baseKeys.has(key(tag))),
                removeTags: base.filter(tag => !currentKeys.has(key(tag))),
            };
        },
        focus: () => input.focus(),
    };
}

function commonTags(items) {
    if (!items.length) return [];
    return (items[0].tags ?? []).filter(tag =>
        items.every(item => (item.tags ?? []).some(value => key(value) === key(tag))));
}

async function suggest(query, datalist) {
    try {
        const values = await invoke('library_metadata_suggest_tags', {
            query: query.trim(), limit: 12,
        });
        paintSuggestions(datalist, values ?? []);
    } catch (error) {
        console.debug('library_metadata_suggest_tags:', error);
    }
}

function paintSuggestions(datalist, values) {
    datalist.replaceChildren();
    values.forEach(value => {
        const option = document.createElement('option');
        option.value = value;
        datalist.appendChild(option);
    });
}

function key(value) {
    return value.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLocaleLowerCase();
}
