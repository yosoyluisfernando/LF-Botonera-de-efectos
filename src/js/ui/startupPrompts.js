import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

let _prompts = null;
let _resolveRelease = null;
let _resolveDonation = null;

const DONATION_URL = 'https://www.paypal.com/donate/?hosted_button_id=3JJVFFBVR4MQQ';

export async function runStartupPrompts() {
    try {
        _prompts = await invoke('prepare_startup_prompts');
    } catch (e) {
        console.error('prepare_startup_prompts:', e);
        return;
    }
    if (_prompts.showReleaseNotes) await _showReleaseNotes();
    if (_prompts.donationDue) await _showDonationPrompt();
}

function _showReleaseNotes() {
    return new Promise(resolve => {
        _resolveRelease = resolve;
        document.getElementById('release-notes-version').textContent = _prompts.currentVersion;
        document.getElementById('release-notes-body').innerHTML = _renderNotes(_prompts.releaseNotes);
        document.getElementById('release-notes-modal').classList.remove('hidden');
    });
}

export function initStartupPrompts() {
    _ensureReleaseModal();
    _ensureDonationModal();
    document.getElementById('release-notes-close')?.addEventListener('click', _closeReleaseNotes);
    document.getElementById('release-notes-ok')?.addEventListener('click', _closeReleaseNotes);
    document.getElementById('donation-later')?.addEventListener('click', _closeDonationPrompt);
    document.getElementById('donation-support')?.addEventListener('click', _supportAndCloseDonation);
}

function _ensureReleaseModal() {
    if (document.getElementById('release-notes-modal')) return;
    const el = document.createElement('div');
    el.id = 'release-notes-modal';
    el.className = 'hidden modal-overlay';
    el.style.zIndex = '4450';
    el.innerHTML = `
        <div class="modal-content modal-lg">
            <div class="modal-header">
                <h3>${t('release_notes.title')} <span id="release-notes-version"></span></h3>
                <button id="release-notes-close" class="close-btn">x</button>
            </div>
            <div class="modal-body release-notes-body" id="release-notes-body"></div>
            <div class="modal-footer">
                <button id="release-notes-ok" class="btn-blue">${t('release_notes.ok')}</button>
            </div>
        </div>`;
    document.body.appendChild(el);
}

function _ensureDonationModal() {
    if (document.getElementById('donation-modal')) return;
    const el = document.createElement('div');
    el.id = 'donation-modal';
    el.className = 'hidden modal-overlay';
    el.style.zIndex = '4440';
    el.innerHTML = `
        <div class="modal-content modal-lg startup-modal">
            <div class="modal-header"><h3>${t('donation.title')}</h3></div>
            <div class="modal-body donation-body">
                <p>${t('donation.p1')}</p>
                <p>${t('donation.p2')}</p>
                <p>${t('donation.p3')}</p>
                <p class="donation-sign">${t('donation.sign')}</p>
            </div>
            <div class="modal-footer">
                <button id="donation-later" class="btn-dark">${t('donation.later')}</button>
                <button id="donation-support" class="btn-blue">${t('donation.support')}</button>
            </div>
        </div>`;
    document.body.appendChild(el);
}

async function _closeReleaseNotes() {
    try {
        await invoke('mark_release_notes_seen');
    } catch (e) {
        console.error('mark_release_notes_seen:', e);
    }
    document.getElementById('release-notes-modal')?.classList.add('hidden');
    _resolveRelease?.();
    _resolveRelease = null;
}

function _renderNotes(markdown) {
    return markdown
        .split(/\r?\n/)
        .map(line => _renderLine(line.trim()))
        .filter(Boolean)
        .join('');
}

function _showDonationPrompt() {
    return new Promise(resolve => {
        _resolveDonation = resolve;
        document.getElementById('donation-modal').classList.remove('hidden');
    });
}

async function _supportAndCloseDonation() {
    window.__TAURI__?.opener?.openUrl(DONATION_URL).catch(console.error);
    await _closeDonationPrompt();
}

async function _closeDonationPrompt() {
    try {
        await invoke('mark_donation_prompt_shown');
    } catch (e) {
        console.error('mark_donation_prompt_shown:', e);
    }
    document.getElementById('donation-modal')?.classList.add('hidden');
    _resolveDonation?.();
    _resolveDonation = null;
}

function _renderLine(line) {
    if (!line) return '';
    // Un separador de Markdown (--- entre versiones) no es contenido: se omite,
    // o el lector de pantalla lo leería como "guion guion guion".
    if (/^-{3,}$/.test(line)) return '';
    if (line.startsWith('### ')) return `<h4>${_escape(line.slice(4))}</h4>`;
    if (line.startsWith('- ')) return `<p class="release-note-item">${_inline(line.slice(2))}</p>`;
    return `<p>${_inline(line)}</p>`;
}

function _inline(text) {
    return _escape(text).replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>').replace(/`(.+?)`/g, '<code>$1</code>');
}

function _escape(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
