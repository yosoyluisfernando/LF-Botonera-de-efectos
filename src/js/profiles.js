/**
 * Archivo: profiles.js
 * Propósito: Gestiona el botón de perfiles y su menú desplegable.
 * Delega toda persistencia a Rust (Regla 4).
 */

import { invoke } from './api.js';
import { placeMenu } from './menuPosition.js';
import { t } from './i18n.js';

let _config    = null;
let _onRefresh = null;

/** Inicializa el botón de perfiles y los listeners del menú contextual. */
export function initProfiles(config, onRefresh) {
    _config    = config;
    _onRefresh = onRefresh;

    document.getElementById('btn-profiles').addEventListener('click', e => {
        e.stopPropagation();
        const menu = document.getElementById('profile-context-menu');
        if (!menu.classList.contains('hidden')) {
            menu.classList.add('hidden');
            return;
        }
        const rect = e.currentTarget.getBoundingClientRect();
        placeMenu(menu, rect.left - 150, rect.bottom + 5);
    });

    document.addEventListener('click', e => {
        if (!e.target.closest('#profile-context-menu') && e.target.id !== 'btn-profiles') {
            document.getElementById('profile-context-menu')?.classList.add('hidden');
        }
    });

    document.getElementById('menu-new-profile').addEventListener('click', () => {
        _hideMenu();
        import('./profileModal.js').then(m => m.openProfileModal(_config, null, _onRefresh));
    });

    document.getElementById('menu-edit-profile').addEventListener('click', () => {
        _hideMenu();
        const active = _config.profiles.find(p => p.id === _config.active_profile_id);
        import('./profileModal.js').then(m => m.openProfileModal(_config, active, _onRefresh));
    });

    document.getElementById('menu-delete-profile').addEventListener('click', async () => {
        _hideMenu();
        try {
            await invoke('delete_profile', { id: _config.active_profile_id });
            _onRefresh?.();
        } catch (e) {
            // Rust devuelve códigos de error; aquí se traducen y muestran
            const key = `errors.${e}`;
            const msg = t(key);
            alert(msg === key ? e : msg);
        }
    });

    document.getElementById('menu-export-profile').addEventListener('click', () => {
        _hideMenu();
        import('./importer.js').then(m => m.exportProfile());
    });
    document.getElementById('menu-import-profile').addEventListener('click', () => {
        _hideMenu();
        import('./importer.js').then(m => m.importProfile(_onRefresh));
    });

    updateProfiles(config, onRefresh);
}

/** Actualiza el botón y la lista de perfiles con nueva configuración. */
export function updateProfiles(config, onRefresh) {
    _config    = config;
    _onRefresh = onRefresh;

    const profile = config.profiles.find(p => p.id === config.active_profile_id);
    if (!profile) return;

    const btn = document.getElementById('btn-profiles');
    btn.textContent = `👤 ${profile.name}`;
    btn.style.backgroundColor = profile.bg   || '#008c3a';
    btn.style.color           = profile.text || '#ffffff';

    _renderProfileList(config);
}

function _renderProfileList(config) {
    const list = document.getElementById('profile-list');
    list.innerHTML = '';
    config.profiles.forEach(p => {
        const li = document.createElement('li');
        li.textContent = p.name;
        if (p.id === config.active_profile_id) li.classList.add('active-profile');
        li.addEventListener('click', async () => {
            _hideMenu();
            await invoke('set_active_profile', { id: p.id });
            _onRefresh?.();
        });
        list.appendChild(li);
    });
}

function _hideMenu() {
    document.getElementById('profile-context-menu')?.classList.add('hidden');
}
