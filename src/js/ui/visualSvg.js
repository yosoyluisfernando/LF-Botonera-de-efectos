const SVG_NS = 'http://www.w3.org/2000/svg';
const VALUE_PATTERN = /^(tabler|game-icons):([a-z-]+):([a-z0-9-]+)$/;

export function createSvgVisual(kind, value, extraClass = '') {
    const parsed = parseSvgValue(kind, value);
    if (!parsed) return null;
    const svg = document.createElementNS(SVG_NS, 'svg');
    const use = document.createElementNS(SVG_NS, 'use');
    use.setAttribute(
        'href',
        `/visuals/basic/${parsed.group}.svg#${parsed.pack}-${parsed.slug}`,
    );
    use.setAttribute('width', '24');
    use.setAttribute('height', '24');
    svg.classList.add('button-visual-svg', `button-visual-${kind}`);
    if (extraClass) svg.classList.add(extraClass);
    svg.setAttribute('viewBox', '0 0 24 24');
    svg.setAttribute('aria-hidden', 'true');
    svg.appendChild(use);
    return svg;
}

export function isSvgVisual(kind, value) {
    return parseSvgValue(kind, value) !== null;
}

function parseSvgValue(kind, value) {
    const match = VALUE_PATTERN.exec(value);
    if (!match) return null;
    const [, pack, group, slug] = match;
    if (kind !== 'basic') return null;
    return { pack, group, slug };
}
