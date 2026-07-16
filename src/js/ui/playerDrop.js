/**
 * Archivo: playerDrop.js
 * Propósito: soltar carpetas y varios archivos en la cola del reproductor. Solo
 * aquí: la botonera sigue admitiendo un archivo y sin carpetas.
 *
 * Rust hace el trabajo y toma las decisiones (regla 4): recorre las carpetas,
 * cuenta, dice si hay que preguntar según el umbral y el ajuste del usuario, y
 * añade en segundo plano emitiendo progreso. Aquí solo se pregunta y se pinta.
 */

import { invoke, listen } from '../bridge/api.js';
import { t } from '../util/i18n.js';
import { appAlert, appConfirmRemember } from './appDialog.js';
import { drawPlayerView } from './playerView.js';
import { alertIpcError } from './ipcError.js';

let _wired = false;

/** Progreso de una carga larga: Rust avisa por lotes según va añadiendo. */
export function initPlayerDrop() {
    if (_wired) return;
    _wired = true;
    listen('player-drop-progress', e => _paintProgress(e.payload ?? {}));
}

/** Se pinta en la cabecera del reproductor, donde el operador está mirando. */
function _paintProgress({ done, total }) {
    const el = document.getElementById('player-drop-progress');
    if (!el) return;
    const running = total > 0 && done < total;
    el.classList.toggle('hidden', !running && done !== total);
    el.textContent = t('player.adding').replace('{done}', done).replace('{total}', total);
    if (!running) setTimeout(() => el.classList.add('hidden'), 1200);
    // La lista crece a la vista: Rust ya insertó este lote.
    drawPlayerView().catch(() => {});
}

/** Soltar sobre una fila inserta ahí; en el vacío, al final. */
export async function dropOnPlayer(target, paths) {
    const row = target.closest?.('#player-rows .player-row');
    const index = row ? Number(row.dataset.index) : undefined;

    // Rust cuenta y decide: la UI no conoce el umbral ni el ajuste.
    const scan = await invoke('player_scan_drop', { paths });
    if (!scan.count) return appAlert(t('errors.player_no_audio_found'));
    if (scan.blocked) return; // el usuario eligió "no añadir nunca"

    let remember = false;
    if (scan.needs_confirm) {
        const answer = await appConfirmRemember(
            t('player.many_tracks_confirm').replace('{count}', scan.count),
            t('player.remember_choice'),
            { ok: t('player.do_add'), cancel: t('app.dont_add') },
        );
        if (!answer.choice) return;
        remember = answer.remember;
    }

    try {
        await invoke('player_add_drop', { paths, index, remember });
    } catch (err) {
        await alertIpcError(err);
    }
    await drawPlayerView();
}
