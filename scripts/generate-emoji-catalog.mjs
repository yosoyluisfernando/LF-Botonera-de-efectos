import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const outputDir = path.join(root, 'src-tauri', 'resources', 'emoji');
const emojiVersion = '17.0';
const cldrVersion = '48.2.0';
const emojiUrl = `https://www.unicode.org/Public/${emojiVersion}.0/emoji/emoji-test.txt`;
const cldrRoot = `https://raw.githubusercontent.com/unicode-org/cldr-json/${cldrVersion}`;
const licenseUrl = `${cldrRoot}/LICENSE`;
const radioAliases = JSON.parse(
    fs.readFileSync(path.join(root, 'scripts', 'emoji-radio-aliases.json'), 'utf8'),
);
const locales = [
    { app: 'es', cldr: 'es' },
    { app: 'en', cldr: 'en' },
    { app: 'pt-BR', cldr: 'pt' },
    { app: 'pt-PT', cldr: 'pt-PT' },
];

async function getText(url) {
    const response = await fetch(url, { headers: { 'User-Agent': 'LF-Botonera-catalog-generator' } });
    if (!response.ok) throw new Error(`${response.status} al descargar ${url}`);
    return response.text();
}

async function getJson(url) {
    return JSON.parse(await getText(url));
}

function slug(value) {
    return value.toLowerCase().replaceAll('&', 'and')
        .replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
}

function parseEmojiTest(text) {
    let group = '';
    let subgroup = '';
    const items = [];
    for (const line of text.split(/\r?\n/)) {
        if (line.startsWith('# group:')) {
            group = slug(line.slice(8).trim());
            continue;
        }
        if (line.startsWith('# subgroup:')) {
            subgroup = slug(line.slice(11).trim());
            continue;
        }
        const match = line.match(/^([0-9A-F ]+)\s*;\s*(fully-qualified|component)\s*#/);
        if (!match) continue;
        const emoji = match[1].trim().split(/\s+/)
            .map(value => String.fromCodePoint(Number.parseInt(value, 16))).join('');
        items.push({ emoji, group, subgroup });
    }
    if (items.length !== 3953) {
        throw new Error(`Unicode ${emojiVersion}: se esperaban 3953 RGI y llegaron ${items.length}`);
    }
    return items;
}

function annotationMap(json) {
    return json.annotations?.annotations
        ?? json.annotationsDerived?.annotations
        ?? {};
}

async function readAnnotations(locale) {
    const base = `${cldrRoot}/cldr-json/cldr-annotations-full/annotations/${locale}/annotations.json`;
    const derived = `${cldrRoot}/cldr-json/cldr-annotations-derived-full/annotationsDerived/${locale}/annotations.json`;
    const combined = { ...annotationMap(await getJson(base)) };
    for (const [emoji, annotation] of Object.entries(annotationMap(await getJson(derived)))) {
        const current = combined[emoji];
        combined[emoji] = current ? {
            default: [...new Set([...(current.default ?? []), ...(annotation.default ?? [])])],
            tts: current.tts ?? annotation.tts,
        } : annotation;
    }
    return combined;
}

function entryFor(item, local, english, aliases) {
    const baseEmoji = item.emoji.replaceAll('\uFE0F', '');
    const annotation = local[item.emoji] ?? local[baseEmoji];
    const fallback = english[item.emoji] ?? english[baseEmoji];
    const name = annotation?.tts?.[0] ?? fallback?.tts?.[0];
    if (!name) {
        throw new Error(`Sin nombre CLDR para ${item.emoji}`);
    }
    return {
        value: item.emoji,
        group: item.group,
        subgroup: item.subgroup,
        name,
        keywords: [...new Set([
            ...(annotation?.default ?? fallback?.default ?? []),
            ...(aliases[item.emoji] ?? aliases[baseEmoji] ?? []),
        ])],
    };
}

function ndjson(header, entries) {
    return [JSON.stringify(header), ...entries.map(entry => JSON.stringify(entry))].join('\n') + '\n';
}

function sha256(text) {
    return crypto.createHash('sha256').update(text).digest('hex');
}

const emojiTest = await getText(emojiUrl);
const unicodeLicense = await getText(licenseUrl);
const emojiItems = parseEmojiTest(emojiTest);
const emojiValues = new Set(emojiItems.map(item => item.emoji));
for (const locale of locales) {
    const aliases = radioAliases[locale.app];
    if (!aliases) throw new Error(`Faltan alias de radio para ${locale.app}`);
    for (const emoji of Object.keys(aliases)) {
        if (!emojiValues.has(emoji)) {
            throw new Error(`Alias de ${locale.app} apunta a un emoji no RGI: ${emoji}`);
        }
    }
}
const annotations = Object.fromEntries(await Promise.all(
    locales.map(async locale => [locale.app, await readAnnotations(locale.cldr)]),
));

fs.mkdirSync(outputDir, { recursive: true });
const files = [];
for (const locale of locales) {
    const entries = emojiItems.map(item =>
        entryFor(item, annotations[locale.app], annotations.en, radioAliases[locale.app]));
    const content = ndjson({
        format: 1,
        emoji_version: emojiVersion,
        cldr_version: cldrVersion,
        locale: locale.app,
        count: entries.length,
    }, entries);
    const name = `${locale.app}.ndjson`;
    fs.writeFileSync(path.join(outputDir, name), content, 'utf8');
    files.push({ name, bytes: Buffer.byteLength(content), sha256: sha256(content) });
}

const manifest = {
    format: 1,
    emoji_version: emojiVersion,
    cldr_version: cldrVersion,
    count: emojiItems.length,
    sources: {
        emoji_test: emojiUrl,
        cldr_json_tag: cldrVersion,
        license: licenseUrl,
    },
    files,
};
fs.writeFileSync(
    path.join(outputDir, 'manifest.json'),
    `${JSON.stringify(manifest, null, 2)}\n`,
    'utf8',
);
fs.writeFileSync(path.join(outputDir, 'UNICODE_LICENSE.txt'), unicodeLicense, 'utf8');

console.log(`Generados ${files.length} catálogos de ${emojiItems.length} emojis.`);
for (const file of files) console.log(`${file.name}: ${file.bytes} bytes, ${file.sha256}`);
