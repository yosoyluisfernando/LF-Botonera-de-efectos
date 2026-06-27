/**
 * Archivo: preloadDialog.js
 * Propósito: diálogo de PRIMER ARRANQUE de la precarga. Solo UI (Regla 4): RUST
 * decide si mostrarlo (should_prompt_preload) y valida/guarda (set_preload_config,
 * mark_preload_prompted). Reutiliza el formulario de settingsPreload.js para no
 * duplicar. El JS aquí solo muestra/oculta el modal y reenvía a Rust.
 */
import { invoke } from './api.js';
import { renderPreloadForm, fillPreloadForm, savePreloadFrom } from './settingsPreload.js';

/** Si Rust indica que toca, muestra el diálogo una sola vez. */
export async function maybeShowPreloadDialog() {
    await showPreloadPrompt();
}

export async function showPreloadPrompt(onDone = null) {
    let show = false;
    try {
        show = await invoke('should_prompt_preload');
    } catch (_) {
        return; // sin backend (navegador) o error: no molestar
    }
    if (!show) { onDone?.(); return; }

    const modal = document.getElementById('preload-dialog');
    const body = document.getElementById('preload-dialog-body');
    renderPreloadForm(body);
    try {
        fillPreloadForm(body, await invoke('get_preload_config'));
    } catch (_) { /* usa los valores por defecto del formulario */ }
    modal.classList.remove('hidden');

    document.getElementById('preload-dialog-save').onclick = async () => {
        try {
            await savePreloadFrom(body);
        } catch (e) {
            console.error('Error al guardar precarga:', e);
        }
        modal.classList.add('hidden');
        onDone?.();
    };

    document.getElementById('preload-dialog-skip').onclick = async () => {
        // No cambia ajustes; solo marca que ya se preguntó (lo registra Rust).
        try {
            await invoke('mark_preload_prompted');
        } catch (e) {
            console.error('Error al marcar precarga:', e);
        }
        modal.classList.add('hidden');
        onDone?.();
    };
}
