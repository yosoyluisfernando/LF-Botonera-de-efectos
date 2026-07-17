/**
 * Archivo: toolbarButtons.js
 * Propósito: qué botones opcionales de la barra superior se enseñan. Carga las
 * casillas de Ajustes, las guarda y aplica el resultado a la barra.
 * Solo dibuja (Regla 4): la preferencia la guarda Rust.
 */
import { invoke } from '../bridge/api.js';

/**
 * Los que se pueden esconder: el botón de la barra, su casilla en Ajustes y el
 * nombre que entiende el IPC.
 *
 * Solo estos dos. El resto de la barra no es opcional: sin el "+" no se pueden
 * crear pestañas, sin 👤 no se cambia de perfil y sin ⚙️ no hay forma de volver
 * aquí a recuperar lo que se escondió.
 */
const BUTTONS = [
    { bus: 'console', btn: 'btn-console', box: 'config-show-console', flag: 'show_console_button' },
    { bus: 'fixed_panel', btn: 'btn-fixed-panel', box: 'config-show-fixed-panel', flag: 'show_fixed_panel_button' },
];

/** Aplica la configuración a la barra. Se llama al arrancar y al guardar. */
export function applyToolbarButtons(config) {
    for (const b of BUTTONS) _toggle(b.btn, config[b.flag]);
}

/** Pone las casillas de Ajustes como está la configuración. */
export function loadToolbarButtons(config) {
    // `?? true`: una configuración anterior a estos ajustes no trae el campo, y
    // ausente significa visible — no debe hacer desaparecer nada.
    for (const b of BUTTONS) document.getElementById(b.box).checked = config[b.flag] ?? true;
}

/** Guarda y aplica en el acto: hacer reiniciar la aplicación para esconder un
 *  icono sería absurdo. */
export async function saveToolbarButtons() {
    const estado = {};
    for (const b of BUTTONS) {
        const visible = document.getElementById(b.box).checked;
        estado[b.flag] = visible;
        await invoke('set_toolbar_button', { button: b.bus, visible });
    }
    applyToolbarButtons(estado);
}

/** `=== false` y no un booleano a secas: ausente significa visible. */
function _toggle(id, visible) {
    document.getElementById(id)?.classList.toggle('hidden', visible === false);
}
