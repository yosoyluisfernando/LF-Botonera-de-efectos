# Compilar LF Botonera de Efectos desde el código fuente

## Requisitos previos (una sola vez)

### Windows
1. [Node.js 20+](https://nodejs.org/) con npm incluido.
2. [Rust (rustup)](https://rustup.rs/) — instala el toolchain `stable` por defecto.
3. [Build Tools para Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/) —
   seleccionar **"Desarrollo con C++"** (necesario para compilar dependencias nativas de Rust).
4. [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) —
   ya viene incluido en Windows 10 (20H2+) y Windows 11. Solo necesario instalarlo en versiones
   antiguas de Windows 10.

### Linux (Ubuntu 22.04 / Debian 12 o compatible)
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libasound2-dev \
  libxdo-dev \
  patchelf \
  rpm \
  squashfs-tools \
  build-essential

# Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

---

## Instalar dependencias del proyecto

En la raíz del repositorio, una sola vez:

```bash
npm install
```

---

## Compilar (producción)

El mismo comando sirve en Windows y Linux:

```bash
npm run tauri build
```

Los artefactos quedan en `src-tauri/target/release/bundle/`.

---

## Formatos de salida

### Windows

| Formato | Ubicación | Descripción |
|---|---|---|
| `.msi` | `bundle/msi/*.msi` | Instalador MSI (Windows Installer). Se integra con Agregar/Quitar programas. Recomendado para despliegues empresariales. |
| `.exe` (instalador NSIS) | `bundle/nsis/*-setup.exe` | Instalador gráfico clásico con asistente paso a paso. El más sencillo para usuarios finales. |
| `.exe` (portable) | `target/release/tauri-app.exe` | Ejecutable directo sin instalación. Requiere que WebView2 Runtime esté instalado en el sistema. No crea accesos directos ni entradas en el registro. |

> **¿Cuál usar?**  
> Para distribuir a usuarios finales: el `.exe` NSIS (instalador).  
> Para pruebas rápidas o uso sin instalación: el `.exe` portable.  
> Para entornos corporativos con políticas de software: el `.msi`.

### Linux

| Formato | Ubicación | Descripción |
|---|---|---|
| `.deb` | `bundle/deb/*.deb` | Paquete para Ubuntu, Debian, Linux Mint y derivados. Se instala con `dpkg -i` o doble clic en el gestor de archivos. |
| `.rpm` | `bundle/rpm/*.rpm` | Paquete para Fedora, openSUSE y derivados compatibles con RPM. |
| `.AppImage` | `bundle/appimage/*.AppImage` | Ejecutable universal sin instalación. Funciona en cualquier distribución Linux con glibc 2.31+. Dar permisos de ejecución con `chmod +x` y ejecutar directamente. |

---

## Modo desarrollo (con recarga en vivo)

```bash
npm run tauri dev
```

Abre la app con hot-reload del frontend. Los cambios en `src/` se reflejan al instante;
los cambios en `src-tauri/src/` requieren recompilación de Rust (automática pero más lenta).

---

## Actualizar la versión

La versión se declara en **tres archivos que deben coincidir**:

| Archivo | Campo |
|---|---|
| `package.json` | `"version"` |
| `src-tauri/Cargo.toml` | `version` en `[package]` |
| `src-tauri/tauri.conf.json` | `"version"` |

Cambiar los tres al mismo valor antes de compilar la release.

---

## Compilación automática con GitHub Actions

Al hacer push de un tag con el formato `v*` (ejemplo: `v1.1.1`), el workflow
`.github/workflows/build.yml` compila automáticamente en Windows y Linux y sube
los artefactos (`.msi`, `.exe`, `.deb`, `.rpm`, `.AppImage`) como descargables del release.

```bash
git tag v1.1.1
git push origin v1.1.1
```

Los artefactos aparecen en la pestaña **Actions** del repositorio de GitHub, en la
ejecución correspondiente al tag.
