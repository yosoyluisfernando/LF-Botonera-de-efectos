import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { normalizedVisual } from './buttonVisual.js';
import { createVisualVirtualGrid } from './visualVirtualGrid.js';
import {
    createResult, createSelectorModal, labelForGroup, labelForKind, paintPreview,
} from './visualSelectorView.js';

let current = normalizedVisual(null);
let modal;
let virtual;
let kind = 'emoji';
let availableGroups = [];
let timer;
let requestId = 0;
const CATALOGS = {
    emoji: {
        search: 'visuals_search_emojis',
        groups: 'visuals_emoji_groups',
    },
    basic: {
        search: 'visuals_search_basics',
        groups: 'visuals_basic_groups',
    },
};

export function initEditVisual(visual) {
    current = normalizedVisual({ visual });
    const mode = document.getElementById('edit-visual-mode');
    mode.onchange = () => {
        if (mode.value === 'auto') {
            current = normalizedVisual(null);
        } else {
            current.mode = mode.value;
        }
        paintChoice();
    };
    document.getElementById('edit-visual-open').onclick = openSelector;
    paintChoice();
}

export function currentEditVisual() {
    return { ...current };
}

export function refreshEditVisualPreview() {
    paintChoice();
}

function setChoice(nextKind, value) {
    current.kind = nextKind;
    current.value = value;
    if (current.mode === 'text') current.mode = 'visual_text';
    paintChoice();
}

function paintChoice() {
    const preview = document.getElementById('edit-visual-preview');
    paintPreview(preview, current, document.getElementById('edit-type')?.value);
    document.getElementById('edit-visual-mode').value =
        current.kind === 'auto' && current.mode === 'visual_text' ? 'auto' : current.mode;
}

function openSelector() {
    ensureModal();
    kind = CATALOGS[current.kind] ? current.kind : 'emoji';
    modal.querySelector('.visual-search').value = '';
    modal.classList.remove('hidden');
    paintTabs();
    loadFresh().catch(console.error);
    modal.querySelector('.visual-search').focus();
}

function ensureModal() {
    if (modal) return;
    modal = createSelectorModal();
    const results = modal.querySelector('.visual-results');
    virtual = createVisualVirtualGrid(
        results, resultButton, loadWindow, paintRange, paintCategory);
    modal.querySelectorAll('[data-kind]').forEach(button => {
        button.onclick = () => {
            kind = button.dataset.kind;
            paintTabs();
            refresh();
        };
    });
    modal.querySelector('.visual-search').oninput = () => {
        clearTimeout(timer);
        timer = setTimeout(refresh, 180);
    };
    modal.querySelector('.visual-group').onchange = refresh;
    modal.querySelector('.visual-skin-tones input').onchange = refresh;
    modal.querySelector('.visual-cancel').onclick = close;
    modal.querySelector('.close-btn').onclick = close;
}

async function loadFresh() {
    const request = ++requestId;
    await loadGroups(request);
    if (request !== requestId) return;
    const query = modal.querySelector('.visual-search').value.trim();
    const selected = modal.querySelector('.visual-group').value;
    const separated = !query
        ? availableGroups.filter(group => !selected || group.id === selected)
            .map(group => ({ ...group, label: labelForGroup(group.id) }))
        : [];
    paintCategory('');
    await virtual.reset(separated);
}

async function loadWindow(offset, limit) {
    const args = {
        language: document.documentElement.lang || 'es',
        query: modal.querySelector('.visual-search').value,
        group: modal.querySelector('.visual-group').value || null,
        offset,
        limit,
    };
    if (kind === 'emoji') args.includeSkinTones = skinTonesEnabled();
    return invoke(CATALOGS[kind].search, args);
}

async function loadGroups(request) {
    const select = modal.querySelector('.visual-group');
    const variant = String(skinTonesEnabled());
    if (select.dataset.kind === kind && select.dataset.variant === variant) return;
    const previous = select.dataset.kind === kind ? select.value : '';
    const args = {
        language: document.documentElement.lang || 'es',
    };
    if (kind === 'emoji') args.includeSkinTones = skinTonesEnabled();
    const groups = await invoke(CATALOGS[kind].groups, args);
    if (request !== requestId) return;
    availableGroups = groups;
    select.replaceChildren(new Option(t('visuals.all_groups'), ''));
    groups.forEach(group => select.add(new Option(labelForGroup(group.id), group.id)));
    select.dataset.kind = kind;
    select.dataset.variant = variant;
    select.value = previous;
}

function paintRange(start, count, total) {
    const first = total ? start + 1 : 0;
    modal.querySelector('.visual-count').textContent =
        `${first}–${start + count} / ${total}`;
}

function paintCategory(label) {
    const query = modal.querySelector('.visual-search').value.trim();
    const selected = modal.querySelector('.visual-group').value;
    const fallback = query ? t('visuals.search_results')
        : selected ? labelForGroup(selected)
            : kind === 'emoji' ? '' : labelForKind(kind);
    const name = label || fallback;
    modal.querySelector('.visual-current-category').textContent = name
        ? t('visuals.current_category').replace('{name}', name) : '';
}

function skinTonesEnabled() {
    return kind === 'emoji'
        && modal.querySelector('.visual-skin-tones input').checked;
}

function resultButton(item) {
    return createResult(item, kind, value => {
        setChoice(kind, value);
        close();
    });
}

function paintTabs() {
    modal.querySelectorAll('[data-kind]').forEach(button => {
        button.classList.toggle('active', button.dataset.kind === kind);
    });
    modal.querySelector('.visual-skin-tones')
        .classList.toggle('hidden', kind !== 'emoji');
}

function close() {
    modal.classList.add('hidden');
    document.getElementById('edit-visual-open')?.focus();
}

function refresh() {
    loadFresh().catch(console.error);
}
