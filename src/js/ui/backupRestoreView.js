import { t } from '../util/i18n.js';

export function modalHtml() {
    return `
      <div class="modal-content backup-modal-content" role="dialog" aria-modal="true"
           aria-labelledby="backup-modal-title" aria-describedby="backup-modal-body">
        <div class="modal-header">
          <h3 id="backup-modal-title">${e(t('backup.modal_title'))}</h3>
          <button id="backup-modal-close" class="close-btn"
                  aria-label="${e(t('backup.close'))}">×</button>
        </div>
        <div id="backup-modal-body" class="modal-body backup-modal-body"></div>
      </div>`;
}

export function homeHtml() {
    return `
      <p class="backup-intro">${e(t('backup.intro'))}</p>
      <p class="backup-warning"><strong>${e(t('backup.audio_warning'))}</strong></p>
      <div class="backup-actions">
        <section class="backup-card">
          <h4>${e(t('backup.create_title'))}</h4>
          <p>${e(t('backup.create_description'))}</p>
          <button id="backup-create" class="btn-blue">${e(t('backup.create_action'))}</button>
        </section>
        <section class="backup-card">
          <h4>${e(t('backup.restore_title'))}</h4>
          <p>${e(t('backup.restore_description'))}</p>
          <button id="backup-choose" class="btn-dark">${e(t('backup.restore_action'))}</button>
        </section>
      </div>
      <p id="backup-status" class="backup-status" role="status" aria-live="polite"></p>`;
}

export function busyHtml(messageKey) {
    return `
      <div class="backup-busy" role="status" aria-live="assertive">
        <span class="backup-spinner" aria-hidden="true"></span>
        <strong>${e(t(messageKey))}</strong>
      </div>`;
}

export function createdHtml(summary) {
    return `
      <h4 class="backup-result-title">${e(t('backup.create_success'))}</h4>
      <p>${e(t('backup.create_verified'))}</p>
      ${summaryHtml(summary)}
      <p class="backup-warning"><strong>${e(t('backup.audio_warning'))}</strong></p>
      <div class="modal-footer">
        <button id="backup-done" class="btn-blue">${e(t('backup.close'))}</button>
      </div>`;
}

export function confirmHtml(summary) {
    return `
      <h4 class="backup-result-title">${e(t('backup.restore_confirm_title'))}</h4>
      <p>${e(t('backup.restore_confirm_intro'))}</p>
      ${summaryHtml(summary)}
      <p>${e(t('backup.emergency_notice'))}</p>
      <p class="backup-restart-warning"><strong>${e(t('backup.restart_warning'))}</strong></p>
      <div class="modal-footer">
        <button id="backup-restore-cancel" class="btn-dark">${e(t('backup.cancel'))}</button>
        <button id="backup-restore-confirm" class="btn-danger">${e(t('backup.restore_and_restart'))}</button>
      </div>`;
}

export function restoreResultHtml(result) {
    if (result.success) {
        return `
          <h4 class="backup-result-title">${e(t('backup.restore_success_title'))}</h4>
          <p>${e(t('backup.restore_success'))}</p>
          ${result.summary ? summaryHtml(result.summary) : ''}
          ${emergencyHtml(result.emergencyBackupPath)}
          ${closeFooter()}`;
    }
    return `
      <h4 class="backup-result-title">${e(t('backup.restore_failed_title'))}</h4>
      <p>${e(t('backup.restore_failed'))}</p>
      <p class="backup-warning">${e(errorText(result.error))}</p>
      ${emergencyHtml(result.emergencyBackupPath)}
      ${closeFooter()}`;
}

export function errorText(raw) {
    const code = String(raw);
    if (code.includes('destination_exists')) return t('backup.error_destination_exists');
    if (code.includes('format_incompatible') || code.includes('database_newer')) {
        return t('backup.error_incompatible');
    }
    if (code.includes('integrity') || code.includes('manifest')
        || code.includes('config_invalid') || code.includes('invalid_file')) {
        return t('backup.error_damaged');
    }
    if (code.includes('write') || code.includes('sync') || code.includes('copy')
        || code.includes('replace') || code.includes('directory')) {
        return t('backup.error_write');
    }
    if (code.includes('restore_pending')) return t('backup.error_pending');
    return t('backup.error_generic');
}

function summaryHtml(summary) {
    return `
      <h4>${e(t('backup.summary_title'))}</h4>
      <dl class="backup-summary">
        ${item('backup.created_at', formatDate(summary.createdAt))}
        ${item('backup.version', summary.appVersion)}
        ${item('backup.profiles', number(summary.profileCount))}
        ${item('backup.palettes', number(summary.paletteCount))}
        ${item('backup.tracks', number(summary.trackCount))}
        ${item('backup.file_size', bytes(summary.fileSize))}
        ${item('backup.integrity', t('backup.integrity_ok'), 'backup-integrity-ok')}
        ${item('backup.audio', t('backup.audio_not_included'))}
        ${item('backup.location', summary.path)}
      </dl>`;
}

function item(key, value, className = '') {
    return `<dt>${e(t(key))}</dt><dd class="${className}">${e(value)}</dd>`;
}

function emergencyHtml(path) {
    if (!path) return '';
    return `<p><strong>${e(t('backup.emergency_location'))}</strong><br>${e(path)}</p>`;
}

function closeFooter() {
    return `<div class="modal-footer">
      <button id="backup-done" class="btn-blue">${e(t('backup.close'))}</button>
    </div>`;
}

function formatDate(value) {
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? String(value)
        : date.toLocaleString(document.documentElement.lang);
}

function number(value) {
    return Number(value ?? 0).toLocaleString(document.documentElement.lang);
}

function bytes(value) {
    const size = Number(value ?? 0);
    const scale = size < 1024 * 1024 ? 1024 : 1024 * 1024;
    const unit = scale === 1024 ? 'KB' : 'MB';
    const formatted = new Intl.NumberFormat(document.documentElement.lang, {
        minimumFractionDigits: 1,
        maximumFractionDigits: 1,
    }).format(size / scale);
    return `${formatted} ${unit}`;
}

function e(value) {
    const node = document.createElement('span');
    node.textContent = String(value ?? '');
    return node.innerHTML;
}
