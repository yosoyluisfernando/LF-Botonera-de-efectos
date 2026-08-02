import { typeIcon } from '../util/typeIcons.js';
import { createSvgVisual } from './visualSvg.js';

const DEFAULT = Object.freeze({ kind: 'auto', value: '', mode: 'visual_text' });
const KINDS = new Set(['auto', 'emoji', 'basic']);
const MODES = new Set(['text', 'visual_text', 'visual']);

export function normalizedVisual(button) {
    const source = button?.visual ?? DEFAULT;
    const kind = KINDS.has(source.kind) ? source.kind : DEFAULT.kind;
    return {
        kind,
        value: kind === 'auto' ? '' : (source.value || ''),
        mode: MODES.has(source.mode) ? source.mode : DEFAULT.mode,
    };
}

export function paintButtonContent(host, button, textClass = '') {
    const visual = normalizedVisual(button);
    const name = button?.name || button?.label || '';
    host.replaceChildren();
    host.classList.toggle('visual-only', visual.mode === 'visual');
    host.classList.toggle('has-visual', visual.mode !== 'text');
    host.classList.toggle(
        'visual-custom',
        visual.kind === 'emoji' || visual.kind === 'basic',
    );

    if (visual.mode !== 'text') {
        const node = visualNode(visual, button?.type_icon);
        if (node) host.appendChild(node);
    }
    if (visual.mode !== 'visual') {
        const text = document.createElement('span');
        text.className = `button-visual-text ${textClass}`.trim();
        text.textContent = name;
        host.appendChild(text);
    }
    host.closest('.grid-item, .fixed-panel-item, .player-row')?.setAttribute('aria-label', name);
}

function visualNode(visual, automaticIcon) {
    const svg = createSvgVisual(visual.kind, visual.value);
    if (svg) {
        svg.classList.add('button-visual');
        return svg;
    }
    const span = document.createElement('span');
    span.setAttribute('aria-hidden', 'true');
    if (visual.kind === 'emoji') {
        span.className = 'button-visual button-visual-emoji';
        span.textContent = visual.value;
        return span;
    }
    if (visual.kind === 'basic') {
        span.className = 'button-visual material-symbols-rounded';
        span.textContent = visual.value;
        return span;
    }
    if (!automaticIcon) return null;
    span.className = 'button-visual button-visual-auto';
    span.innerHTML = typeIcon(automaticIcon);
    span.querySelectorAll('*').forEach(node => node.setAttribute('aria-hidden', 'true'));
    return span;
}
