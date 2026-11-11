# Continuidad de sesión — estado actual

## Apariencia de Windows — 2026-11-11

Corrección beta.2: la plataforma real es desktop, no windows. La primera
compilación ocultaba el selector por esa comparación incorrecta. Se corrigieron
el guard y el CSS, y se verificó el HTML real en un navegador sin interfaz.
El ejecutable instalado del autor coincidía por SHA-256 con el paquete anterior.
La nueva versión 1.6.0-beta.2 / MSI 1.6.0.2 permite distinguir la corrección.

Predeterminado y Moderno están integrados en Principal de codex/desarrollo.
La selección se aplica y guarda inmediatamente y admite claro/oscuro/sistema.
Solo Windows: Linux y las plataformas móviles conservan su interfaz.
El experimento original en C:/OVERLAY/BOTONERA se conserva sin modificaciones.
Ver APARIENCIA_WINDOWS.md para IPC, persistencia, capas CSS y verificación.

## Unificación de ramas — 2026-11-10

El repositorio GitHub se organiza en main y codex/desarrollo.
Main conserva el commit ed38181d1e318784020addeff621feea8b5bc304.
codex/desarrollo reúne Android, mejoras de escritorio, OBS, control remoto LAN,
pruebas de latencia, preparación de Flatpak y documentación de presentación.
Los historiales anteriores quedan integrados mediante commits de fusión.
Ver RAMAS_DESARROLLO.md para la organización y el respaldo local de Git.


## Nota de esta copia de trabajo — 2026-11-10

La rama de desarrollo unificada es codex/desarrollo, con versión 1.6.0-beta.1.
El APK Android comprobado se compiló desde 502a944 en GitHub Actions
[42944969653](https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/actions/runs/42944969653).

La interfaz de escritorio se restauró en 5bd84d2. Android incorpora el espejo
LAN, editor táctil, ajustes adaptados, panel completo en vertical y selector
de carpetas de locuciones. La barra Android usa iconos para perfiles y control
remoto; este acceso se oculta al desactivarlo en sus ajustes.

Se corrigieron identificadores duplicados de importación, la activación de
audio del perfil importado y la consulta del nombre real de documentos Android.
La duración se detecta por contenido para recuperar copias antiguas con extensión
incorrecta. Ver ANDROID_PERFILES_IMPORTACION.md y ANDROID_LOCUCIONES.md.

Comprobado localmente con 403 pruebas Rust aprobadas, 19 omitidas, compilación
de biblioteca Rust y frontend, y 10 pruebas de interfaz/traducciones.
El emulador Android 35 verificó exportación/importación de pestaña y perfil
con un MP3 sintético, recorte y ganancia. Midió señal tras importar, borrar
y reimportar, borrar el perfil original y recargar la interfaz.
También pasaron los iconos, la desactivación del acceso remoto y las carpetas.

El APK firmado está en D:/BOTONERA/Compilados/Pruebas-Android-Perfiles/.
El informe está en D:/BOTONERA/Informacion para el usuario/android-perfiles-importacion.html.
Queda la confirmación del autor en el teléfono y la prueba física en tablet.
Las herramientas MSVC de D:/BOTONERA/app/.tools/msvc/ permiten verificar Rust.

**Actualizado:** 2026-10-11
**Rama:** `main`
**Versión:** 1.5.0

## Estado de publicación

Microsoft Store confirmó publicada Submission 5 (`1152921505702099923`),
con el paquete 1.5.0.0 x64 y el mensaje de actualización disponible. La firma y
las actualizaciones de ese canal las administra Microsoft Store. El autor
autorizó publicar también `v1.5.0` en GitHub con MSI, EXE, DEB, AppImage y RPM.
El release `v1.5.0` está publicado como última versión estable con cinco adjuntos,
notas para usuarios con emojis y enlace a Microsoft Store. La etiqueta apunta a
`ccd0fceb0e14f1c0cc462bbd8470649cdac72337`. Los hashes SHA-256 de los cinco
adjuntos coinciden con las copias locales. La compilación duplicada `38197638896`
disparada por la etiqueta se canceló para conservar los paquetes ya verificados.
Los cinco paquetes proceden de GitHub Actions `38191218663`, compilado desde
`0395023f78d8755a152b993131e2b65c3a57671e`; el cierre de versión modifica
únicamente documentación y avisos de licencias, sin cambios en la aplicación.

La sección 1.5.0 de CHANGELOG está cerrada con fecha local 2026-10-11 y existe
una nueva sección `[Sin publicar]` vacía. Las cinco referencias de versión
(package.json, package-lock.json, Cargo.toml, Cargo.lock y tauri.conf.json)
coinciden en 1.5.0. Las ramas auxiliares integradas se retiraron; solo queda main.

## Paquetes y evidencia

- GitHub: `Compilados/GitHub-1.5.0/`, instaladores Windows y Linux descargados
  del run correcto. Notas para usuarios con emojis en `release-notes.md`.
- Store: `Compilados/Microsoft-Store-1.5.0/LF-Botonera-1.5.0.0-x64-unsigned.msix`.
  17.380.834 bytes; SHA-256:
  `8D3F5887B6A897D67AE4BC77700DAF8024EDB08649C0BD49EFF675B1F922D971`.
- Identidad: `LuisFernandoVelasquez.LFBotoneradeEfectos`.
  Publisher: `CN=AD90DE58-447F-47AE-AC1A-3D369955282B`.
  Windows Desktop desde 10.0.19041.0; cuatro idiomas; `runFullTrust` y `microphone`.
- Release Store, MakeAppx y generación de licencias: correctos. Se comprobó la
  ausencia de firma local y la coincidencia del ejecutable y las licencias
  dentro del MSIX. Partner Center validó el paquete y publicó la entrega.
- Última verificación de código: 349 tests Rust aprobados, 19 ignorados,
  cero fallidos; `cargo build --lib` y `npm run build` correctos. El único aviso
  MSVC corresponde a la creación de la biblioteca y el objeto de enlace.
- Los jobs Windows y Linux del run 38191218663 terminaron correctamente.

## Decisiones cerradas de 1.5.0

- Mic ofrece mantener pulsado, alternar y automático, con destinos combinables
  y nivel absoluto (60 → 20 → 60). Rust captura, mide y aplica la atenuación
  temporal conservando faders y master. Mic está entre Solo y Vol; clic derecho
  abre los ajustes. Automático requiere activación explícita; ante fallo de
  entrada restaura el nivel. No mezcla, graba ni transmite el micrófono.
- El medidor de ajustes tiene escucha temporal propia; al cerrar o cambiar a
  manual libera la entrada, salvo si automático sigue activo. Comparte captura
  con automático por nombre real de dispositivo, incluido el predeterminado.
  La resolución ocurre al elegir o abrir la entrada, sin enumeración periódica.
- El panel fijo restaura `visible`; se eliminó `show_on_start` y las dos casillas
  duplicadas. Principal conserva la opción de mostrar su botón superior.
- El volumen del reproductor se controla solo desde el botón de altavoz.
  Guardar otros ajustes ya no escribe ese volumen.
- Hora y fecha permanecen visibles; el contador de sonidos tiene su propio
  destino junto a Detener todo. Rust calcula el tiempo; no hay alternancia
  con el reloj ni temporizador de tres segundos.
- Los atajos opcionales del reproductor son por perfil y respetan `global_keys`.
  Rust valida conflictos y reutiliza su transporte. Nacen vacíos y se exportan
  como campos opcionales.
- Biblioteca y Buscador inician arrastre nativo de rutas con tauri-plugin-drag.
  fileDrop.js es el receptor único. Pestañas y reproductor admiten lotes;
  una celda o botón fijo exige una sola pista.
- Partner Center tiene novedades en los cuatro idiomas y privacidad con Mic
  en español e inglés. La página de privacidad del blog aún debe sincronizarse
  con PRIVACY.md; la política de la ficha Store se proporciona como texto.

## Pendientes

- Pruebas físicas del autor: Mic y captura compartida, atajos locales/globales,
  panel abierto/cerrado tras reiniciar y arrastre entre las dos ventanas.
- Comprobación MIDI en Linux y distintos canales Windows. No lanzar la aplicación
  desde la IA; las pruebas funcionales corresponden al autor.
- Issue #5: esperar confirmación del reportante en Debian/KDE antes de cerrarlo.
- La reorganización visual futura necesita plan y aprobación. No retomar la
  política de colores descartada ni la propuesta del issue #10 por ahora.
- No tocar `Capturas_Tienda/`: material ajeno a esta tarea.
