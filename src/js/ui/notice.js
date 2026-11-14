/**
 * Archivo: notice.js
 * Propósito: avisos breves que NO bloquean (en pleno directo un modal estorba):
 * un efecto que aún se está convirtiendo, una conversión fallida, etc.
 */
import { t } from '../util/i18n.js';

let _host = null;

export function showNotice(text, kind = 'info', ms = 4500) {
    if (!_host) {
        _host = document.createElement('div');
        _host.className = 'lf-notice-host';
        _host.setAttribute('role', 'status');
        _host.setAttribute('aria-live', 'polite');
        document.body.appendChild(_host);
    }
    const item = document.createElement('div');
    item.className = `lf-notice lf-notice-${kind}`;
    item.textContent = text;
    _host.appendChild(item);
    setTimeout(() => item.remove(), ms);
}

/** Traduce un código de error de Rust (`errors.<código>`) y lo avisa. */
export function noticeError(error) {
    const key = `errors.${error}`;
    const msg = t(key);
    showNotice(msg === key ? String(error) : msg, 'error');
}

/** Disparo desde la UI: los errores con código conocido se avisan. */
export function reportPlayError(error) {
    const code = String(error);
    if ((code.startsWith('audio_convert') || code === 'audio_converting')) {
        noticeError(code);
    } else {
        console.error(error);
    }
}
