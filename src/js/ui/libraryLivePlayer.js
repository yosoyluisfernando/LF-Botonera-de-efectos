/** Reproductor compacto LIVE: misma interfaz que CUE, bus Programa independiente. */
import { invoke } from '../bridge/api.js';
import { createMiniAudioPlayer } from './miniAudioPlayer.js';

const LIVE_ID = '__library_live__';
const player = createMiniAudioPlayer({
    audioId: LIVE_ID,
    elements: {
        panel: 'live-player',
        close: 'close-live',
        stop: 'btn-stop-live',
        name: 'live-name',
        volume: 'live-volume',
        progressBg: 'live-progress-bg',
        progress: 'live-progress',
        time: 'live-time',
    },
    play: ({ path, duration, position, volume }) => invoke('library_play_live', {
        path,
        durationS: duration,
        positionS: position,
        volume,
    }),
});

export function openLivePlayer(track) {
    return player.open({
        path: track.path,
        name: track.title || track.file_name,
        volume: 1,
        duration: track.duration_s,
    });
}
