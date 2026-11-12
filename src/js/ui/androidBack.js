import { invoke, listen } from '../bridge/api.js';
import { isAndroid } from '../bridge/platform.js';
import { t } from '../util/i18n.js';
import { appConfirm } from './appDialog.js';
import { cancelMidiCapture, hasActiveMidiCapture } from './midiControls.js';
import { exitMapping, isMapping } from './mapping.js';
import { toggleFixedPanel } from './fixedPanel.js';

let wired = false;
let exitPending = false;

export async function initAndroidBack() {
    if (!isAndroid() || wired) return;
    const unlisten = await listen('android-back', () => {
        handleAndroidBack().catch(console.error);
    });
    try {
        await invoke('android_back_attach');
        wired = true;
    } catch (error) {
        unlisten();
        throw error;
    }
}

// La prioridad depende de las superficies visibles del DOM, no de un historial ficticio.
export async function handleAndroidBack() {
    const loading = document.getElementById('loading-screen');
    if (visible(loading)) return;
    const dialog = document.querySelector('dialog[open]');
    if (dialog) { dialog.close(); return; }
    if (hasActiveMidiCapture()) {
        await cancelMidiCapture();
        return;
    }
    const menu = [...document.querySelectorAll('.context-style:not(.hidden), .master-volume-menu:not(.hidden), .player-volume-panel:not(.hidden)')].find(visible);
    if (menu) {
        document.body.click();
        return;
    }
    if (visible(document.getElementById('master-volume-panel'))) {
        document.getElementById('master-volume-btn')?.click();
        return;
    }
    const modal = topModal();
    if (modal) {
        if (modal.id === 'settings-modal' && modal.dataset.androidView === 'detail') {
            modal.querySelector('.android-settings-back')?.click();
        } else {
            cancelModal(modal);
        }
        return;
    }
    if (isMapping()) {
        exitMapping();
        return;
    }
    if (visible(document.getElementById('prelisten-player'))) {
        document.getElementById('close-prelisten')?.click();
        return;
    }
    if (visible(document.getElementById('live-player'))) {
        document.getElementById('close-live')?.click();
        return;
    }
    if (visible(document.getElementById('fixed-panel'))) {
        await toggleFixedPanel();
        return;
    }
    if (exitPending) return;
    exitPending = true;
    try {
        const confirmed = await appConfirm(t('mobile.exit_confirm'), {
            ok: t('mobile.exit_app'), cancel: t('mobile.stay_in_app'),
        });
        if (confirmed) await invoke('android_exit_app');
    } finally {
        exitPending = false;
    }
}

function visible(element) {
    return !!element && !element.classList.contains('hidden') && element.getClientRects().length > 0;
}

function topModal() {
    return [...document.querySelectorAll('.modal-overlay:not(.hidden)')]
        .filter(visible)
        .sort((a, b) => (Number(getComputedStyle(a).zIndex) || 0) - (Number(getComputedStyle(b).zIndex) || 0))
        .at(-1);
}

function cancelModal(modal) {
    const actions = {
        'app-dialog-modal': '.app-dialog-cancel, .close-btn',
        'edit-modal': '#btn-cancel-edit',
        'track-editor-modal': '#te-close',
        'capture-modal': '#btn-cancel-capture',
        'color-picker-modal': '#color-picker-cancel',
        'delete-confirm-modal': '#btn-delete-cancel',
        'preload-dialog': '#preload-dialog-skip',
        'norm-config-modal': '#nc-cancel',
        'release-notes-modal': '#release-notes-close',
        'donation-modal': '#donation-later',
        'backup-modal': '#backup-restore-cancel, #backup-modal-close',
        'console-modal': '#close-console',
    };
    const selector = actions[modal.id] ?? '.visual-cancel, .close-btn, [data-close]';
    const action = [...modal.querySelectorAll(selector)].find(button => visible(button) && !button.disabled);
    action?.click();
}
