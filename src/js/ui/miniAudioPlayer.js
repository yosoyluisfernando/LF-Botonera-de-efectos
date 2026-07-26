/**
 * Control visual compartido por CUE y LIVE. Cada instancia conserva su propio
 * id, elementos y ruta de audio; compartir interfaz nunca mezcla buses.
 */
import { invoke } from '../bridge/api.js';

export function createMiniAudioPlayer(options) {
    let duration = 0;
    let origin = 0;
    let path = '';
    let active = false;
    let wired = false;

    async function open(item) {
        if (!item.path) return;
        path = item.path;
        origin = 0;
        duration = item.duration > 0 ? item.duration : 0;
        wire();
        element('name').textContent = item.name || '';
        element('volume').value = item.volume ?? 1;
        element('panel').classList.remove('hidden');
        try {
            const actual = await options.play(playArgs());
            if (Number(actual) > 0) duration = Number(actual);
            active = true;
        } catch (error) {
            element('panel').classList.add('hidden');
            throw error;
        }
    }

    function stop() {
        invoke('stop_audio', { id: options.audioId });
        active = false;
        element('panel').classList.add('hidden');
    }

    function update(payload) {
        const ticks = Array.isArray(payload) ? payload : (payload?.buttons ?? []);
        const tick = ticks.find(value => value.id === options.audioId);
        if (!tick) {
            if (active) stop();
            return;
        }
        const absolute = origin + tick.pos;
        element('progress').style.width = duration > 0
            ? `${Math.min(100, absolute / duration * 100)}%` : '0%';
        element('time').textContent = duration > 0
            ? `${format(absolute)} / ${format(duration)}` : format(absolute);
    }

    function wire() {
        if (wired) return;
        wired = true;
        element('close').addEventListener('click', stop);
        element('stop').addEventListener('click', stop);
        element('volume').addEventListener('input', event =>
            invoke('set_audio_volume', {
                id: options.audioId,
                volume: Number(event.target.value),
            }));
        element('progressBg').addEventListener('click', event => {
            if (!path || duration <= 0) return;
            const rect = event.currentTarget.getBoundingClientRect();
            origin = Math.max(0, Math.min(
                duration,
                (event.clientX - rect.left) / rect.width * duration,
            ));
            options.play(playArgs()).catch(console.error);
        });
        window.addEventListener('lf-audio-tick', event => update(event.detail));
    }

    function playArgs() {
        return {
            path,
            duration,
            position: origin,
            volume: Number(element('volume').value),
        };
    }

    function element(key) {
        return document.getElementById(options.elements[key]);
    }

    return { open, stop, update };
}

function format(seconds) {
    const minutes = Math.floor(seconds / 60);
    const remainder = Math.floor(seconds % 60);
    return `${String(minutes).padStart(2, '0')}:${String(remainder).padStart(2, '0')}`;
}
