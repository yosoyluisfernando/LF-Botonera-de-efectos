import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { writeCatalog } from './visuals/catalog-writer.mjs';
import { namespacedValue, ordered } from './visuals/grouping.mjs';
import { loadSources } from './visuals/source-loader.mjs';
import { readGameIcons, readTabler } from './visuals/source-catalog.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const readJson = async file => JSON.parse(
  await readFile(path.join(root, 'scripts', file), 'utf8'));
const config = await readJson('visual-packs.json');
const rules = await readJson('game-icon-groups.json');
const gameIconTags = await readJson('game-icon-tags.json');
const groupTerms = await readJson('visual-group-terms.json');
const overrides = await readJson('visual-token-overrides.json');
const translations = await readJson('visual-token-translations.json');
const locales = ['es', 'en', 'pt-BR', 'pt-PT'];
const sources = await loadSources(config);

const tabler = await readTabler(sources.tabler, {
  ...config.tabler,
  categoryMap: config.tablerCategories,
});
const gameIcons = await readGameIcons(
  sources.gameIcons, config.gameIcons, rules, gameIconTags);
const material = await materialItems();

tabler.forEach(item => { item.value = namespacedValue(item); });
gameIcons.forEach(item => { item.value = namespacedValue(item); });
assertUnique([...material, ...tabler, ...gameIcons]);

const basicItems = ordered(
  [...material, ...tabler, ...gameIcons],
  config,
  ['material', 'tabler', 'game-icons']);
const localeData = { groupTerms, overrides, translations };

const basic = await writeCatalog({
  root,
  kind: 'basic',
  items: basicItems,
  locales,
  localeData,
  sources: {
    material: { count: material.length, source: 'Material Symbols Rounded' },
    tabler: {
      count: tabler.length,
      version: config.tabler.version,
      commit: config.tabler.commit,
    },
    gameIcons: { count: gameIcons.length, commit: config.gameIcons.commit },
  },
  licenses: [
    {
      source: path.join(sources.tabler, 'LICENSE'),
      file: 'TABLER_LICENSE.txt',
    },
    {
      source: path.join(sources.gameIcons, 'license.txt'),
      file: 'GAME_ICONS_LICENSE.txt',
    },
  ],
});
console.log(`Básicos: ${basic.count}.`);
console.log(`Tabler ${tabler.length}; Game Icons ${gameIcons.length}.`);

async function materialItems() {
  const input = await readJson('basic-symbols.json');
  return input.symbols.map(item => ({
    pack: 'material',
    slug: item.id,
    value: item.id,
    group: config.materialGroups[item.group] || 'other',
    locales: Object.fromEntries(locales.map(locale => [locale, item[locale]])),
  }));
}

function assertUnique(items) {
  const values = new Set();
  for (const item of items) {
    if (values.has(item.value)) throw new Error(`duplicate_visual:${item.value}`);
    values.add(item.value);
  }
}
