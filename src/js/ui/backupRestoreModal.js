import { invoke } from '../bridge/api.js';
import {
    busyHtml, confirmHtml, createdHtml, errorText, homeHtml,
    modalHtml, restoreResultHtml,
} from './backupRestoreView.js';

let overlay;
let body;
let previousFocus;
let busy = false;

export async function initBackupRestore(options = {}) {
    ensureModal();
    const trigger = options.buttonId
        ? document.getElementById(options.buttonId) : null;
    trigger?.addEventListener('click', () => openHome(trigger));
    if (!options.checkResult) return false;
    try {
        const result = await invoke('backup_take_restore_result');
        if (!result) return false;
        openWith(restoreResultHtml(result), null);
        wireDone();
        return true;
    } catch (error) {
        console.error('backup_take_restore_result:', error);
        return false;
    }
}

function ensureModal() {
    if (overlay) return;
    overlay = document.createElement('div');
    overlay.id = 'backup-modal';
    overlay.className = 'hidden modal-overlay';
    overlay.innerHTML = modalHtml();
    document.body.appendChild(overlay);
    body = document.getElementById('backup-modal-body');
    document.getElementById('backup-modal-close').addEventListener('click', close);
    overlay.addEventListener('keydown', onKeyDown);
}

function openHome(trigger) {
    openWith(homeHtml(), trigger);
    document.getElementById('backup-create').addEventListener('click', createBackup);
    document.getElementById('backup-choose').addEventListener('click', chooseRestore);
    document.getElementById('backup-create').focus();
}

async function createBackup() {
    if (busy) return;
    showBusy('backup.creating');
    try {
        const summary = await invoke('backup_create');
        if (!summary) return openHome(previousFocus);
        body.innerHTML = createdHtml(summary);
        setBusy(false);
        wireDone();
    } catch (error) {
        openHome(previousFocus);
        showError(error);
    }
}

async function chooseRestore() {
    if (busy) return;
    showBusy('backup.checking');
    try {
        const inspection = await invoke('backup_choose_restore');
        if (!inspection) return openHome(previousFocus);
        body.innerHTML = confirmHtml(inspection.summary);
        setBusy(false);
        document.getElementById('backup-restore-cancel')
            .addEventListener('click', () => openHome(previousFocus));
        document.getElementById('backup-restore-confirm')
            .addEventListener('click', () => prepareRestore(inspection.sourcePath));
        document.getElementById('backup-restore-cancel').focus();
    } catch (error) {
        openHome(previousFocus);
        showError(error);
    }
}

async function prepareRestore(sourcePath) {
    if (busy) return;
    showBusy('backup.preparing');
    try {
        await invoke('backup_prepare_restore', { sourcePath });
        body.innerHTML = busyHtml('backup.restarting');
        await new Promise(resolve => setTimeout(resolve, 300));
        await invoke('backup_restart');
    } catch (error) {
        openHome(previousFocus);
        showError(error);
    }
}

function openWith(html, trigger) {
    previousFocus = trigger ?? document.activeElement;
    body.innerHTML = html;
    setBusy(false);
    overlay.classList.remove('hidden');
}

function close() {
    if (busy) return;
    overlay.classList.add('hidden');
    previousFocus?.focus?.();
    previousFocus = null;
}

function showBusy(key) {
    setBusy(true);
    body.innerHTML = busyHtml(key);
}

function setBusy(value) {
    busy = value;
    document.getElementById('backup-modal-close').disabled = value;
}

function showError(error) {
    setBusy(false);
    const status = document.getElementById('backup-status');
    if (!status) return;
    status.textContent = errorText(error);
    status.classList.add('error');
    status.focus?.();
}

function wireDone() {
    document.getElementById('backup-done')?.addEventListener('click', close);
    document.getElementById('backup-done')?.focus();
}

function onKeyDown(event) {
    if (event.key === 'Escape' && !busy) {
        event.preventDefault();
        return close();
    }
    if (event.key !== 'Tab') return;
    const controls = [...overlay.querySelectorAll(
        'button:not(:disabled),input:not(:disabled),select:not(:disabled)',
    )];
    if (!controls.length) return;
    const first = controls[0];
    const last = controls[controls.length - 1];
    if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
    }
}
