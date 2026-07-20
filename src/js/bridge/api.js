/**
 * Archivo: api.js
 * Propósito: Wrapper seguro sobre las APIs globales de Tauri (window.__TAURI__).
 * Cumple Regla 4: Punto central que aísla la comunicación IPC del resto de módulos.
 *
 * CRÍTICO: window.__TAURI__ se resuelve en el CUERPO de cada función, nunca en tiempo
 * de módulo. En producción (WebView2), el objeto global puede no estar inyectado todavía
 * cuando el módulo se parsea por primera vez. Capturarlo al nivel del módulo lo congela
 * como undefined permanentemente y ningún evento ni comando IPC llega jamás.
 */

/**
 * Envía un comando IPC al motor Rust.
 * En navegador local permite datos demo; en Tauri real Rust es obligatorio.
 * @param {string} cmd  Nombre del comando Tauri/Rust.
 * @param {object} [args] Argumentos opcionales.
 * @returns {Promise<any>}
 */
export function invoke(cmd, args) {
    if (window.__TAURI__?.core?.invoke)
        return window.__TAURI__.core.invoke(cmd, args);

    if (!isBrowserFallback()) {
        return Promise.reject(new Error(`TAURI_IPC_NOT_READY:${cmd}`));
    }

    console.warn(`[Sin Backend Tauri] invoke('${cmd}') — usando datos de fallback.`);
    if (cmd === 'get_config') {
        return Promise.resolve({
            is_first_boot: false,
            theme: 'dark',
            language: 'es',
            button_text_size: 'normal',
            weather_module_enabled: false,
            lf_automatizador_link: false,
            active_profile_id: 'demo_profile',
            profiles: [{
                id: 'demo_profile',
                name: 'Demo',
                bg: '#008c3a',
                text: '#ffffff',
                active_paleta_id: 'demo_tab',
                audio: { out_main: 'default', out_pre: 'default', key_stop: '', key_next: '', key_prev: '' },
                paletas: [{
                    id: 'demo_tab', nombre: 'BOTONERA 1', rows: 5, cols: 5,
                    audio_out: '', shortcut: '', tab_bg: '#3a3f44',
                    tab_text: '#ffffff', botones: [],
                }],
            }],
        });
    }
    if (cmd === 'get_grid_state') {
        return Promise.resolve({ columns: 5, rows: 5, buttons: [] });
    }
    if (cmd === 'get_audio_devices') return Promise.resolve(['default']);
    if (cmd === 'get_app_version') return Promise.resolve('dev');
    if (cmd === 'set_button_text_size') return Promise.resolve(null);
    if (cmd === 'undo_config') return invoke('get_config');
    if (cmd === 'redo_config') return invoke('get_config');
    if (cmd === 'export_tab_by_id') return Promise.resolve(null);
    if (cmd === 'export_profile_by_id') return Promise.resolve(null);
    if (cmd === 'get_color_palette') {
        const colors = [
            '#E53935', '#D81B60', '#8E24AA', '#5E35B1', '#3949AB', '#1E88E5', '#039BE5', '#00ACC1',
            '#00897B', '#43A047', '#7CB342', '#C0CA33', '#FDD835', '#FFB300', '#FB8C00', '#F4511E',
            '#6D4C41', '#546E7A', '#C62828', '#AD1457', '#6A1B9A', '#4527A0', '#283593', '#1565C0',
            '#0277BD', '#00838F', '#00695C', '#2E7D32', '#558B2F', '#9E9D24', '#EF6C00', '#D84315',
        ];
        return Promise.resolve(colors.map(c => ({
            base: c, darkBg: c, darkText: '#FFFFFF', lightBg: c, lightText: '#FFFFFF',
        })));
    }
    if (cmd === 'get_playback_mode') return Promise.resolve('normal');
    if (cmd === 'get_playback_state') return Promise.resolve({ mode: 'normal', solo: false });
    if (cmd === 'set_solo_mode') return Promise.resolve(null);
    if (cmd === 'get_master_volume_state') {
        return Promise.resolve({ volume: 1, remember: false, boost: false, max: 1 });
    }
    if (cmd === 'set_master_volume') {
        const volume = Math.max(0, Math.min(Number(args?.volume ?? 1), 1));
        return Promise.resolve({ volume, remember: false, boost: false, max: 1 });
    }
    if (cmd === 'set_master_volume_options') {
        const boost = !!args?.boost;
        return Promise.resolve({ volume: 1, remember: !!args?.remember, boost, max: boost ? 1.5 : 1 });
    }
    if (cmd === 'check_for_updates') {
        return Promise.resolve({
            checked: true,
            storeManaged: false,
            updateAvailable: false,
            currentVersion: 'dev',
            latestVersion: 'dev',
            releaseUrl: '',
            notes: '',
        });
    }
    return Promise.resolve(null);
}

/**
 * Espera a que window.__TAURI__ esté disponible.
 * En navegador local acepta fallback; en Tauri real falla para no inventar estado.
 * Debe llamarse al inicio del arranque antes de cualquier invoke() o listen().
 * @param {number} [timeoutMs=5000]
 * @returns {Promise<void>}
 */
export function waitForTauri(timeoutMs = 5000) {
    if (_hasTauriBridge()) return Promise.resolve();
    return new Promise((resolve, reject) => {
        const start = Date.now();
        const id = setInterval(() => {
            if (_hasTauriBridge()) {
                clearInterval(id);
                resolve();
            } else if (Date.now() - start > timeoutMs) {
                clearInterval(id);
                if (isBrowserFallback()) resolve();
                else reject(new Error('TAURI_BRIDGE_TIMEOUT'));
            }
        }, 50);
    });
}

/** Devuelve true solo para navegador/Vite sin backend, nunca para app empaquetada. */
export function isBrowserFallback() {
    const host = window.location.hostname;
    return host === 'localhost' || host === '127.0.0.1' || host === '::1';
}

function _hasTauriBridge() {
    return !!(window.__TAURI__?.core?.invoke && window.__TAURI__?.event?.listen);
}

/**
 * Suscribe un handler a un evento emitido por el motor Rust.
 * @param {string}   event   Nombre del evento.
 * @param {Function} handler Callback que recibe el evento.
 * @returns {Promise<Function>} Función para cancelar la suscripción.
 */
export function listen(event, handler) {
    if (window.__TAURI__?.event?.listen)
        return window.__TAURI__.event.listen(event, handler);
    if (!isBrowserFallback()) {
        return Promise.reject(new Error(`TAURI_EVENT_NOT_READY:${event}`));
    }
    console.warn(`[Sin Backend Tauri] listen('${event}') — evento ignorado.`);
    return Promise.resolve(() => {});
}

/** Emite un evento Tauri para sincronizar ventanas del frontend. */
export function emit(event, payload) {
    if (window.__TAURI__?.event?.emit)
        return window.__TAURI__.event.emit(event, payload);
    if (!isBrowserFallback()) {
        return Promise.reject(new Error(`TAURI_EVENT_NOT_READY:${event}`));
    }
    return Promise.resolve(null);
}
