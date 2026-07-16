/**
 * Archivo: playerModes.js
 * Propósito: elegir el modo de avance desde el propio panel, sin entrar en
 * Ajustes. El icono del botón **es** el modo activo: así se sabe en qué modo
 * estás de un vistazo, que unas rayas mudas no dirían. Ojo: ∞ es repetir la
 * LISTA; el 🔂 del transporte repite UNA canción (ver `playerProgress.js`).
 */

import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { placeMenu } from '../util/menuPosition.js';

// El orden es el de Ajustes: la misma lista en los dos sitios.
// El modo dice QUÉ pista viene; si el reproductor se para al acabar lo decide el
// botón "detener al finalizar", que se combina con cualquiera de los tres.
const MODES = ['normal', 'repeat', 'random'];
// Mismo estilo de transporte que ▶ ⏸ ■ ⏭: símbolos monocromos, no emojis.
const ICONS = { normal: '→', repeat: '∞', random: '⇄' };

let _menu = null;
let _mode = 'normal';

export function initPlayerModes() {
    const button = document.getElementById('player-modes');
    button.addEventListener('click', _toggleMenu);
    // Un clic fuera cierra el menú, como el resto de menús de la aplicación.
    document.addEventListener('click', e => {
        if (!_menu || _menu.classList.contains('hidden')) return;
        if (!_menu.contains(e.target) && e.target !== button) _closeMenu();
    });
}

/** El modo lo manda Rust en cada tick: aquí solo se pinta. */
export function paintPlayerModes(mode) {
    if (!mode || mode === _mode) return;
    _mode = mode;
    const button = document.getElementById('player-modes');
    if (!button) return;
    button.textContent = ICONS[mode] ?? ICONS.normal;
    button.title = `${t('player.mode')} ${t(`player.mode_name_${mode}`)}`;
    if (_menu && !_menu.classList.contains('hidden')) _renderItems();
}

function _toggleMenu(event) {
    event.stopPropagation();
    if (_menu && !_menu.classList.contains('hidden')) return _closeMenu();
    if (!_menu) {
        _menu = document.createElement('div');
        _menu.className = 'player-modes-menu hidden';
        document.body.appendChild(_menu);
    }
    _renderItems();
    // Bajo el botón; `placeMenu` lo mete hacia dentro si no cupiera.
    const r = event.currentTarget.getBoundingClientRect();
    placeMenu(_menu, r.left, r.bottom + 2);
}

function _renderItems() {
    _menu.innerHTML = '';
    for (const mode of MODES) {
        const item = document.createElement('button');
        item.className = 'player-modes-item';
        item.classList.toggle('active', mode === _mode);
        // El check marca el elegido; solo puede haber uno.
        item.innerHTML = `<span class="player-modes-check">${mode === _mode ? '✓' : ''}</span>
            <span>${ICONS[mode]}</span><span>${t(`player.mode_name_${mode}`)}</span>`;
        item.addEventListener('click', async () => {
            _closeMenu();
            await invoke('player_set_mode', { mode }).catch(console.error);
        });
        _menu.appendChild(item);
    }
}

function _closeMenu() {
    _menu?.classList.add('hidden');
}
