# Canales de compilación: Beta y Estable

Decisión del autor, 2026-11-11. Leer junto a [COMPILAR.md](../COMPILAR.md).
La rama de trabajo es `codex/desarrollo`; main conserva la referencia estable.

## Regla para próximos chats

Antes de generar un paquete, identificar **sistema operativo + canal + versión**.
No inferir el canal únicamente por el número de versión o por el nombre del tag.
No publicar ni generar instaladores si el autor solo pide preparar o documentar.

El nombre visible de las pruebas pasa de «LF Botonera LAN Prueba» a
**LF Botonera de Efectos Beta**. «LAN» era el nombre del experimento, no el canal.
El cambio se verá en próximas compilaciones y arranques de desarrollo; no cambia
por sí mismo el nombre de las aplicaciones que ya están instaladas.

## Windows y Linux: identidades que deben conservarse

| Dato | Beta / desarrollo | Estable de escritorio |
| --- | --- | --- |
| Nombre del producto | `LF Botonera de Efectos Beta` | `LF Botonera de Efectos` |
| Título principal | `LF Botonera de Efectos Beta` | `LF Botonera de Efectos` |
| Identificador Tauri | `io.github.yosoyluisfernando.lfbotonera.lanprueba` | `io.github.yosoyluisfernando.LF-Botonera-de-efectos` |
| MSI upgradeCode | `7eeb01c0-09c5-5e7c-b5ce-f69c2081adbf` | `43888972-c5a4-5e8d-a996-ca913f3b6d8e` |
| Versión de ejemplo | `1.6.0-beta.5` | `1.6.0` |
| Versión MSI de ejemplo | `1.6.0.5` | `1.6.0` |

La columna estable se verificó contra `origin/main` de la versión 1.5.0.
El identificador y upgradeCode de la beta conservan los valores históricos del
experimento para mantener la continuidad de su configuración e instaladores.
No hacer un reemplazo global de `lanprueba`: también identifica el paquete de
Android y sus scripts de prueba. Renombrarlo sería otra migración.

El nombre y la identidad cumplen funciones diferentes. La identidad separa los
datos de pruebas de los datos estables; no copiar ni fusionar automáticamente
perfiles, configuración o bases de datos entre ambos canales.

### Rutas y comprobación de actualizaciones

- Windows: la carpeta predeterminada de una instalación nueva se corresponde
  con el nombre del producto. Una instalación previa o una ruta elegida por el
  usuario puede conservar otro destino; comprobar el instalador resultante.
- Estable: restaurar nombre, identificador y upgradeCode originales para
  preparar la actualización de la edición habitual; verificar desde 1.5.0 con
  respaldo de los datos del usuario.
- Beta: comprobar la transición desde «LAN Prueba» hacia «Beta». El mismo ID
  conserva la identidad de datos, pero el renombrado puede afectar carpetas,
  accesos directos, entradas de desinstalación y nombres de paquetes.
- Linux: revisar el nombre del paquete y el archivo `.desktop`. El cambio de
  producto puede crear otro paquete frente al antiguo nombre LAN; no anunciar
  una actualización automática sin probarla.
- Microsoft Store: usar el flujo `scripts/build-store-msix.ps1`, manteniendo
  IdentityName y Publisher registrados. No subir una beta aislada como si fuera
  el paquete estable de Store. Su identidad no se sustituye por el GUID MSI.

## Versiones: cinco archivos relacionados

1. `package.json`: version.
2. `package-lock.json`: versión raíz y versión de `packages[""]`.
3. `src-tauri/Cargo.toml`: version solo en `[package]`.
4. `src-tauri/tauri.conf.json`: version superior y, en Windows, versión MSI.
5. `src-tauri/Cargo.lock`: regenerar con Cargo después de cambiar Cargo.toml.

Las versiones de aplicación deben coincidir. Para MSI se usa la versión numérica
admitida por Windows Installer; incrementar el número de compilación en cada beta.
Al preparar la estable, retirar el valor MSI de la beta y usar el de la estable.

**Limitación del helper existente:** `SET-VERSION.ps1` valida solo `X.Y.Z` y
sus sustituciones no cubren versiones de origen con `-beta.N`. No asumir que
`SET-VERSION.bat` convierte correctamente una beta en estable. Hasta ampliar y
verificar ese helper, actualizar los campos indicados y comprobar el resultado.
Se puede usar `npm version VERSION --no-git-tag-version --ignore-scripts` para
los dos archivos npm; eso no cambia los archivos Rust/Tauri ni el canal.

## Procedimiento Beta / desarrollo

1. Conservar el estado local del autor y trabajar en `codex/desarrollo`.
2. Comprobar la columna Beta en nombre, título, ID y upgradeCode.
3. Sincronizar una versión prerelease nueva y su MSI numérico.
4. Ejecutar las verificaciones de COMPILAR.md sin abrir la app.
5. Solo si se pidió empaquetar, generar los formatos del sistema operativo.
6. Guardar en `D:/BOTONERA/Compilados/` con canal y versión; verificar SHA-256,
   nombre, arquitectura y versión interna. Conservar el paquete anterior.
7. Una publicación de pruebas debe marcarse como prerelease; la decisión de
   publicar es independiente de compilar y requiere instrucción del autor.

## Procedimiento Estable de escritorio

1. Confirmar que el autor pidió preparar una estable; completar sus verificaciones.
2. Elegir el commit a publicar, conservando main como referencia hasta autorizar
   su actualización. No fusionar ramas ni mover main por cambiar el canal local.
3. Sincronizar `X.Y.Z` en las versiones de aplicación y restaurar los cuatro
   valores de identidad de la columna Estable: nombre, título, ID y upgradeCode.
4. Comprobar que MSI no conserva el número de la beta.
5. Revisar la configuración efectiva de Tauri y cualquier override por plataforma.
6. Verificar y compilar por separado Windows y Linux. Probar instalación limpia
   y actualización desde la estable anterior, conservando sus datos.
7. Preparar changelog y borrador del release con las funciones nuevas desde la
   estable anterior; las correcciones internas de funciones inéditas no se listan.
8. Publicar solo por petición del autor. `build.yml` puede publicar al empujar un
   tag final, por lo que el tag es un paso de publicación, no de comprobación.

## Android: primera estable 1.6.0 y Beta anterior

El autor aprobó una edición estable Android independiente el 2026-11-11.
El override `tauri.android.conf.json` fija LF Botonera de Efectos, versión 1.6.0,
paquete `io.github.yosoyluisfernando.lfbotonera` y versionCode 1006001 (revisión interna posterior al paquete inicial).
Los espacios entre nombre y versión son solo tipográficos; los valores de
configuración se verifican directamente en el paquete compilado.

Escritorio mantiene su versión e identidad Beta. Esta separación de versiones
por plataforma es deliberada: Android usa la versión efectiva de la configuración
Tauri, mientras Cargo/package.json conservan el desarrollo de escritorio.
La Beta anterior Android (`...lfbotonera.lanprueba`) mantiene sus propios datos
y certificado. Se puede instalar junto a la estable; no hay migración automática.

El workflow verifica una compilación de depuración en Android 35 y, al activar
`build_release`, genera y comprueba un APK de producción sin depuración.
La entrega se firma localmente con la clave privada del autor, con respaldo
fuera del repositorio. No publicar ni enviar esa clave a GitHub.

El único canal disponible actualmente es GitHub Releases. La identificación
futura de Google Play/F-Droid no supone disponibilidad en esas tiendas.
Ver [ANDROID_ESTABLE.md](ANDROID_ESTABLE.md) para firma, respaldo, canales,
versionCode, comprobaciones y pasos que siguen separados de la publicación.

## Comprobaciones obligatorias del paquete resultante

- Nombre visible y canal correctos; ID, versión y arquitectura efectivos.
- Windows: versión del EXE/MSI e identidad de actualización; comprobar la ruta
  real y la conservación de configuración al actualizar la edición anterior.
- Linux: metadatos DEB/RPM, archivo desktop y ejecución del AppImage en destino.
- Android: package/application ID, versionCode, etiqueta de la app y certificado
  de firma; repetir arranque, importación de audio y controles básicos.
- Calcular SHA-256 y registrar commit de aplicación y sistema de compilación.

Fuentes: [configuración Tauri](https://v2.tauri.app/reference/config/),
[CLI Tauri](https://v2.tauri.app/reference/cli/), configuración estable de
`origin/main` y workflows de este repositorio.
