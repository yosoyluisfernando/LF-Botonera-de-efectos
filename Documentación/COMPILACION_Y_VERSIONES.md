# REGLAS DE COMPILACION Y VERSIONES

Este documento define los sistemas operativos soportados y el proceso de
compilacion, versionado y publicacion del proyecto Tauri + Rust.

La estrategia para Microsoft Store, Flathub y repositorios oficiales de Linux
se mantiene en [`PLAN_DISTRIBUCION_TIENDAS.md`](PLAN_DISTRIBUCION_TIENDAS.md).
Los instaladores descritos aquí son artefactos técnicos; no todos cumplen todavía
los requisitos de una tienda.

## 1. Sistemas Operativos Soportados

**Windows 10 y 11**

* Windows 7, 8 y 8.1 quedan descartados.
* Tauri usa WebView2 para renderizar la interfaz. Microsoft finalizo el soporte
  de WebView2 para Windows 7 y 8 en enero de 2023.
* El compilador moderno de Rust tambien exige Windows 10 como base practica.
* Soporte final: Windows 10 y Windows 11.

**Linux**

* Se compila nativamente en paquetes `.deb`, `.rpm` y `.AppImage`.
* Los assets Linux deben generarse desde un runner Linux o una maquina Linux.

## 2. Compilacion

Preparacion inicial:

```bash
npm install
```

Compilar para Windows:

```bash
npm run tauri build
```

Salidas esperadas:

* `src-tauri/target/release/tauri-app.exe`
* `src-tauri/target/release/bundle/nsis/*.exe`
* `src-tauri/target/release/bundle/msi/*.msi`

Compilar para Linux:

```bash
npm run tauri build
```

Salidas esperadas:

* `src-tauri/target/release/bundle/deb/*.deb`
* `src-tauri/target/release/bundle/rpm/*.rpm`
* `src-tauri/target/release/bundle/appimage/*.AppImage`

## 3. Subida de Version

Para publicar una version nueva, ejecutar desde la raiz del proyecto:

```
SET-VERSION.bat X.Y.Z
```

El script actualiza en un solo paso:

1. `package.json` y `package-lock.json` (via `npm version`)
2. `src-tauri/Cargo.toml` (primera ocurrencia, seccion `[package]`)
3. `src-tauri/tauri.conf.json`

Al terminar, el script recuerda los pasos siguientes:

```bash
cd src-tauri
cargo check           # regenera Cargo.lock con la version correcta
```

Luego actualizar `CHANGELOG.md` con los cambios de la nueva version, hacer
commit y crear el tag:

```bash
git commit -am "Release X.Y.Z"
git tag vX.Y.Z
git push && git push --tags
```

## 4. Publicacion y Antivirus

Antes de publicar o reemplazar assets de Windows en GitHub Releases, es
obligatorio descargar y escanear exactamente los instaladores publicados.

Motivo documentado: en la version `1.0.4`, el instalador NSIS `.exe` generado
por GitHub Actions fue bloqueado por Defender/Edge al descargarse desde GitHub,
aunque el instalador NSIS compilado localmente, el ejecutable local y el MSI
escaneaban limpios. Fue un falso positivo asociado al asset remoto publicado,
no al codigo fuente de la aplicacion.

Checklist obligatorio para releases:

1. Compilar localmente Windows con `npm run tauri build`.
2. Copiar los artefactos limpios a `Compilados`.
3. Escanear `Compilados\*.exe` y `Compilados\*.msi` con Windows Defender.
4. Crear tag y release.
5. Ejecutar GitHub Actions para compilar Windows y Linux.
6. Descargar desde GitHub los assets Windows publicados.
7. Escanear los archivos descargados, no solo los locales.
8. Si el NSIS remoto falla pero el local pasa, reemplazar el asset remoto con
   el instalador local limpio usando `gh release upload --clobber`.
9. Descargar de nuevo el asset reemplazado desde GitHub y escanearlo otra vez.

El MSI debe mantenerse como alternativa principal cuando el NSIS active falsos
positivos. El `upgradeCode` de MSI no debe cambiar entre versiones, porque
Windows lo usa para reconocer actualizaciones de la misma aplicacion.

## 5. Instaladores Windows

El instalador NSIS se configura en modo `perMachine` para instalar en
`C:\Program Files` y estar disponible para todos los usuarios del equipo.

Consecuencias:

* Windows puede pedir permisos de administrador.
* Si una version anterior fue instalada por usuario, el primer salto a
  instalacion por equipo puede pedir limpiar o reemplazar una vez.
* Las versiones futuras deben conservar la misma identidad de instalador para
  comportarse como actualizaciones, no como aplicaciones separadas.
