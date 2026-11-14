/**
 * Archivo: buttonSearch.js
 * Propósito: buscador global de efectos (lupa a la izquierda de las pestañas).
 * Rust busca en TODAS las pestañas del perfil (search_buttons); aquí solo se
 * pinta la lista y, al elegir, se cambia de pestaña y se resalta el botón.
 * Buscar no toca el audio: lo que suena sigue sonando.
 */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

const DEBOUNCE_MS = 90;
const HIGHLIGHT_MS = 3000;

let _getConfig = () => null;
let _onRefresh = null;
let _panel = null;
let _input = null;
let _list = null;
let _hits = [];
let _active = -1;
let _timer = null;
let _seq = 0;

export function initButtonSearch(getConfig, onRefresh) {
    _getConfig = getConfig;
    _onRefresh = onRefresh;
    const btn = document.getElementById('btn-button-search');
    if (!btn || _panel) return;
    _build();
    btn.addEventListener('click', e => {
        e.stopPropagation();
        if (_panel.classList.contains('hidden')) _open(btn); else _close();
    });
    document.addEventListener('mousedown', e => {
        if (!_panel.classList.contains('hidden') && !_panel.contains(e.target) && e.target !== btn
            && !btn.contains(e.target)) _close();
    });
}

function _build() {
    _panel = document.createElement('div');
    _panel.className = 'button-search-panel hidden';
    _panel.setAttribute('role', 'dialog');
    _panel.innerHTML = `
        <input type="search" autocomplete="off" spellcheck="false">
        <p class="button-search-hint"></p>
        <div class="button-search-results" role="listbox"></div>`;
    _input = _panel.querySelector('input');
    _list = _panel.querySelector('.button-search-results');
    _input.addEventListener('input', () => {
        clearTimeout(_timer);
        _timer = setTimeout(_search, DEBOUNCE_MS);
    });
    _input.addEventListener('keydown', _onKey);
    document.body.appendChild(_panel);
}

function _open(anchor) {
    _input.placeholder = t('search.placeholder');
    _panel.querySelector('.button-search-hint').textContent = t('search.hint');
    const r = anchor.getBoundingClientRect();
    _panel.style.left = `${Math.max(8, r.left)}px`;
    _panel.style.top = `${r.bottom + 4}px`;
    _panel.classList.remove('hidden');
    _input.select();
    _input.focus();
    if (_input.value.trim()) _search();
}

function _close() {
    _panel.classList.add('hidden');
}

async function _search() {
    const query = _input.value.trim();
    const seq = ++_seq;
    const hits = query ? await invoke('search_buttons', { query }).catch(() => []) : [];
    if (seq !== _seq) return; // llegó tarde: ya se escribió otra cosa
    _hits = hits;
    _active = hits.length ? 0 : -1;
    _render(query);
}

function _render(query) {
    _list.innerHTML = '';
    if (query && !_hits.length) {
        const empty = document.createElement('div');
        empty.className = 'button-search-empty';
        empty.textContent = t('search.no_results');
        _list.appendChild(empty);
        return;
    }
    _hits.forEach((hit, i) => {
        const row = document.createElement('div');
        row.className = `button-search-row${i === _active ? ' active' : ''}`;
        row.setAttribute('role', 'option');
        const name = document.createElement('span');
        name.className = 'name';
        name.textContent = hit.label || hit.file_name;
        const where = document.createElement('span');
        where.className = 'where';
        where.textContent = t('search.location')
            .replace('{tab}', hit.paleta_nombre)
            .replace('{n}', hit.tab_position)
            .replace('{index}', hit.index)
            .replace('{row}', hit.row)
            .replace('{col}', hit.col);
        row.append(name, where);
        row.addEventListener('mouseenter', () => _setActive(i));
        row.addEventListener('click', () => _choose(hit));
        _list.appendChild(row);
    });
}

function _setActive(i) {
    _active = i;
    _list.querySelectorAll('.button-search-row').forEach((row, n) => {
        row.classList.toggle('active', n === i);
        if (n === i) row.scrollIntoView({ block: 'nearest' });
    });
}

function _onKey(e) {
    if (e.key === 'Escape') { e.stopPropagation(); _close(); return; }
    if (!_hits.length) return;
    if (e.key === 'ArrowDown') { e.preventDefault(); _setActive((_active + 1) % _hits.length); }
    if (e.key === 'ArrowUp') { e.preventDefault(); _setActive((_active - 1 + _hits.length) % _hits.length); }
    if (e.key === 'Enter' && _active >= 0) { e.preventDefault(); _choose(_hits[_active]); }
}

/** Lleva a la pestaña del resultado y hace parpadear el botón. */
async function _choose(hit) {
    _close();
    const cfg = _getConfig();
    if (cfg && cfg.profiles.find(p => p.id === cfg.active_profile_id)?.active_paleta_id !== hit.paleta_id) {
        await invoke('set_active_paleta', { profileId: cfg.active_profile_id, paletaId: hit.paleta_id });
        await _onRefresh?.();
    }
    const cell = document.querySelector(`#main-grid .grid-item[data-id="${CSS.escape(hit.button_id)}"]`);
    if (!cell) return;
    cell.scrollIntoView({ block: 'nearest', inline: 'nearest' });
    cell.classList.remove('search-found');
    void cell.offsetWidth; // reinicia la animación si se busca dos veces seguidas
    cell.classList.add('search-found');
    setTimeout(() => cell.classList.remove('search-found'), HIGHLIGHT_MS);
}
