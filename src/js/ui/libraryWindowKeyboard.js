/** Navegación accesible de la tabla completa sin asignar acciones a Enter. */
import {
    activeLibraryPath, clearLibrarySelection, moveLibrarySelection,
    selectionForLibraryContext,
} from './librarySelection.js';
import { showLibraryContextMenu } from './libraryContextMenu.js';

export function initLibraryWindowKeyboard(container, getItems, getVirtual) {
    container.addEventListener('keydown', event =>
        navigate(event, container, getItems(), getVirtual()));
}

function navigate(event, container, items, virtual) {
    if (event.key === 'Escape') {
        clearLibrarySelection();
        return virtual.render();
    }
    if (event.key === 'ContextMenu' || (event.shiftKey && event.key === 'F10')) {
        event.preventDefault();
        return openContext(container, items);
    }
    const page = Math.max(1, Math.floor(container.clientHeight / virtual.rowHeight));
    const delta = event.key === 'ArrowDown' ? 1
        : event.key === 'ArrowUp' ? -1
            : event.key === 'PageDown' ? page
                : event.key === 'PageUp' ? -page
                    : event.key === 'Home' ? -items.length
                        : event.key === 'End' ? items.length : 0;
    if (!delta || !items.length) return;
    event.preventDefault();
    const index = moveLibrarySelection(items, delta, event.shiftKey);
    if (index >= 0) virtual.ensureVisible(index);
}

function openContext(container, items) {
    const item = items.find(value => value.path === activeLibraryPath());
    if (!item || item.is_directory) return;
    const selection = selectionForLibraryContext(item, items);
    const row = container.querySelector(`[data-path="${CSS.escape(item.path)}"]`);
    const rect = row?.getBoundingClientRect() ?? container.getBoundingClientRect();
    showLibraryContextMenu(rect.left + 14, rect.top + 14, selection);
}
