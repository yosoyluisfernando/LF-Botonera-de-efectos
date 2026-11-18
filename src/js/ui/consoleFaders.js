/** Faders de consola: representan los controles Rust, sin una ganancia adicional. */
import { invoke } from '../bridge/api.js';
import { bindVolumeSlider, paintVolumeSlider } from './volumeSlider.js';
function paintFeedback(input) {
    const track = input.closest('.fader-track');
    const percent = `${Math.round(Number(input.value) * 100)}%`;
    input.setAttribute('aria-valuetext', percent);
    input.title = percent;
    track.querySelector('.fader-value').textContent = percent;
    track.style.setProperty('--fader-position', String(1 - Number(input.value) / Number(input.max)));
    track.style.setProperty('--fader-unity', String(1 - 1 / Number(input.max)));
    track.classList.toggle('has-boost', Number(input.max) > 1);
}
export function wireConsoleFader(input, bus, device) {
    const track = input.closest('.fader-track');
    const value = document.createElement('span');
    value.className = 'fader-value';
    value.setAttribute('aria-hidden', 'true');
    const unity = document.createElement('span');
    unity.className = 'fader-unity';
    unity.textContent = '100%';
    unity.setAttribute('aria-hidden', 'true');
    track.append(value, unity);
    let timer;
    function show() {
        clearTimeout(timer);
        track.classList.add('show-value');
        paintFeedback(input);
    }
    function hide() {
        clearTimeout(timer);
        timer = setTimeout(() => track.classList.remove('show-value'), 400);
    }
    bindVolumeSlider(input, (value, persist) => invoke('set_bus_fader', { bus, device, value: Number(value), persist }));
    for (const name of ['pointerdown', 'input']) input.addEventListener(name, show);
    input.addEventListener('keydown', e => {
        if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End', 'PageUp', 'PageDown'].includes(e.key)) show();
    });
    for (const name of ['pointerup', 'pointercancel', 'lostpointercapture', 'keyup', 'change', 'blur']) input.addEventListener(name, hide);
    paintFeedback(input);
}
export function paintConsoleFaders(strips, state) {
    for (const [bus, el] of strips) {
        const fader = state.faders?.[bus];
        if (!fader) continue;
        const input = el.querySelector('.fader');
        paintVolumeSlider(input, fader.value, fader.max);
        // Durante un gesto se representa el valor local que conserva volumeSlider.
        paintFeedback(input);
    }
}
