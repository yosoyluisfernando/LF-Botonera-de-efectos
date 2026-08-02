import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { localizedEntry } from './locale.mjs';
import { digest, sprite } from './svg-tools.mjs';

export async function writeCatalog(options) {
  const { root, kind, items, locales, localeData, sources, licenses } = options;
  const catalogFolder = path.join(root, 'src-tauri', 'resources', kind);
  const spriteFolder = path.join(root, 'src', 'public', 'visuals', kind);
  await mkdir(catalogFolder, { recursive: true });
  await mkdir(spriteFolder, { recursive: true });
  const catalogs = {};
  for (const locale of locales) {
    const rows = items.map(item => localizedEntry(item, locale, localeData));
    const header = JSON.stringify({ format: 2, locale, count: rows.length });
    const body = `${[header, ...rows.map(row => JSON.stringify(row))].join('\n')}\n`;
    const file = `${locale}.ndjson`;
    await writeFile(path.join(catalogFolder, file), body);
    catalogs[file] = digest(body);
  }
  const sprites = {};
  for (const [group, groupItems] of grouped(items)) {
    const symbols = groupItems.map(item => item.symbol).filter(Boolean);
    if (!symbols.length) continue;
    const body = sprite(symbols);
    const file = `${group}.svg`;
    await writeFile(path.join(spriteFolder, file), body);
    sprites[file] = {
      count: symbols.length,
      bytes: Buffer.byteLength(body),
      sha256: digest(body),
    };
  }
  for (const license of licenses) {
    const body = await readFile(license.source);
    await writeFile(path.join(catalogFolder, license.file), body);
  }
  const manifest = {
    format: 2,
    kind,
    count: items.length,
    sources,
    catalogs,
    sprites,
  };
  await writeFile(
    path.join(catalogFolder, 'manifest.json'),
    `${JSON.stringify(manifest, null, 2)}\n`);
  return manifest;
}

function grouped(items) {
  const groups = new Map();
  for (const item of items) {
    if (!groups.has(item.group)) groups.set(item.group, []);
    groups.get(item.group).push(item);
  }
  return groups;
}
