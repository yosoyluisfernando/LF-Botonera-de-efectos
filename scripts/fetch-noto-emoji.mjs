import { createHash } from 'node:crypto';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';

const commit = '8998f5dd683424a73e2314a8c1f1e359c19e8742';
const root = path.resolve(import.meta.dirname, '..');
const raw = `https://raw.githubusercontent.com/googlefonts/noto-emoji/${commit}`;
const font = new Uint8Array(await fetchBytes(`${raw}/fonts/Noto-COLRv1.ttf`));
const license = new Uint8Array(await fetchBytes(`${raw}/LICENSE`));
const fontPath = path.join(root, 'src/public/fonts/Noto-COLRv1.ttf');
const licensePath = path.join(root, 'src-tauri/resources/emoji/NOTO_EMOJI_LICENSE.txt');

if (font.byteLength !== 4_991_984) throw new Error(`unexpected_noto_size:${font.byteLength}`);
const sha256 = digest(font);
if (sha256 !== '0ae57fe58645638523ba35f388d93739d292539a9acb84df5700c81b1e1a28d2') {
  throw new Error(`unexpected_noto_hash:${sha256}`);
}
await mkdir(path.dirname(fontPath), { recursive: true });
await mkdir(path.dirname(licensePath), { recursive: true });
await writeFile(fontPath, font);
await writeFile(licensePath, license);
console.log(`Noto Emoji COLRv1: ${font.byteLength} bytes, ${sha256}`);

async function fetchBytes(url) {
  const response = await fetch(url, { headers: { 'User-Agent': 'LF-Botonera-build' } });
  if (!response.ok) throw new Error(`${response.status}:${url}`);
  return response.arrayBuffer();
}

function digest(data) {
  return createHash('sha256').update(data).digest('hex');
}
