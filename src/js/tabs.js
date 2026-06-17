/**
 * Archivo: tabs.js
 * Propósito: Gestiona la barra de pestañas. Renderiza paletas del perfil activo
 * y delega cambios de estado a Rust mediante IPC. Regla 4.
 */

import { invoke } from './api.js';
import { placeMenu } from './menuPosition.js';
import { isMapping, captureTab } from './mapping.js';
import { t } from './i18n.js';
import { paintAdaptive } from './colorAdapter.js';
import { confirmDelete } from './deleteConfirm.js';
import { appAlert } from './appDialog.js';

let _config     = null;
let _onRefresh  = null;
let _ctxPaletaId = null;
let _wired      = false;

/** Inicializa la barra de pestañas y los listeners del menú contextual de pestaña. */
export function initTabs(config, onRefresh) {
    _config    = config;
    _onRefresh = onRefresh;

    if (_wired) {
        renderTabs(config);
        return;
    }
    _wired = true;

    document.getElementById('btn-add-tab')
        .addEventListener('click', () => _openTabModal(null));

    document.getElementById('tab-menu-editar')
        .addEventListener('click', () => { _hideCtxMenu(); _openTabModal(_ctxPaletaId); });

    document.getElementById('tab-menu-eliminar')
        .addEventListener('click', () => { _hideCtxMenu(); _deleteTab(_ctxPaletaId); });

    document.getElementById('tab-menu-exportar')
        .addEventListener('click', () => {
            _hideCtxMenu();
            import('./importer.js').then(m => m.exportTab());
        });

    document.getElementById('tab-menu-importar')
        .addEventListener('click', () => {
            _hideCtxMenu();
            import('./importer.js').then(m => m.importTab(_onRefresh));
        });

    document.addEventListener('click', () => _hideCtxMenu());

    renderTabs(config);
}

/** Actualiza la barra con nueva configuración (llamar tras cualquier cambio). */
export function updateTabs(config, onRefresh) {
    _config    = config;
    _onRefresh = onRefresh;
    renderTabs(config);
}

/** Dibuja las pestañas del perfil activo en #tabs-list. */
function renderTabs(config) {
    const profile = config.profiles.find(p => p.id === config.active_profile_id);
    if (!profile) return;

    const list = document.getElementById('tabs-list');
    list.innerHTML = '';

    profile.paletas.forEach(paleta => {
        const tab = document.createElement('div');
        tab.className = `tab${paleta.id === profile.active_paleta_id ? ' active' : ''}`;
        tab.dataset.paletaId = paleta.id;
        paintAdaptive(tab, paleta.tab_bg || '#3a3f44', paleta.tab_text || '#ffffff', 'tab');
        tab.textContent = paleta.shortcut
            ? `[${paleta.shortcut}] ${paleta.nombre}`
            : paleta.nombre;

        tab.addEventListener('click', async e => {
            if (isMapping()) {
                e.stopPropagation();
                captureTab(paleta, config.active_profile_id);
                return;
            }
            await invoke('set_active_paleta', {
                profileId: config.active_profile_id,
                paletaId:  paleta.id,
            });
            _onRefresh?.();
        });

        tab.addEventListener('contextmenu', e => {
            e.preventDefault();
            e.stopPropagation();
            _ctxPaletaId = paleta.id;
            placeMenu(document.getElementById('tab-context-menu'), e.clientX, e.clientY);
        });

        list.appendChild(tab);
    });
}

/** Marca pestañas no activas cuando alguno de sus botones está sonando. */
export function updateTabPlayback(payload) {
    const ticks = Array.isArray(payload) ? payload : (payload?.buttons ?? []);
    document.querySelectorAll('#tabs-list .tab[data-paleta-id]').forEach(tab => {
        const pid = tab.dataset.paletaId;
        const playing = !tab.classList.contains('active') &&
            ticks.some(t2 => t2.id?.startsWith(`${pid}_btn_`));
        tab.classList.toggle('tab-playing', playing);
    });
}

function _hideCtxMenu() {
    document.getElementById('tab-context-menu')?.classList.add('hidden');
}

async function _deleteTab(paletaId) {
    if (!paletaId || !_config) return;
    const action = await confirmDelete('tab');
    if (action === 'cancel') return;
    try {
        if (action === 'save_delete') {
            await invoke('export_tab_by_id', {
                profileId: _config.active_profile_id,
                paletaId,
            });
        }
        await invoke('delete_paleta', {
            profileId: _config.active_profile_id,
            paletaId,
        });
        _onRefresh?.();
    } catch (e) {
        if (_isCanceled(e)) return;
        // Rust devuelve códigos de error; aquí se traducen y muestran
        const key = `errors.${e}`;
        const msg = t(key);
        await appAlert(msg === key ? String(e) : msg);
    }
}

function _isCanceled(e) {
    return String(e).toLowerCase().includes('cancel');
}

function _openTabModal(paletaId) {
    const profile = _config?.profiles.find(p => p.id === _config.active_profile_id);
    const paleta  = paletaId ? profile?.paletas.find(p => p.id === paletaId) : null;
    import('./tabModal.js').then(m => m.openTabModal(_config, paleta, _onRefresh));
}
