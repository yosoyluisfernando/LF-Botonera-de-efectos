import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { digest } from './visuals/svg-tools.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const locales = ['es', 'en', 'pt-BR', 'pt-PT'];
await verifyGameIconTags();
await verifyKind('basic', 8_000);
console.log('Catálogos visuales, hashes y sprites verificados.');

async function verifyKind(kind, minimum) {
  const resourceFolder = path.join(root, 'src-tauri', 'resources', kind);
  const spriteFolder = path.join(root, 'src', 'public', 'visuals', kind);
  const manifest = JSON.parse(
    await readFile(path.join(resourceFolder, 'manifest.json'), 'utf8'));
  assert(manifest.format === 2 && manifest.kind === kind, `${kind}_manifest`);
  assert(manifest.count >= minimum, `${kind}_unexpected_count:${manifest.count}`);
  const sourceTotal = Object.values(manifest.sources)
    .reduce((sum, source) => sum + source.count, 0);
  assert(sourceTotal === manifest.count, `${kind}_source_count`);
  const sprites = await readSprites(spriteFolder);
  let canonicalValues;
  let canonicalEntries;
  for (const locale of locales) {
    const file = `${locale}.ndjson`;
    const body = await readFile(path.join(resourceFolder, file), 'utf8');
    assert(digest(body) === manifest.catalogs[file], `${kind}_${locale}_hash`);
    const [headerLine, ...entryLines] = body.trimEnd().split(/\r?\n/);
    const header = JSON.parse(headerLine);
    const entries = entryLines.map(line => JSON.parse(line));
    assert(header.format === 2 && header.locale === locale, `${kind}_${locale}_header`);
    assert(header.count === manifest.count && entries.length === manifest.count,
      `${kind}_${locale}_count`);
    const values = entries.map(entry => entry.value);
    assert(new Set(values).size === values.length, `${kind}_${locale}_duplicates`);
    if (canonicalValues) {
      assert(values.every((value, index) => value === canonicalValues[index]),
        `${kind}_${locale}_order`);
    } else {
      canonicalValues = values;
      canonicalEntries = entries;
    }
    entries.forEach(entry => verifyEntry(kind, entry, sprites));
  }
  const files = (await readdir(spriteFolder)).filter(file => file.endsWith('.svg'));
  assert(files.length === Object.keys(manifest.sprites).length, `${kind}_sprite_files`);
  for (const file of files) {
    const body = await readFile(path.join(spriteFolder, file), 'utf8');
    const metadata = manifest.sprites[file];
    assert(digest(body) === metadata.sha256, `${kind}_${file}_hash`);
    assert(Buffer.byteLength(body) === metadata.bytes, `${kind}_${file}_bytes`);
    assert(sprites.get(file.slice(0, -4)).size === metadata.count,
      `${kind}_${file}_symbols`);
    assert(metadata.bytes < 1_250_000, `${kind}_${file}_too_large`);
    assert(!/<script|foreignObject|<image|<a\b|on\w+=/i.test(body),
      `${kind}_${file}_unsafe`);
  }
  const spriteTotal = Object.values(manifest.sprites)
    .reduce((sum, sprite) => sum + sprite.count, 0);
  assert(spriteTotal === canonicalEntries.filter(entry => entry.value.includes(':')).length,
    `${kind}_sprite_total`);
  verifyUsefulGroups(kind, canonicalEntries);
}

async function readSprites(folder) {
  const groups = new Map();
  for (const file of (await readdir(folder)).filter(name => name.endsWith('.svg'))) {
    const body = await readFile(path.join(folder, file), 'utf8');
    const ids = new Set([...body.matchAll(/<symbol id="([^"]+)"/g)]
      .map(match => match[1]));
    groups.set(file.slice(0, -4), ids);
  }
  return groups;
}

function verifyEntry(kind, entry, sprites) {
  assert(entry.name.trim() && entry.keywords.length, `${kind}_empty_text`);
  const parts = entry.value.split(':');
  if (parts.length === 1) {
    assert(kind === 'basic' && entry.subgroup === 'material', `legacy_${entry.value}`);
    return;
  }
  assert(parts.length === 3, `${kind}_value:${entry.value}`);
  const [pack, group, slug] = parts;
  assert(group === entry.group && pack === entry.subgroup, `${kind}_namespace`);
  assert(sprites.get(group)?.has(`${pack}-${slug}`), `${kind}_symbol:${entry.value}`);
  assert(!(pack === 'tabler' && slug.endsWith('-off')), `tabler_off:${slug}`);
}

function verifyUsefulGroups(kind, entries) {
  const counts = entries.reduce((result, entry) => {
    result[entry.group] = (result[entry.group] || 0) + 1;
    return result;
  }, {});
  const minimums = { audio: 200, animals: 400, nature: 500, objects: 350, office: 400 };
  for (const [group, minimum] of Object.entries(minimums)) {
    assert(counts[group] >= minimum, `${kind}_${group}_coverage:${counts[group]}`);
  }
}

async function verifyGameIconTags() {
  const rules = JSON.parse(
    await readFile(path.join(root, 'scripts', 'game-icon-groups.json'), 'utf8'));
  const snapshot = JSON.parse(
    await readFile(path.join(root, 'scripts', 'game-icon-tags.json'), 'utf8'));
  const official = Object.keys(snapshot.tags);
  const mapped = rules.flatMap(rule => rule.tags);
  assert(new Set(mapped).size === mapped.length, 'game_icon_duplicate_tag');
  assert(official.every(tag => mapped.includes(tag)), 'game_icon_unmapped_tag');
  const icons = new Set(
    Object.values(snapshot.tags).flatMap(metadata => metadata.icons));
  assert(icons.size >= 4_130, `game_icon_tag_coverage:${icons.size}`);
}

function assert(condition, error) {
  if (!condition) throw new Error(error);
}
