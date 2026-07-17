/**
 * Archivo: playerView.js
 * Proposito: dibuja el modo reproductor del panel fijo (cola + transporte) y
 * envia los comandos al motor propio. La UI solo pinta: Rust decide el avance.
 * Verde = pista sonando; naranja = pista marcada como siguiente.
 */

import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { typeIcon } from '../util/typeIcons.js';
import { mmss } from '../util/durationFormat.js';
import { appConfirm3 } from './appDialog.js';
import { alertIpcError } from './ipcError.js';
import { initPlayerProgress, paintPlayerProgress, setQueueTotal } from './playerProgress.js';
import { initPlayerModes, paintPlayerModes } from './playerModes.js';
import { initPlayerVolume, paintPlayerVolume } from './playerVolume.js';

let _wired = false;
let _tracks = [];

export function initPlayerView() {
    if (_wired) return;
    _wired = true;
    initPlayerProgress();
    initPlayerModes();
    initPlayerVolume();
    _on('player-play', () => invoke('player_resume'));
    _on('player-pause', () => invoke('player_pause'));
    _on('player-stop', () => invoke('player_stop'));
    _on('player-next', () => invoke('player_next'));
    // El repintado llega solo por "player-tick": cambiar el estado ya lo dispara.
    _on('player-stop-after', async () => {
        const snap = await invoke('get_player_snapshot');
        await invoke('player_set_stop_after', { enabled: !snap.stop_after });
    });
    _on('player-save', _save);
    _on('player-clear', () => _replaceQueue('player.save_before_clear', _clear));
    _on('player-open', () => _replaceQueue('player.save_before_open', _open));
}

/**
 * Limpiar y Abrir borran la cola actual, asi que antes ofrecen guardarla:
 * Guardar / No guardar / Cancelar. Si el guardado se cancela, no se borra nada.
 */
async function _replaceQueue(promptKey, action) {
    const { tracks } = await invoke('get_player');
    if (tracks.length) {
        const choice = await appConfirm3(t(promptKey), {
            ok: t('player.do_save'), alt: t('player.dont_save'), cancel: t('edit_modal.cancel'),
        });
        if (choice === 'cancel') return;
        if (choice === 'yes' && !await _save()) return;
    }
    await action();
}

/** Guarda la lista. Devuelve false si el usuario cerro el dialogo. */
async function _save() {
    try {
        return await invoke('player_save_playlist');
    } catch (err) {
        await alertIpcError(err);
        return false;
    }
}

async function _clear() {
    await invoke('player_clear_queue');
    await drawPlayerView();
}

async function _open() {
    try {
        if (await invoke('player_open_playlist')) await drawPlayerView();
    } catch (err) {
        await alertIpcError(err);
    }
}

/** Redibuja la cola completa. Solo tras editarla o al cambiar de vista. */
export async function drawPlayerView() {
    const view = await invoke('get_player');
    _tracks = view.tracks;
    const rows = document.getElementById('player-rows');
    rows.innerHTML = '';
    // Con la lista vacía no hay nada que hacer evidente: los botones fijos tienen
    // su "+", pero aquí las canciones entran arrastrándolas y eso no se ve por
    // ningún lado. Hasta que haya un explorador propio, se dice con palabras.
    if (!view.tracks.length) rows.appendChild(_emptyHint());
    view.tracks.forEach((track, position) => rows.appendChild(_row(track, position)));
    // El total lo suma Rust: que cuenta y que no es regla de negocio, no de la UI.
    setQueueTotal(view.total_s, view.time_display);
    paintPlayerTick(view.snapshot);
}

/** Pinta el estado en vivo: verde (sonando), naranja (siguiente) y el tiempo. */
export function paintPlayerTick(snapshot) {
    const rows = document.querySelectorAll('#player-rows .player-row');
    rows.forEach(row => {
        const position = Number(row.dataset.index);
        const isNext = snapshot.next_index === position;
        row.classList.toggle('playing', snapshot.current_index === position);
        // Naranja = "suena a continuación". Con "detener al finalizar" puesto no
        // va a sonar sola, así que se marca en gris: sigue siendo lo elegido y se
        // respeta, pero avisa de que habrá que pulsar play. Parado no aplica: no
        // hay una "actual" tras la que sonar, y ahí el naranja es la guía de por
        // dónde se retomaría.
        const held = isNext && snapshot.stop_after && snapshot.playing;
        row.classList.toggle('next', isNext && !held);
        row.classList.toggle('next-held', held);
    });
    document.getElementById('player-stop-after').classList.toggle('active', !!snapshot.stop_after);
    document.getElementById('player-play').classList.toggle('active', !!snapshot.playing);
    paintPlayerProgress(snapshot);
    paintPlayerModes(snapshot.mode);
    paintPlayerVolume(snapshot.volume);
}

/** El cartel de la lista vacía. Se cae solo en cuanto entre la primera canción:
 *  `drawPlayerView` vacía las filas y solo lo vuelve a poner si sigue sin haber. */
function _emptyHint() {
    const el = document.createElement('p');
    el.className = 'player-empty';
    el.textContent = t('player.empty_hint');
    return el;
}

function _row(track, position) {
    const el = document.createElement('div');
    el.className = 'player-row';
    el.dataset.index = position;
    const icon = track.type_icon ? typeIcon(track.type_icon) : '';
    // Los tipos especiales no se resuelven hasta sonar: no hay duracion que
    // mostrar. Misma convencion que el Automatizador.
    const dur = track.type === 'audio' ? mmss(track.duration) : '--:--';
    el.innerHTML = `<span class="player-row-title">${icon}
            <span class="player-row-text">${track.name || track.label}</span>
        </span>
        <span class="player-row-dur">${dur}</span>`;
    // Doble clic: Rust decide segun suene o no (reproducir / marcar siguiente).
    // Un clic no hace nada: marcar sin querer al rozar una fila era problematico.
    el.addEventListener('dblclick', () => invoke('player_activate_index', { index: position }));
    el.title = t('player.row_hint');
    return el;
}

function _on(id, handler) {
    document.getElementById(id).addEventListener('click', () => {
        Promise.resolve(handler()).catch(console.error);
    });
}
