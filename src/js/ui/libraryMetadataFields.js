/** Construye los campos adaptativos y devuelve únicamente los cambios elegidos. */
import { t } from '../util/i18n.js';

const MUSIC = [
    ['title', 'track_title', false], ['artist', 'artist', true],
    ['album', 'album', true], ['album_artist', 'album_artist', true],
    ['genre', 'genre', true],
    ['year', 'year', true, 'number'], ['track_number', 'track_number', false, 'number'],
    ['composer', 'composer', true], ['comment', 'comment', true, 'textarea'],
];
const EFFECTS = [
    ['display_name', 'display_name', false],
    ['category', 'category', true],
    ['description', 'description', true, 'textarea'],
];

export function renderMetadataFields(host, items) {
    host.replaceChildren();
    const collections = new Set(items.map(item => item.collection));
    if (collections.size !== 1) {
        appendNotice(host, t('metadata_editor.mixed_help'));
        return { fields: () => ({}), focus: () => null, mixed: true };
    }
    const multiple = items.length > 1;
    appendFileInfo(host, items, multiple);
    const descriptors = items[0].collection === 'music' ? MUSIC : EFFECTS;
    const controls = descriptors
        .filter(([, , batch]) => !multiple || batch)
        .map(descriptor => appendField(host, descriptor, items, multiple));
    if (multiple) appendNotice(host, t('metadata_editor.batch_help'));
    return {
        fields: () => collect(controls, multiple),
        focus: () => controls[0]?.input.focus(),
        mixed: false,
    };
}

function appendFileInfo(host, items, multiple) {
    const box = document.createElement('div');
    box.className = 'lm-file-info';
    const label = document.createElement('span');
    label.textContent = multiple
        ? t('metadata_editor.selected_count').replace('{count}', items.length)
        : t('metadata_editor.filename');
    const value = document.createElement('strong');
    value.id = 'lm-current-file';
    value.textContent = multiple ? t('metadata_editor.multiple_files') : items[0].file_name;
    box.append(label, value);
    host.appendChild(box);
}

function appendField(host, descriptor, items, multiple) {
    const [name, labelKey, , type = 'text'] = descriptor;
    const wrapper = document.createElement('div');
    wrapper.className = `lm-field${type === 'textarea' ? ' wide' : ''}`;
    const id = `lm-field-${name}`;
    const label = document.createElement('label');
    const different = !same(items.map(item => item[name] ?? ''));
    const initial = different ? '' : items[0][name] ?? '';
    let apply = null;
    if (multiple) {
        apply = document.createElement('input');
        apply.type = 'checkbox';
        apply.id = `lm-apply-${name}`;
        label.htmlFor = apply.id;
        apply.setAttribute('aria-label',
            t('metadata_editor.apply_field').replace('{field}',
                t(`metadata_editor.${labelKey}`)));
        label.append(apply, document.createTextNode(t(`metadata_editor.${labelKey}`)));
    } else {
        label.htmlFor = id;
        label.textContent = t(`metadata_editor.${labelKey}`);
    }
    const input = type === 'textarea'
        ? document.createElement('textarea') : document.createElement('input');
    input.id = id;
    if (multiple) input.setAttribute('aria-label', t(`metadata_editor.${labelKey}`));
    if (type === 'number') {
        input.type = 'number';
        input.min = name === 'year' ? '0' : '1';
        input.max = '9999';
        input.step = '1';
    } else {
        if (type !== 'textarea') input.type = 'text';
        input.maxLength = 500;
    }
    input.value = initial;
    input.dataset.initial = String(initial);
    if (different) input.placeholder = t('metadata_editor.multiple_values');
    if (multiple) {
        input.disabled = true;
        input.dataset.lmDisabled = 'true';
        apply.addEventListener('change', () => {
            input.disabled = !apply.checked;
            input.dataset.lmDisabled = String(!apply.checked);
            if (apply.checked) input.focus();
        });
    }
    wrapper.append(label, input);
    host.appendChild(wrapper);
    return { name, input, apply, numeric: type === 'number' };
}

function collect(controls, multiple) {
    const fields = {};
    controls.forEach(({ name, input, apply, numeric }) => {
        if (multiple && !apply.checked) return;
        const raw = input.value.trim();
        if (!multiple && raw === input.dataset.initial) return;
        fields[name] = numeric ? (raw === '' ? '' : Number(raw)) : raw;
    });
    return fields;
}

function same(values) {
    return values.every(value => String(value) === String(values[0]));
}

function appendNotice(host, text) {
    const notice = document.createElement('p');
    notice.className = 'lm-help';
    notice.textContent = text;
    host.appendChild(notice);
}
