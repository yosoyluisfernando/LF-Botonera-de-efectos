import { invoke } from '../bridge/api.js';

const MODES = ['normal', 'loop', 'overlap', 'restart'];

export function createPlaybackModeController(options) {
    let mode = 'normal';
    let solo = false;
    let wired = false;

    async function refresh() {
        const state = await invoke(options.getState);
        mode = state?.mode || 'normal'; solo = !!state?.solo; paint();
    }

    function init() {
        if (wired) return refresh();
        wired = true;
        MODES.forEach(value => options.element(value)?.addEventListener('click', () => {
            mode = value; paint(); invoke(options.setMode, { mode: value }).catch(console.error);
        }));
        options.soloElement()?.addEventListener('click', () => {
            solo = !solo; paint(); invoke(options.setSolo, { enabled: solo }).catch(console.error);
        });
        options.stopElement()?.addEventListener('click', () => invoke(options.stop).catch(console.error));
        return refresh();
    }

    function paint() {
        MODES.forEach(value => options.element(value)?.classList.toggle('active', value === mode));
        options.soloElement()?.classList.toggle('active', solo);
    }

    return { init, refresh, getMode: () => mode };
}
