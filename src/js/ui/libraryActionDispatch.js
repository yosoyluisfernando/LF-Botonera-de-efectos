/** Una sola implementación de acciones para panel y ventana Biblioteca. */
import { emit, invoke } from '../bridge/api.js';
import { drawPlayerView } from './playerView.js';

export async function dispatchLibraryAction(action, selection, onEdited) {
    if (document.body.classList.contains('library-window')) {
        return emit('library-window-action', { action, selection });
    }
    return executeLibraryAction(action, selection, onEdited);
}

export async function executeLibraryAction(action, selection, onEdited) {
    const item = selection[0];
    if (action === 'live') {
        const module = await import('./libraryLivePlayer.js');
        return module.openLivePlayer(item);
    }
    if (action === 'preview') {
        const module = await import('./prelisten.js');
        return module.openPrelisten(
            item.path, item.title || item.file_name, 1, item.duration_s);
    }
    if (action === 'player') {
        await invoke('player_add_tracks', {
            paths: selection.map(value => value.path),
        });
        return drawPlayerView();
    }
    if (action === 'editor') {
        const module = await import('./trackEditor.js');
        return module.openPreferredTrackEditor(
            item.path, item.title || item.file_name, onEdited);
    }
}
