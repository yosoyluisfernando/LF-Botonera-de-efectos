/**
 * Archivo: numberInputs.js
 * Proposito: Permite ajustar campos numericos con la rueda del raton.
 */

let _wired = false;

/** Activa el scroll sobre input[type="number"] respetando min, max y step. */
export function initNumberInputs() {
    if (_wired) return;
    _wired = true;
    document.addEventListener('wheel', _handleWheel, { passive: false });
}

function _handleWheel(e) {
    const input = e.target?.closest?.('input[type="number"]');
    if (!input || input.disabled || input.readOnly) return;

    e.preventDefault();
    const step = _numberAttr(input, 'step', 1);
    const min = _numberAttr(input, 'min', -Infinity);
    const max = _numberAttr(input, 'max', Infinity);
    const current = Number.isFinite(input.valueAsNumber)
        ? input.valueAsNumber
        : _numberAttr(input, 'value', 0);
    const delta = e.deltaY < 0 ? step : -step;
    const next = Math.min(max, Math.max(min, current + delta));

    input.value = _format(next, step);
    input.dispatchEvent(new Event('input', { bubbles: true }));
    input.dispatchEvent(new Event('change', { bubbles: true }));
}

function _numberAttr(input, attr, fallback) {
    const raw = input.getAttribute(attr);
    if (raw === null || raw === 'any') return fallback;
    const value = Number(raw);
    return Number.isFinite(value) ? value : fallback;
}

function _format(value, step) {
    if (Number.isInteger(step)) return String(Math.round(value));
    const decimals = String(step).split('.')[1]?.length ?? 0;
    return value.toFixed(decimals);
}
