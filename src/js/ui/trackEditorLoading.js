/**
 * Estado visual transitorio del editor. No decide si el análisis terminó:
 * esa única verdad es la respuesta de `analyze_track`.
 */
const INTERACTIVE_IDS = [
    'te-fijar-inicio', 'te-play-inicio', 'te-fijar-fin', 'te-clear-end',
    'te-gain', 'te-norm-enabled', 'te-normalize', 'te-norm-settings',
    'te-play', 'te-stop', 'te-zoom', 'te-popout', 'te-save',
];

export function begin(wave, zoom, text) {
    setInteractive(false);
    document.getElementById('te-wave-inner').style.pointerEvents = 'none';
    document.getElementById('te-gain').value = 0;
    document.getElementById('te-gain-readout').textContent = '0.0 dB';
    document.getElementById('te-norm-enabled').checked = false;
    for (const id of ['te-cue-start', 'te-cue-end', 'te-duration', 'te-lufs', 'te-peak']) {
        document.getElementById(id).textContent = '—';
    }
    wave.setZoom(zoom);
    wave.setData({ duration: 0, peaks: [] });
    wave.setMarkers(0, null);
    wave.setGain(1);
    show(text);
}

export function progress(text) { show(text); }

export function complete() {
    setInteractive(true);
    document.getElementById('te-wave-inner').style.pointerEvents = '';
    show(null);
}

export function failed(text) {
    document.getElementById('te-wave-inner').style.pointerEvents = 'none';
    show(text);
}

export function dismiss() { show(null); }

function setInteractive(enabled) {
    for (const id of INTERACTIVE_IDS) document.getElementById(id).disabled = !enabled;
}

function show(text) {
    const status = document.getElementById('te-status');
    if (!text) return status.classList.add('hidden');
    status.textContent = text;
    status.classList.remove('hidden');
}
