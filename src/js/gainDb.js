/**
 * Archivo: gainDb.js
 * Proposito: conversion pequena entre dB visibles y multiplicador lineal que
 * conserva el formato historico `vol` de los botones.
 */

export const MIN_GAIN_DB = -24;
export const MAX_GAIN_DB = 24;

/** Limita un valor de dB al rango usado por los controles de volumen. */
export function clampGainDb(db) {
    const n = Number.parseFloat(db);
    if (!Number.isFinite(n)) return 0;
    return Math.max(MIN_GAIN_DB, Math.min(MAX_GAIN_DB, n));
}

/** Convierte dB a multiplicador lineal para el motor de audio. */
export function dbToLinear(db) {
    return Math.pow(10, clampGainDb(db) / 20);
}

/** Convierte el multiplicador `vol` legado a dB para mostrarlo en la UI. */
export function linearToDb(vol) {
    const n = Number.parseFloat(vol);
    if (!Number.isFinite(n) || n <= 0) return MIN_GAIN_DB;
    return clampGainDb(20 * Math.log10(n));
}

/** Texto uniforme para lecturas de controles en dB. */
export function formatGainDb(db) {
    return `${clampGainDb(db).toFixed(1)} dB`;
}
