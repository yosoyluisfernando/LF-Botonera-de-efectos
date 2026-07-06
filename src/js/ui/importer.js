/**
 * Archivo: importer.js
 * Propósito: Fachada para exportar/importar pestañas (.bdelf) y perfiles (.bdeplf).
 * Delega toda la lógica de serialización y el diálogo de archivo a Rust (Regla 4).
 */

import { invoke } from '../bridge/api.js';

const _CANCELLED = 'Operación cancelada.';

export async function exportTab() {
    try { await invoke('export_tab'); }
    catch (e) { if (String(e) !== _CANCELLED) console.error('Error al exportar pestaña:', e); }
}

export async function importTab(onRefresh) {
    try {
        await invoke('import_tab');
        onRefresh?.();
    } catch (e) { if (String(e) !== _CANCELLED) console.error('Error al importar pestaña:', e); }
}

export async function exportProfile() {
    try { await invoke('export_profile'); }
    catch (e) { if (String(e) !== _CANCELLED) console.error('Error al exportar perfil:', e); }
}

export async function importProfile(onRefresh) {
    try {
        await invoke('import_profile');
        onRefresh?.();
    } catch (e) { if (String(e) !== _CANCELLED) console.error('Error al importar perfil:', e); }
}
