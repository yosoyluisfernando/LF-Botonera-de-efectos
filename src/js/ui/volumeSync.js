/** Distribuye el mismo estado Rust a todas las vistas, también en reposo. */
import { invoke, listen } from '../bridge/api.js';
const views = new Map();
let latest;
let starting;
let reading;
let readAgain = false;
let eventRevision = 0;
export function registerVolumeView(name, paint) {
    views.set(name, paint);
    if (latest) paint(latest);
}
function apply(state) {
    if (!state) return;
    latest = state;
    for (const paint of views.values()) paint(state);
}
export function initVolumeSync() {
    starting ??= (async () => {
        await listen('volume-controls-changed', e => { eventRevision++; apply(e.payload); });
        await refreshVolumeControls();
    })().catch(error => { starting = undefined; throw error; });
    return starting;
}
export function refreshVolumeControls() {
    if (reading) { readAgain = true; return reading; }
    const observed = eventRevision;
    reading = invoke('get_volume_controls').then(state => { if (observed === eventRevision) apply(state); }).catch(console.error).finally(() => {
        reading = undefined;
        if (readAgain) { readAgain = false; refreshVolumeControls(); }
    });
    return reading;
}
