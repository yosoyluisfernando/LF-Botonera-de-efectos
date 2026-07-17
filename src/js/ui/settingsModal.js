/**
 * Archivo: settingsModal.js
 * Propósito: Controla el modal de Configuración Global (salidas de audio,
 * atajos de teclado, acerca de). Delega audio a Rust (Regla 4).
 */

import { emit, invoke } from '../bridge/api.js';
import { applyTheme } from './theme.js';
import { t } from '../util/i18n.js';
import { initLocutionsPanel, loadLocutionsPanel, saveLocutions } from './settingsLocutions.js';
import { initPreloadPanel, loadPreloadPanel, savePreload } from './settingsPreload.js';
import { initPlaybackPanel, loadPlaybackPanel, savePlaybackPanel } from './settingsPlayback.js';
import { initKeyInputs } from '../util/keyInputs.js';
import { appAlert } from './appDialog.js';
import { initFixedPanelSettings, loadFixedPanelSettings, saveFixedPanelSettings } from './settingsFixedPanel.js';
import { loadToolbarButtons, saveToolbarButtons } from './toolbarButtons.js';

let _onSaved         = null;
let _currentOutMain  = null; // Tarjeta vigente al abrir el modal
let _currentLanguage = null; // Idioma vigente al abrir el modal
let _wired           = false;

/** Inicializa el modal de ajustes y el sistema de captura de teclas. */
export function initSettingsModal(onSaved) {
    _onSaved = onSaved;
    if (_wired) return;
    _wired = true;

    document.getElementById('btn-settings').addEventListener('click', _openSettings);
    document.getElementById('btn-save-settings').addEventListener('click', _saveSettings);
    initPlaybackPanel();
    initFixedPanelSettings();

    // Pestañas internas del modal
    document.querySelectorAll('.s-tab').forEach(tab => {
        tab.addEventListener('click', () => {
            document.querySelectorAll('.s-tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.s-panel').forEach(p => p.classList.add('hidden'));
            tab.classList.add('active');
            document.getElementById(tab.getAttribute('data-target'))?.classList.remove('hidden');
        });
    });

    initLocutionsPanel();
    initPreloadPanel();
    initKeyInputs();
    _wireDonateButton();

    // Tema: applyTheme gestiona localStorage, listener de SO y data-theme (Regla 7)
    document.getElementById('config-theme').addEventListener('change', async e => {
        const theme = e.target.value;
        applyTheme(theme);
        emit('theme-changed', { theme }).catch(console.error);
        try { await invoke('set_theme', { theme }); } catch (err) { console.error(err); }
    });
}

async function _openSettings() {
    const [devices, config, version] = await Promise.all([
        invoke('get_audio_devices').catch(() => ['default']),
        invoke('get_config').catch(() => null),
        invoke('get_app_version').catch(() => '?'),
    ]);
    document.getElementById('app-version-number').textContent = version;

    if (!config) return;
    const profile = config.profiles.find(p => p.id === config.active_profile_id);
    const outMain = profile?.audio?.out_main || 'default';
    const outPre  = profile?.audio?.out_pre  || 'default';
    _currentOutMain = outMain;

    _fillDeviceSelect('config-out-main', devices, outMain);
    _fillDeviceSelect('config-out-pre',  devices, outPre);

    // Atajos globales guardados en el perfil
    document.getElementById('config-key-stop').value = profile?.audio?.key_stop || '';
    document.getElementById('config-key-next').value = profile?.audio?.key_next || '';
    document.getElementById('config-key-prev').value = profile?.audio?.key_prev || '';
    document.getElementById('config-global-keys').checked = !!profile?.audio?.global_keys;

    document.getElementById('config-theme').value    = config.theme    || 'dark';
    document.getElementById('config-language').value = config.language || 'es';
    document.getElementById('config-button-text-size').value = config.button_text_size || 'normal';
    loadToolbarButtons(config);
    _currentLanguage = config.language || 'es';

    loadLocutionsPanel(config);
    loadPreloadPanel();
    loadPlaybackPanel(config);
    await loadFixedPanelSettings(config, devices);
    _renderOrphanedShortcuts(config);
    document.getElementById('settings-modal').classList.remove('hidden');
}

async function _saveSettings() {
    const outMain = document.getElementById('config-out-main').value;
    try {
        // Solo aplicar si la tarjeta cambió: re-aplicarla detendría el audio
        if (outMain !== _currentOutMain) {
            await invoke('set_audio_device', { deviceName: outMain });
        }
        // Salida de pre-escucha: Rust aplica el fallback (vacía o = principal → principal)
        await invoke('set_pre_device', { deviceName: document.getElementById('config-out-pre').value });
        await invoke('set_global_keys', {
            keyStop: document.getElementById('config-key-stop').value,
            keyNext: document.getElementById('config-key-next').value,
            keyPrev: document.getElementById('config-key-prev').value,
            globalKeys: document.getElementById('config-global-keys').checked,
        });
        await saveLocutions();
        await savePreload();
        await savePlaybackPanel();
        await saveFixedPanelSettings();
        await invoke('set_button_text_size', {
            size: document.getElementById('config-button-text-size').value,
        });
        await saveToolbarButtons();

        // Idioma: persistir y recargar la UI solo si cambió
        const lang = document.getElementById('config-language').value;
        if (lang !== _currentLanguage) {
            await invoke('set_language', { language: lang });
            window.location.reload();
            return;
        }
        _onSaved?.();
    } catch (e) {
        console.error('Error al guardar ajustes:', e);
        const msg = t(`errors.${e}`);
        await appAlert(msg === `errors.${e}` ? String(e) : msg);
        return;
    }
    document.getElementById('settings-modal').classList.add('hidden');
}

function _fillDeviceSelect(id, devices, current) {
    const select = document.getElementById(id);
    select.innerHTML = '';
    devices.forEach(d => {
        const opt = document.createElement('option');
        opt.value = d;
        opt.textContent = d;
        if (d === current) opt.selected = true;
        select.appendChild(opt);
    });
}

function _wireDonateButton() {
    document.getElementById('btn-donate')?.addEventListener('click', () => {
        window.__TAURI__?.opener?.openUrl('https://www.paypal.com/donate/?hosted_button_id=3JJVFFBVR4MQQ')
            .catch(console.error);
    });
}

/** Muestra atajos asignados a botones sin archivo de audio en el panel Atajos. */
function _renderOrphanedShortcuts(config) {
    const section = document.getElementById('shortcuts-orphan-section');
    if (!section) return;
    section.innerHTML = '';

    const profile = config.profiles?.find(p => p.id === config.active_profile_id);
    const orphans = [];
    for (const paleta of (profile?.paletas ?? [])) {
        for (const btn of (paleta.botones ?? [])) {
            if (btn.shortcut && !btn.path && !btn.folder) {
                orphans.push({ paletaId: paleta.id, paletaNombre: paleta.nombre, btn });
            }
        }
    }

    if (!orphans.length) {
        const p = document.createElement('p');
        p.className = 'hint';
        p.textContent = t('settings.no_orphaned');
        section.appendChild(p);
        return;
    }

    const hint = document.createElement('p');
    hint.className = 'hint danger';
    hint.textContent = t('settings.orphaned_hint');
    section.appendChild(hint);

    orphans.forEach(({ paletaId, paletaNombre, btn }) => {
        const row = document.createElement('div');
        row.className = 'orphan-row';
        const lbl = document.createElement('span');
        lbl.innerHTML = `${paletaNombre} — ${btn.label}: <code>${btn.shortcut}</code>`;
        const clrBtn = document.createElement('button');
        clrBtn.className = 'btn-xs-danger';
        clrBtn.textContent = t('settings.clear_shortcut');
        clrBtn.addEventListener('click', async () => {
            await invoke('clear_button_shortcut', { paletaId, index: btn.index });
            row.remove();
        });
        row.appendChild(lbl);
        row.appendChild(clrBtn);
        section.appendChild(row);
    });
}
