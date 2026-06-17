/**
 * Archivo: shortcutSave.js
 * Proposito: muestra confirmaciones de conflictos reportados por Rust.
 */

import { invoke } from './api.js';
import { t } from './i18n.js';
import { appAlert, appConfirm } from './appDialog.js';

/** Ejecuta un comando de guardado y confirma solo conflictos reemplazables. */
export async function invokeShortcutSave(command, payload) {
    try {
        return await invoke(command, payload);
    } catch (error) {
        const conflict = _parseConflict(error);
        if (!conflict) throw error;

        if (conflict.code === 'shortcut_blocked_global') {
            await appAlert(_format(t('shortcuts.blocked_global'), conflict));
            throw error;
        }
        if (conflict.code === 'shortcut_reserved_system') {
            await appAlert(_format(t('shortcuts.reserved_system'), conflict));
            throw error;
        }

        const messageKey = {
            shortcut_conflict_button: 'shortcuts.replace_button',
            shortcut_conflict_tab: 'shortcuts.replace_tab',
            shortcut_conflict_button_any: 'shortcuts.replace_button_any',
        }[conflict.code];
        if (!messageKey || !await appConfirm(_format(t(messageKey), conflict))) throw error;
        return await invoke(command, { ...payload, replaceShortcut: true });
    }
}

function _parseConflict(error) {
    const parts = String(error ?? '').split('|');
    if (!parts[0]?.startsWith('shortcut_')) return null;
    return { code: parts[0], key: parts[1] ?? '', target: parts[2] ?? '' };
}

function _format(text, conflict) {
    return text
        .replaceAll('{key}', conflict.key)
        .replaceAll('{target}', conflict.target);
}
