export function classifyGameIcon(slug, rules, officialTags = []) {
  const tags = new Set(officialTags);
  const tagged = rules.find(rule => rule.tags.some(tag => tags.has(tag)));
  if (tagged) return tagged.group;
  const tokens = new Set(slug.split('-'));
  return rules.find(rule => rule.tokens.some(token => tokens.has(token)))?.group
    || 'other';
}

export function ordered(items, config, packOrder) {
  const groupPosition = new Map(
    config.groupOrder.map((group, index) => [group, index]));
  const packPosition = new Map(packOrder.map((pack, index) => [pack, index]));
  return items.sort((left, right) =>
    (groupPosition.get(left.group) ?? 999) - (groupPosition.get(right.group) ?? 999)
    || (packPosition.get(left.pack) ?? 999) - (packPosition.get(right.pack) ?? 999)
    || left.slug.localeCompare(right.slug, 'en'));
}

export function namespacedValue(item) {
  return `${item.pack}:${item.group}:${item.slug}`;
}
