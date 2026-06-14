/**
 * Archivo: tabs.js
 * Propósito: Gestiona la barra de pestañas. Renderiza paletas del perfil activo
 * y delega cambios de estado a Rust mediante IPC. Regla 4.
 */

import { invoke, listen } from './api.js';
import { placeMenu } from './menuPosition.js';
import { isMapping, captureTab } from './mapping.js';
import { t } from './i18n.js';

let _config     = null;
let _onRefresh  = null;
let _ctxPaletaId = null;

/** Inicializa la barra de pestañas y los listeners del menú contextual de pestaña. */
export function initTabs(config, onRefresh) {
    _config    = config;
    _onRefresh = onRefresh;

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

    // Pestaña en verde cuando alguno de sus botones está sonando (Fase 4).
    // Los ids de botón llevan el prefijo de su paleta: "{paletaId}_btn_{n}".
    listen('audio-tick', e => {
        const ticks = e.payload ?? [];
        document.querySelectorAll('#tabs-list .tab[data-paleta-id]').forEach(tab => {
            const pid = tab.dataset.paletaId;
            // Solo marcar pestañas NO visibles: el usuario ya ve la activa
            const playing = !tab.classList.contains('active') &&
                ticks.some(t2 => t2.id.startsWith(`${pid}_btn_`));
            tab.classList.toggle('tab-playing', playing);
        });
    });

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
        // Sin colores personalizados: las pestañas siguen al tema claro/oscuro.
        // tab_bg/tab_text se conservan en datos solo por compatibilidad LFA.
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

function _hideCtxMenu() {
    document.getElementById('tab-context-menu')?.classList.add('hidden');
}

async function _deleteTab(paletaId) {
    if (!paletaId || !_config) return;
    try {
        await invoke('delete_paleta', {
            profileId: _config.active_profile_id,
            paletaId,
        });
        _onRefresh?.();
    } catch (e) {
        // Rust devuelve códigos de error; aquí se traducen y muestran
        const key = `errors.${e}`;
        const msg = t(key);
        alert(msg === key ? e : msg);
    }
}

function _openTabModal(paletaId) {
    const profile = _config?.profiles.find(p => p.id === _config.active_profile_id);
    const paleta  = paletaId ? profile?.paletas.find(p => p.id === paletaId) : null;
    import('./tabModal.js').then(m => m.openTabModal(_config, paleta, _onRefresh));
}
