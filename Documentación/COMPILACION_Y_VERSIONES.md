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

### 2.1 Plataforma, formato y canal no son lo mismo

Estas tres decisiones deben mantenerse separadas:

- **Plataforma:** Windows o Linux. Rust la conoce mediante `target_os`.
- **Formato:** EXE, MSI, MSIX, DEB, RPM, AppImage o Flatpak.
- **Canal de distribución:** quién entregó la aplicación y quién administra sus
  actualizaciones.

El formato no permite deducir el canal. Un DEB descargado desde GitHub es `direct`
y conserva GitHub Releases; un DEB instalado desde un repositorio APT será un canal
administrado y no deberá consultar GitHub. Tampoco se debe usar el sistema operativo
como canal: Windows puede ser `direct` o `store`, y Linux puede ser directo, Flathub
o un repositorio administrado.

La fuente única actual está en `src-tauri/src/domain/distribution.rs`. El valor se
incorpora al ejecutable durante la compilación mediante
`LF_DISTRIBUTION_CHANNEL`:

- `direct`: GitHub Releases administra las actualizaciones.
- `store`: Microsoft Store administra las actualizaciones y GitHub queda desactivado.

La ausencia de la variable también produce `direct` para conservar compatibilidad
con los builds existentes. Aun así, en una compilación manual conviene declararla
explícitamente para evitar heredar por accidente un valor de otra terminal.

Compilación directa en PowerShell:

```powershell
$env:LF_DISTRIBUTION_CHANNEL = 'direct'
npm run tauri build
Remove-Item Env:LF_DISTRIBUTION_CHANNEL
```

Compilación directa en Linux:

```bash
LF_DISTRIBUTION_CHANNEL=direct npm run tauri build
```

GitHub Actions usa `.github/workflows/release-builds.yml` y actualmente obtiene el
mismo resultado directo por el valor predeterminado. Sus EXE, MSI, DEB, RPM y
AppImage pertenecen todos a GitHub Releases.

Microsoft Store **no** se compila con el comando genérico. Se usa:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build-store-msix.ps1
```

Ese script fija temporalmente `LF_DISTRIBUTION_CHANNEL=store`, compila en Release,
genera el MSIX y restaura la variable previa aunque ocurra un error.

Antes de entregar cualquier paquete se comprueba en «Acerca de»:

1. versión;
2. sistema;
3. canal de distribución;
4. administrador de actualizaciones.

Los futuros canales `flatpak`, `apt` o `dnf` todavía no están implementados. Cuando
se aprueben deberán añadirse a la misma fuente Rust y a su pipeline de empaquetado.
No se reutilizará `store` como nombre genérico ni se intentará detectar el origen por
la extensión, la ruta de instalación o una heurística en tiempo de ejecución.

## 3. Subida de Version

La versión funcional debe ser una sola para todos los canales disponibles. No se
mantendrán números distintos para GitHub, Microsoft Store y Linux. Si una tienda
obliga a corregir y subir el número antes de completar la publicación coordinada, se
actualizan los archivos fuente una sola vez y los siguientes paquetes de todos los
canales se generan con ese mismo número. El cuarto componente técnico del MSIX
(`X.Y.Z.0`) no crea una versión funcional diferente.

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
