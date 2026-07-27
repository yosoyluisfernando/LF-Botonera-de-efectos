/** Escucha previa CUE sobre el control visual compartido. */
import { invoke } from '../bridge/api.js';
import { createMiniAudioPlayer } from './miniAudioPlayer.js';

const PRELISTEN_ID = '__prelisten__';
const player = createMiniAudioPlayer({
    audioId: PRELISTEN_ID,
    elements: {
        panel: 'prelisten-player',
        close: 'close-prelisten',
        stop: 'btn-stop-prelisten',
        name: 'prelisten-name',
        volume: 'prelisten-volume',
        progressBg: 'prelisten-progress-bg',
        progress: 'prelisten-progress',
        time: 'prelisten-time',
    },
    play: ({ path, duration, position, volume }) => invoke('play_audio', {
        id: PRELISTEN_ID,
        path,
        volume,
        duration,
        loopMode: false,
        stopOther: false,
        overlap: false,
        restart: true,
        cueStartS: position,
    }),
});

export function openPrelisten(path, name, vol, duration) {
    return player.open({ path, name, volume: vol, duration });
}

export const stopPrelisten = player.stop;
export const updatePrelistenTick = player.update;
