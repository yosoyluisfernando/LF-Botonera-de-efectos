/** Gestos visuales: conserva el arrastre y confirma el valor desde Rust al terminar. */
import { refreshVolumeControls } from './volumeSync.js';
const editing = new WeakSet();
export function paintVolumeSlider(slider, value, max) {
    if (!slider) return;
    slider.max = String(max);
    if (!editing.has(slider)) slider.value = String(value);
}
export function bindVolumeSlider(slider, send) {
    let pending;
    let sending = false;
    async function drain() {
        if (sending) return;
        sending = true;
        while (pending) {
            const request = pending; pending = undefined;
            try { await send(request.value, request.persist); }
            catch (error) { console.error(error); }
            if (request.persist) refreshVolumeControls();
        }
        sending = false;
    }
    function queue(persist) {
        pending = { value: slider.value, persist: persist || !!pending?.persist };
        drain();
    }
    function finish() {
        editing.delete(slider);
        refreshVolumeControls();
    }
    slider.addEventListener('pointerdown', e => {
        editing.add(slider);
        if (e.isTrusted) slider.setPointerCapture?.(e.pointerId);
    });
    for (const name of ['pointerup', 'pointercancel', 'lostpointercapture', 'blur']) slider.addEventListener(name, finish);
    slider.addEventListener('keydown', e => {
        if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End', 'PageUp', 'PageDown'].includes(e.key)) editing.add(slider);
    });
    slider.addEventListener('keyup', finish);
    slider.addEventListener('input', () => queue(false));
    slider.addEventListener('change', () => { editing.delete(slider); queue(true); });
}
