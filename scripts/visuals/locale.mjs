export function localizedEntry(item, locale, data) {
  if (item.locales) {
    const [name, sourceKeywords] = item.locales[locale];
    const keywords = new Set(sourceKeywords.split(/\s+/));
    data.groupTerms[item.group][locale].split(/\s+/).forEach(term => keywords.add(term));
    return {
      value: item.value,
      group: item.group,
      subgroup: item.pack,
      name,
      keywords: [...keywords].filter(Boolean),
    };
  }
  const tokens = item.slug.split('-').filter(Boolean);
  const name = locale === 'en'
    ? title(tokens.join(' '))
    : title(tokens.map(token => translated(token, locale, data)).join(' '));
  const keywords = new Set(tokens);
  tokens.forEach(token => keywords.add(translated(token, locale, data)));
  item.tags.flatMap(tag => words(tag)).forEach(token => {
    keywords.add(token);
    keywords.add(translated(token, locale, data));
  });
  data.groupTerms[item.group][locale].split(/\s+/).forEach(term => keywords.add(term));
  return {
    value: item.value,
    group: item.group,
    subgroup: item.pack,
    name,
    keywords: [...keywords].filter(Boolean),
  };
}

function translated(token, locale, data) {
  const override = data.overrides[token]?.[locale];
  if (override) return override;
  const generated = data.translations[token];
  if (!generated) return token;
  return locale === 'es' ? generated.es : generated.pt;
}

function words(value) {
  return value.toLowerCase().match(/[a-z]+/g) || [];
}

function title(value) {
  return value ? value[0].toLocaleUpperCase() + value.slice(1) : value;
}
