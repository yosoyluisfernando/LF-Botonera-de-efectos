# Avisos de software de terceros / Third-Party Software Notices

LF Botonera de Efectos se distribuye bajo `GPL-3.0-or-later`. También incorpora
bibliotecas de terceros con licencias compatibles propias. Esas bibliotecas conservan
sus titulares de derechos y sus condiciones originales.

## Inventario auditado

Inventario obtenido el 2026-07-20 desde los archivos bloqueados de la versión 1.2.0:

- 585 paquetes en la resolución completa de Cargo para todas las plataformas y
  herramientas de compilación.
- 61 paquetes en `package-lock.json` para el frontend y la herramienta de compilación.
- Todos los paquetes resueltos declaran una licencia SPDX o una expresión de licencia.

Las familias presentes incluyen MIT, Apache-2.0, MPL-2.0, Unicode-3.0, BSD-2-Clause,
BSD-3-Clause, ISC, Zlib, 0BSD, Unlicense, CC0-1.0 y CDLA-Permissive-2.0, además de
expresiones que permiten elegir entre varias de ellas. La aplicación principal declara
GPL-3.0-or-later.

## Fuente de verdad

Los nombres, versiones, fuentes y sumas exactos están fijados en:

- `src-tauri/Cargo.lock`
- `package-lock.json`

Los manifiestos que seleccionan las dependencias directas son:

- `src-tauri/Cargo.toml`
- `package.json`

## Datos Unicode

El catálogo local de identificadores visuales se genera a partir de Unicode Emoji
17.0 y Unicode CLDR 48.2.0. No es una dependencia de red en tiempo de ejecución:
`scripts/generate-emoji-catalog.mjs` fija las fuentes oficiales y produce los
archivos versionados de `src-tauri/resources/emoji/`.

La distribución incluye el texto aplicable en
`legal/UNICODE_LICENSE.txt`, generado desde la licencia fijada por la etiqueta CLDR
48.2.0. Los datos y sus titulares conservan sus condiciones originales.

La representación colorida usa Noto Emoji COLRv1, fijada al commit
`8998f5dd683424a73e2314a8c1f1e359c19e8742`. Su licencia se incluye como
`legal/NOTO_EMOJI_LICENSE.txt`.

## Material Symbols

La colección local `Básicos` usa un subconjunto de Material Symbols Rounded de
Google, obtenido mediante la API oficial de Google Fonts y validado contra el
repositorio oficial fijado en el commit registrado por
`scripts/basic-symbols.json`. La aplicación no descarga la fuente en tiempo de
ejecución.

Material Symbols se distribuye bajo Apache License 2.0. El texto correspondiente se
incluye como `legal/MATERIAL_SYMBOLS_LICENSE.txt`.

## Tabler Icons

La colección monocromática `Básicos` incorpora una selección automática de Tabler
Icons `v3.46.0`: se excluyen marcas y variantes terminadas en `-off` para reducir
riesgo de marcas y repeticiones visuales. Los SVG se fijan al commit registrado en
`scripts/visual-packs.json`, se agrupan localmente por categoría y no realizan
peticiones de red durante el uso.

Tabler Icons se distribuye bajo licencia MIT. El texto correspondiente se incluye
como `legal/TABLER_LICENSE.txt`.

## Game Icons

`Básicos` incorpora además los pictogramas de Game Icons fijados al commit indicado
en `scripts/visual-packs.json`. Se excluye la carpeta auxiliar `badges` y, cuando dos
autores publicaron el mismo identificador, se conserva una sola variante
determinista. Las 134 páginas oficiales de etiquetas se conservan como una
instantánea local para clasificar los diseños. Los iconos se normalizan a un único
color para adaptarse al texto del botón.

Los iconos se distribuyen bajo CC BY 3.0 o CC0 cuando el archivo de licencia de la
colección identifica esa excepción. La atribución completa de Lorc, Delapouite y los
demás autores se conserva en `legal/GAME_ICONS_LICENSE.txt`.

Para verificar el inventario Rust sin modificar el proyecto:

```powershell
cd src-tauri
$metadata = cargo metadata --format-version 1 --locked | ConvertFrom-Json
$metadata.packages.Count
$metadata.packages | Group-Object license | Sort-Object Count -Descending
```

Para verificar el inventario Node sin instalar paquetes adicionales:

```powershell
node -e "const p=require('./package-lock.json').packages; const x=Object.entries(p).filter(([k])=>k); console.log(x.length)"
```

## Informes completos

Los textos y avisos se generan en modo estricto desde los archivos bloqueados:

- [Licencias de componentes Rust](THIRD_PARTY_LICENSES_RUST.html)
- [Licencias de herramientas Node y frontend generado](THIRD_PARTY_LICENSES_NODE.html)

El informe Rust usa `cargo-about` 0.9.1, fijado por versión, con objetivos Windows y
Linux. Omite dependencias exclusivas de desarrollo o compilación y ejecuta con
`--frozen --fail`: no usa red, no cambia `Cargo.lock` y falla si una licencia no puede
resolverse. El resultado actual contiene 404 componentes y ocho textos de licencia
seleccionados conforme a sus expresiones SPDX.

El informe Node se genera con un script local sin dependencias adicionales. Registra
los 61 paquetes bloqueados. `node_modules` se usa para compilar y no se copia al paquete
de escritorio; se conservan además los avisos agregados de Vite, que puede aportar
código al frontend producido, y las licencias de Tauri CLI como transparencia de la
cadena de build.

Regeneración completa:

```powershell
cargo install cargo-about --version 0.9.1 --locked --features cli
npm run licenses
```

Estos informes deben regenerarse, revisarse e incluirse en cada candidato de release.

## Third-party notice

Third-party components remain subject to their respective licenses and copyright
notices. The lockfiles listed above are the authoritative inventory for version 1.2.0.
The linked Rust and Node reports contain the generated release-specific license texts
and notices and must be regenerated and included with every distributed package.
