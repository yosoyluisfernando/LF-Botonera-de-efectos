# Compilar LF Botonera de Efectos

## Antes de compilar: elegir el canal

**Leer [Documentación/CANALES_COMPILACION.md](Documentación/CANALES_COMPILACION.md).**
La versión, el nombre, el identificador y el código de actualización deben
corresponder al canal. Quitar `-beta.N` de la versión no convierte la edición
de pruebas en estable. Esta comprobación aplica también a GitHub Actions.

Escritorio conserva **LF Botonera de Efectos Beta**, aislada de la estable.
Android prepara **LF Botonera de Efectos 1.6.0** con identidad y firma propias;
leer [ANDROID_ESTABLE.md](Documentación/ANDROID_ESTABLE.md).
La ruta principal es `C:/OVERLAY/BOTONERA`, rama `codex/desarrollo`.
Herramientas, SDK, cachés, informes y compilados se conservan en D: en este equipo.

## Requisitos

- Node.js 24 recomendado y npm; usar las versiones indicadas en los workflows.
- Rust estable y dependencias instaladas con `npm ci` desde la raíz.
- Windows: Build Tools C++, Windows SDK y WebView2 Runtime.
- Linux: las dependencias del ejecutor Ubuntu 22.04 están en
  `.github/workflows/release-builds.yml`. Incluyen WebKitGTK 4.1, ALSA, D-Bus,
  OpenSSL, AppIndicator, librsvg, libxdo, patchelf, rpm y herramientas de compilación.
- Android: JDK 21, SDK/plataforma 36, build-tools 36.0.0 y NDK 27.2.12479018,
  según `.github/workflows/android.yml`; target Rust `aarch64-linux-android`.
  `JAVA_HOME`, `ANDROID_HOME` y `NDK_HOME` deben señalar las herramientas reales.
  En este equipo se mantienen en D:, sin descargar otro entorno en C:.

## Verificar sin abrir la aplicación ni crear instaladores

```bash
npm ci
node --test --test-concurrency=1 scripts/*.test.mjs
cd src-tauri
cargo test --lib
cargo build --lib
cd ..
npm run build
```

`npm run build` compila el frontend; `cargo build --lib` verifica la biblioteca.
No equivalen a generar instaladores. La prueba funcional la hace el autor.

## Windows: EXE y MSI

Después de comprobar el canal y sus versiones:

```bash
npm run tauri -- build --bundles nsis,msi
```

En este equipo puede usarse el entorno MSVC existente, desde CMD:

```bat
cd /d C:\OVERLAY\BOTONERA
call D:\BOTONERA\app\.tools\msvc\entorno-msvc.cmd
npm run tauri -- build --bundles nsis,msi
```

Ese entorno fija `CARGO_TARGET_DIR` y temporales en D:.

| Salida | Carpeta bajo el directorio de destino de Cargo |
| --- | --- |
| Instalador EXE | `release/bundle/nsis/` |
| Instalador MSI | `release/bundle/msi/` |
| Ejecutable sin empaquetar | `release/tauri-app.exe` |

El ejecutable suelto no se anuncia como distribución portable validada sin
comprobar sus recursos y WebView2. Microsoft Store tiene un flujo propio:
`scripts/build-store-msix.ps1` y `Documentación/store/README.md`.

## Linux: DEB, RPM y AppImage

Compilar en Linux con las dependencias del workflow:

```bash
npm ci
npm run tauri -- build --bundles deb,appimage,rpm
```

Los paquetes quedan bajo `release/bundle/deb/`, `rpm/` y `appimage/`.
El ejecutor actual construye para x86_64. Comprobar la distribución de destino;
AppImage no implica compatibilidad universal con cualquier sistema Linux.
Flatpak continúa como trabajo separado y no se ofrece como paquete disponible.

## Android: APK ARM64 de pruebas

Comprobar primero el canal efectivo en `tauri.android.conf.json`.
La identidad Android es independiente de la identidad estable de escritorio.

```bash
npm ci
npm run tauri -- android init --ci --skip-targets-install
python scripts/prepare-android-midi.py src-tauri/gen/android
```

Aplicar el icono versionado después de generar el proyecto. En PowerShell:

```powershell
$res = 'src-tauri/gen/android/app/src/main/res'
Copy-Item 'src-tauri/icons/android/*' $res -Recurse -Force
Copy-Item "$res/mipmap-anydpi-v26/ic_launcher.xml" "$res/mipmap-anydpi-v26/ic_launcher_round.xml" -Force
npm run tauri -- android build --ci --debug --apk --target aarch64
```

En Linux, los pasos equivalentes de copia están en `android.yml`.
No omitir `prepare-android-midi.py`: incorpora los módulos Kotlin de MIDI y
carpetas, permisos y ajustes del proyecto generado. `gen/android` no sustituye
al código fuente versionado en `src-tauri/android/`.

Localizar el APK en `src-tauri/gen/android/app/build/outputs/apk/` y comprobar
su firma con `apksigner verify --verbose`. El flujo de comprobación usa firma de debug:
no es el procedimiento para una publicación estable de Android.

## Dónde encontrar los resultados

Sin personalización, Cargo usa `src-tauri/target/`. Con `CARGO_TARGET_DIR`,
consultar esa ruta; no asumir que los archivos quedaron en C:.
Guardar las entregas al autor en `D:/BOTONERA/Compilados/`, identificando
sistema, canal y versión. Conservar versiones anteriores y comprobar SHA-256.

## Desarrollo y compilación automática

- `DEV.bat` o `npm run tauri -- dev`: desarrollo con recarga; lo ejecuta el autor.
- `release-builds.yml`: ejecución manual, elige ref, Windows/Linux/ambos y
  `publish`. Para pruebas usar `publish=false`; subir assets requiere una
  petición de publicación y un release de destino ya preparado.
- `build.yml`: los tags finales `vX.Y.Z` pueden publicar automáticamente un
  release estable. Excluye tags con guion, como `v1.6.0-beta.5`.
- `android.yml`: comprobación Android en `codex/desarrollo`; la ejecución manual
  con `build_release=true` añade un APK de producción para firma local.
  Compilar no publica automáticamente un release ni registra una tienda.

**No crear ni empujar tags ni activar publicación como parte de una simple
verificación, cambio de nombre o actualización de documentación.**
Main se conserva como referencia estable; trabajar en `codex/desarrollo`.

## Fuentes técnicas

- [CLI oficial de Tauri](https://v2.tauri.app/reference/cli/).
- [Configuración y opciones de empaquetado](https://v2.tauri.app/reference/config/).
- Workflows versionados en `.github/workflows/`: referencia de las versiones y
  comandos que realmente utiliza este proyecto.
