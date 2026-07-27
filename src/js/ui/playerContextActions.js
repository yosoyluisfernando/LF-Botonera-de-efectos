/**
 * Acciones exclusivas del menú contextual de una fila del reproductor.
 * El menú compartido decide antes cuáles están habilitadas.
 */

export function wirePlayerContextActions(track, onUpdate, onRemove, single) {
    const name = track.name || track.label;
    const previaEl = document.getElementById('menu-previa');
    const editTrackEl = document.getElementById('menu-editar-pista');
    const removeEl = document.getElementById('menu-player-remove');
    const hide = () => document.getElementById('context-menu')?.classList.add('hidden');
    const onPrevia = () => {
        if (!single || !track.can_prelisten) return;
        hide();
        import('./prelisten.js').then(m =>
            m.openPrelisten(track.path, name, track.vol ?? 1.0, track.duration ?? 0));
    };
    const onEditTrack = () => {
        if (!single || !track.can_prelisten) return;
        hide();
        import('./trackEditor.js').then(m =>
            m.openPreferredTrackEditor(track.path, name, onUpdate));
    };
    const remove = () => {
        hide();
        onRemove?.();
    };
    previaEl.addEventListener('click', onPrevia);
    editTrackEl.addEventListener('click', onEditTrack);
    removeEl.addEventListener('click', remove);
    return () => {
        previaEl.removeEventListener('click', onPrevia);
        editTrackEl.removeEventListener('click', onEditTrack);
        removeEl.removeEventListener('click', remove);
    };
}
