/** Presentación de escritorio. Rust valida y persiste; JS aplica CSS y su caché. */
import { invoke, listen } from '../bridge/api.js';
import { platformCapabilities } from '../bridge/platform.js';

const CACHE = 'lf-botonera-desktop-style';
const STYLES = ['modern'];
let current = { available: false, style: 'default' };

function apply(view) {
    current = view;
    const root = document.documentElement;
    if (__LF_DESKTOP_THEMES__ && view.available && STYLES.includes(view.style)) {
        root.dataset.desktopStyle = view.style;
        localStorage.setItem(CACHE, view.style);
    } else {
        delete root.dataset.desktopStyle;
        localStorage.removeItem(CACHE);
    }
    document.dispatchEvent(new CustomEvent('lf-desktop-style-change', { detail: current }));
    document.dispatchEvent(new CustomEvent('lf-theme-change', {
        detail: { theme: root.dataset.theme || 'dark' },
    }));
}

/** Caché de primer pintado; una compilación móvil nunca la consulta ni la aplica. */
export function initializeAppearanceCache() {
    if (__LF_DESKTOP_THEMES__) {
        const style = localStorage.getItem(CACHE);
        if (STYLES.includes(style)) document.documentElement.dataset.desktopStyle = style;
    }
}

/** Se ejecuta antes de mostrar cualquier ventana, también Biblioteca y pop-outs. */
export async function initializeDesktopAppearance() {
    if (!__LF_DESKTOP_THEMES__ || platformCapabilities()?.platform !== 'desktop') return;
    await listen('desktop-appearance-changed', event => apply(event.payload));
    apply(await invoke('get_desktop_appearance'));
}

export function desktopAppearance() { return current; }

export async function selectDesktopAppearance(style) {
    const view = await invoke('set_desktop_appearance', { style });
    apply(view);
}
