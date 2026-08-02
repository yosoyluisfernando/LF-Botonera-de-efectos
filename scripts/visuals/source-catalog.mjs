import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import { classifyGameIcon } from './grouping.mjs';
import { gameSymbol, tablerSymbol } from './svg-tools.mjs';

export async function readTabler(sourceRoot, config) {
  const folder = path.join(sourceRoot, 'icons', 'outline');
  const files = (await readdir(folder)).filter(file => file.endsWith('.svg')).sort();
  const items = [];
  for (const file of files) {
    const slug = file.slice(0, -4);
    if (config.excludeSuffixes.some(suffix => slug.endsWith(suffix))) continue;
    const source = await readFile(path.join(folder, file), 'utf8');
    const category = field(source, 'category');
    if (config.excludeCategories.includes(category)) continue;
    const group = config.categoryMap[category] || 'other';
    items.push({
      pack: 'tabler', slug, group,
      tags: listField(source, 'tags'),
      symbol: tablerSymbol(slug, source),
    });
  }
  return items;
}

export async function readGameIcons(sourceRoot, config, rules, tagSnapshot) {
  const tagsByIcon = gameIconTags(tagSnapshot.tags);
  const folders = (await readdir(sourceRoot, { withFileTypes: true }))
    .filter(entry => entry.isDirectory() && !config.excludeFolders.includes(entry.name))
    .map(entry => entry.name)
    .sort();
  const unique = new Map();
  for (const folder of folders) {
    const files = (await readdir(path.join(sourceRoot, folder)))
      .filter(file => file.endsWith('.svg'))
      .sort();
    for (const file of files) {
      const slug = file.slice(0, -4);
      if (unique.has(slug)) continue;
      const source = await readFile(path.join(sourceRoot, folder, file), 'utf8');
      const officialTags = tagsByIcon.get(slug) || [];
      unique.set(slug, {
        pack: 'game-icons',
        slug,
        group: classifyGameIcon(slug, rules, officialTags),
        tags: officialTags
          .filter(tag => !config.excludeTags.includes(tag))
          .flatMap(tag => [tag, tagSnapshot.tags[tag].name]),
        symbol: gameSymbol(slug, source),
      });
    }
  }
  return [...unique.values()];
}

function gameIconTags(tags) {
  const byIcon = new Map();
  for (const [tag, metadata] of Object.entries(tags)) {
    for (const icon of metadata.icons) {
      if (!byIcon.has(icon)) byIcon.set(icon, []);
      byIcon.get(icon).push(tag);
    }
  }
  return byIcon;
}

function field(source, key) {
  return source.match(new RegExp(`^${key}:\\s*(.+)$`, 'm'))?.[1].trim() || '';
}

function listField(source, key) {
  const value = field(source, key).replace(/^\[|\]$/g, '');
  return value.split(',').map(item => item.trim()).filter(Boolean);
}
