import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const read = relative => fs.readFileSync(path.join(root, relative), 'utf8');
const json = relative => JSON.parse(read(relative));
const escape = value => String(value)
    .replace(/[ \t]+$/gm, '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');

const lock = json('package-lock.json');
const packages = Object.entries(lock.packages)
    .filter(([location]) => location)
    .map(([location, data]) => ({
        name: location.replace(/^node_modules\//, ''),
        version: data.version,
        license: data.license,
    }))
    .sort((a, b) => a.name.localeCompare(b.name));

const missing = packages.filter(item => !item.version || !item.license);
if (missing.length) {
    throw new Error(`Paquetes sin versión o licencia: ${missing.map(x => x.name).join(', ')}`);
}

const appPackage = json('package.json');
const vitePackage = json('node_modules/vite/package.json');
const tauriPackage = json('node_modules/@tauri-apps/cli/package.json');
const viteNotices = read('node_modules/vite/LICENSE.md');
const tauriMit = read('node_modules/@tauri-apps/cli/LICENSE_MIT');
const tauriApache = read('node_modules/@tauri-apps/cli/LICENSE_APACHE-2.0');

const rows = packages.map(item => `
      <tr><td>${escape(item.name)}</td><td>${escape(item.version)}</td><td>${escape(item.license)}</td></tr>`)
    .join('');

const html = `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>LF Botonera de Efectos — Node build licenses</title>
  <style>
    body { font-family: sans-serif; line-height: 1.5; margin: 2rem; }
    main { max-width: 70rem; margin: 0 auto; }
    table { border-collapse: collapse; width: 100%; }
    th, td { border: 1px solid #777; padding: .4rem; text-align: left; }
    pre { border: 1px solid #777; padding: 1rem; white-space: pre-wrap; }
    @media (prefers-color-scheme: dark) {
      body { background: #202124; color: #f1f3f4; }
      a { color: #8ab4f8; }
    }
  </style>
</head>
<body>
<main>
  <h1>Node build tools and licenses</h1>
  <p>Generated for LF Botonera de Efectos ${escape(appPackage.version)} from package-lock.json.</p>
  <p>Node modules are build tools and are not copied as node_modules into the desktop package.
  Vite may contribute runtime helpers to the generated frontend, so its complete aggregated
  notice is preserved below. Tauri CLI is listed for build-chain transparency.</p>
  <h2>Locked build dependency graph (${packages.length} packages)</h2>
  <table>
    <thead><tr><th>Package</th><th>Version</th><th>Declared license</th></tr></thead>
    <tbody>${rows}
    </tbody>
  </table>
  <h2>Vite ${escape(vitePackage.version)} aggregated notices</h2>
  <pre>${escape(viteNotices)}</pre>
  <h2>Tauri CLI ${escape(tauriPackage.version)} build-tool licenses</h2>
  <h3>MIT</h3><pre>${escape(tauriMit)}</pre>
  <h3>Apache-2.0</h3><pre>${escape(tauriApache)}</pre>
</main>
</body>
</html>
`;

fs.writeFileSync(path.join(root, 'THIRD_PARTY_LICENSES_NODE.html'), html, 'utf8');
console.log(`Generated THIRD_PARTY_LICENSES_NODE.html with ${packages.length} locked packages.`);
