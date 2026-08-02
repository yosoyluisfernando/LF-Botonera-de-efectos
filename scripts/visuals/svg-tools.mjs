import { createHash } from 'node:crypto';

export function innerSvg(source) {
  return source
    .replace(/^[\s\S]*?<svg\b[^>]*>/i, '')
    .replace(/<\/svg>\s*$/i, '')
    .trim();
}

export function tablerSymbol(slug, source) {
  return symbol(
    `tabler-${slug}`, '0 0 24 24', innerSvg(source),
    'fill="none" stroke="currentColor" stroke-width="2" '
      + 'stroke-linecap="round" stroke-linejoin="round"');
}

export function gameSymbol(slug, source) {
  let body = innerSvg(source);
  body = body.replace(/<path\s+d="M0 0h512v512H0z"\s*\/>/i, '');
  body = body.replaceAll('fill="#fff"', 'fill="currentColor"');
  body = body.replaceAll('fill="#FFF"', 'fill="currentColor"');
  body = body.replaceAll('fill="#ffffff"', 'fill="currentColor"');
  body = body.replaceAll('fill="#000"', 'fill="currentColor"');
  if (/<script|foreignObject|<image|<a\b|on\w+=|https?:/i.test(body)) {
    throw new Error(`unsafe_game_icon:${slug}`);
  }
  return symbol(`game-icons-${slug}`, '0 0 512 512', body, 'fill="currentColor"');
}

export function sprite(symbols) {
  return `<svg xmlns="http://www.w3.org/2000/svg">\n${symbols.join('\n')}\n</svg>\n`;
}

export function digest(data) {
  return createHash('sha256').update(data).digest('hex');
}

function symbol(id, viewBox, body, attributes = '') {
  return `<symbol id="${id}" viewBox="${viewBox}" ${attributes}>${body}</symbol>`;
}
