/** Balística visual común: ataque inmediato y caída apenas amortiguada. */
const ATTACK_MS = 20;
const RELEASE_MS = 50;

export function activeMeterTransition(element, level) {
    const normalized = Math.min(Math.max(Number(level) || 0, 0), 1);
    const previous = Number(element.dataset.meterLevel);
    element.dataset.meterLevel = String(normalized);
    return Number.isFinite(previous) && normalized < previous
        ? RELEASE_MS : ATTACK_MS;
}
