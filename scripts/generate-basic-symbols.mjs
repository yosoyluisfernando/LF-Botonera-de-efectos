import { createHash } from 'node:crypto';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';

const root = path.resolve(import.meta.dirname, '..');
const input = JSON.parse(await readFile(path.join(root, 'scripts/basic-symbols.json'), 'utf8'));
const output = path.join(root, 'src-tauri/resources/basic');
const fontOutput = path.join(root, 'src/public/fonts/material-symbols-rounded-basic.woff2');
const locales = ['es', 'en', 'pt-BR', 'pt-PT'];
const commit = input.sourceCommit;
const rawBase = `https://raw.githubusercontent.com/google/material-design-icons/${commit}`;
const codepointsUrl = `${rawBase}/variablefont/MaterialSymbolsRounded%5BFILL%2CGRAD%2Copsz%2Cwght%5D.codepoints`;
const licenseUrl = `${rawBase}/LICENSE`;

if (input.format !== 1 || input.symbols.length !== 24) throw new Error('basic_manifest_invalid');
const ids = input.symbols.map(item => item.id);
const sorted = [...ids].sort();
if (new Set(ids).size !== ids.length || ids.some((id, i) => id !== sorted[i])) {
  throw new Error('basic_ids_must_be_unique_and_sorted');
}

const codepoints = await fetchText(codepointsUrl);
const available = new Set(codepoints.trim().split(/\r?\n/).map(line => line.split(' ')[0]));
const missing = ids.filter(id => !available.has(id));
if (missing.length) throw new Error(`unknown_material_symbols:${missing.join(',')}`);

const cssUrl = 'https://fonts.googleapis.com/css2?family=Material+Symbols+Rounded'
  + ':FILL,GRAD,opsz,wght@1,0,48,500'
  + `&icon_names=${ids.join(',')}&display=block`;
const css = await fetchText(cssUrl);
const fontUrl = css.match(/src: url\((https:\/\/[^)]+)\)/)?.[1];
if (!fontUrl) throw new Error('material_font_url_missing');
const font = new Uint8Array(await fetchBytes(fontUrl));
const license = await fetchText(licenseUrl);

await mkdir(output, { recursive: true });
await mkdir(path.dirname(fontOutput), { recursive: true });
await writeFile(fontOutput, font);
await writeFile(path.join(output, 'MATERIAL_SYMBOLS_LICENSE.txt'), license);

const files = {};
for (const locale of locales) {
  const header = JSON.stringify({ format: 1, locale, count: ids.length });
  const rows = input.symbols.map(item => {
    const [name, keywordText] = item[locale];
    return JSON.stringify({
      value: item.id,
      group: item.group,
      subgroup: '',
      name,
      keywords: keywordText.split(' '),
    });
  });
  const body = `${[header, ...rows].join('\n')}\n`;
  const file = `${locale}.ndjson`;
  await writeFile(path.join(output, file), body);
  files[file] = digest(Buffer.from(body));
}
const manifest = {
  format: 1,
  sourceCommit: commit,
  source: 'Google Material Symbols Rounded',
  count: ids.length,
  fontFile: path.basename(fontOutput),
  fontBytes: font.byteLength,
  fontSha256: digest(font),
  catalogs: files,
};
await writeFile(path.join(output, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`Generados ${ids.length} símbolos básicos; fuente ${font.byteLength} bytes.`);
console.log(`SHA-256: ${manifest.fontSha256}`);

async function fetchText(url) {
  return new TextDecoder().decode(await fetchBytes(url));
}

async function fetchBytes(url) {
  const response = await fetch(url, { headers: { 'User-Agent': 'LF-Botonera-build' } });
  if (!response.ok) throw new Error(`${response.status}:${url}`);
  return response.arrayBuffer();
}

function digest(data) {
  return createHash('sha256').update(data).digest('hex');
}
