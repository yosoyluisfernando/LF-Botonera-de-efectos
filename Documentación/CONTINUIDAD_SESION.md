# Continuidad de sesión — estado actual

## Ajustes finales Android 1.6.0 antes de publicar — 2026-11-11

El autor confirmó que el APK estable se instala correctamente y solicitó solo
dos ajustes: notas de bienvenida sin cortes artificiales y Acerca de al final
de las secciones de ajustes Android. Se conserva versión 1.6.0/versionCode 1006000
y la misma firma de producción, porque aún no se ha publicado.
El visor estaba convirtiendo cada línea física del Markdown en un párrafo;
ahora une las continuaciones de cada párrafo/viñeta y conserva los encabezados,
separadores y formato en línea. El navegador decide el ajuste de línea por ancho.
Android mueve el botón de Acerca de al final sin alterar la navegación de escritorio.
Pruebas de regresión cubren notas a 390/768px, texto real Android y orden de ajustes.

Segunda revisión entregada en:
D:/BOTONERA/Compilados/Android-Estable-1.6.0-Revision2/LF-Botonera-1.6.0-android-arm64.apk.
GitHub Actions 43137025520 compiló 620c72790f1c1d939cfc366545ee5d2c93d3e4a2;
comprobación Android 35 y APK de producción correctos. El menú real también
comprueba Acerca de al final. Local: 422 Rust aprobadas, 19 omitidas; 30 pruebas
de interfaz aprobadas; biblioteca Rust y frontend compilan.
APK final SHA-256:
06b0809d525e650f5600e2c7becd2eaf206ec7e4db71d09eea83ad727972d0f5.
Mismo certificado de producción b64c372578ac7749c70742bc018f6cbc072d900ac943b9b5a94fca8a200a212f,
firma v2/v3 válida, identidad/versión/versionCode sin cambios y sin depuración.
La entrega anterior se conserva. El contenido coincide con CI salvo firma.
No se publicó ningún release. Al conservar versión, las notas ya leídas no
se abren automáticamente otra vez; no se borró ni alteró ese estado del usuario.

## Primera estable Android: alcance aprobado — 2026-11-11

El autor aprobó las propuestas de editor/etiquetas/notas y una estable Android
independiente de la Beta. Se prepara Android 1.6.0, paquete
io.github.yosoyluisfernando.lfbotonera y versionCode 1006000, con clave propia
de producción fuera de Git. Escritorio conserva versión/identidad Beta y main
la estable 1.5.0. Leer ANDROID_ESTABLE.md y PENDIENTES_ESCRITORIO.md.
La clave de producción y su respaldo están fuera del repositorio en D:/BOTONERA;
la documentación de firma explica recuperación y respaldo externo. Nunca subirlos.

About identifica Android y distingue GitHub actual de futuras instalaciones
Google Play/F-Droid mediante el instalador del sistema. Las tiendas administran
sus actualizaciones; GitHub filtra releases con archivos del sistema actual.
CHANGELOG.android.md contiene la presentación inicial y límites; escritorio
conserva CHANGELOG.md. La versión efectiva Android se obtiene de Tauri.

Atrás nativo usa el dispatcher Android: cierra superficies con sus manejadores
reales, ajustes: detalle → secciones → principal, panel fijo → principal y confirma
salida solo en principal; salir detiene reproducción y guarda historial.
El editor Android pliega teclado/MIDI sin dispositivo y habilita cada captura
según disponibilidad calculada por Rust. Dos preferencias globales permiten
ocultar sus etiquetas de rejilla independientemente; ambas visibles por defecto.

Esta preparación no publica un release ni registra Google Play/F-Droid.
Los pendientes de PC quedan en PENDIENTES_ESCRITORIO.md para el siguiente trabajo.

Entrega verificada: APK de producción 1.6.0 ARM64 en
D:/BOTONERA/Compilados/Android-Estable-1.6.0/LF-Botonera-1.6.0-android-arm64.apk.
GitHub Actions 43131269897 compiló df90c10daae840c7dc528ec0cf650d49508766a3,
pasó la comprobación funcional de Android 35 (incluido Atrás nativo real y
About Android/GitHub/1.6.0) y dos arranques del APK de producción sin depuración.
Local: 422 pruebas Rust aprobadas, 19 omitidas; biblioteca y frontend compilan;
28 pruebas Node/interfaz aprobadas. Solo el código de Android se empaquetó.
Firma propia de producción RSA 4096, certificado público SHA-256
b64c372578ac7749c70742bc018f6cbc072d900ac943b9b5a94fca8a200a212f.
SHA-256 APK final:
8127c5c170454368cd0e4394e13a91806cb478ca2f6fbdc91011731b9409bbde.
Se compararon todas las entradas del ZIP salvo los archivos de firma:
el contenido de aplicación coincide con el APK de CI. Firma válida v2/v3.
El directorio incluye notas, hashes, certificado público y procedencia;
Origen-GitHub es copia intermedia de auditoría, no la entrega pública firmada.
La clave privada y contraseña de recuperación permanecen fuera de Compilados y Git.
No se publicó ningún release ni se modificó main. El autor puede probar este APK;
el teclado Bluetooth de escritura continúa sin comprobación física.

## Prueba física Android y preparación pública — 2026-11-11

El autor confirmó conexión satisfactoria de Launchpad por Bluetooth MIDI
desde la propia aplicación Android beta.5. No se ha probado un teclado
Bluetooth de escritura: el autor no dispone de uno. La ayuda de búsqueda
explica que Dispositivos cercanos no activa Bluetooth: el usuario debe
encenderlo en Android y la app no lo activa automáticamente.
El autor pidió preparar la primera estable Android con una nota explícita
sobre apartados aún en desarrollo y posibles desajustes visuales en móviles
y tablets; changelog separado por plataforma, sin historial de correcciones
previo a su primera publicación Android. También pidió plegar los atajos
del editor Android según disponibilidad y poder ocultar las etiquetas de
teclado/MIDI en la rejilla, visibles por defecto en todas las plataformas.
La identidad y firma de producción Android deben concretarse antes de generar
un paquete estable; no se ha publicado ni cambiado el canal por esta nota.

## MIDI Android beta.5 — 2026-11-11

El autor aprobó priorizar MIDI en Android y plegar todos los atajos del teclado
al final. Ajustes → MIDI y teclado expone USB y Bluetooth sin desplegables.
Cada dispositivo permite conectar, desconectar/cancelar y reintentar; el
adaptador nativo comunica el estado por midi_devices.connection_state.
Los nombres BLE consultan las fuentes disponibles y conservan el nombre conocido.
Escritorio mantiene su distribución; comparte la conservación de nombres BLE.
No cambia el esquema de configuración ni se añaden dependencias.
El autor añadió detección de teclados de escritura Bluetooth: InputManager
lista teclados físicos externos por nombre; se abre Bluetooth del sistema para
buscar/vincular HID. Nuevos IPC keyboard_devices/keyboard_open_bluetooth_settings
y evento keyboard-devices-changed, sin cambiar configuración.
La prueba con controladores y teclados físicos corresponde al autor.

Verificado: 410 pruebas Rust aprobadas y 19 omitidas, cargo build --lib,
frontend y 25 pruebas Node/interfaz. GitHub Actions 43123719954 compiló
el commit 3a217c7951029faf4313db3e9874cd96cdc7c5d8 y pasó Android 35:
arranque, ajustes vertical/horizontal, consulta nativa de teclados, editor,
importación/exportación de pestaña/perfil y carpetas de locuciones.
El APK final está en D:/BOTONERA/Compilados/Pruebas-Android-MIDI-beta.5/,
con nombre LF-Botonera-1.6.0-beta.5-android-arm64.apk, SHA-256 y procedencia.
Se firmó con el certificado local de pruebas ya usado en Perfiles (SHA-256
2f17553a5b961c8e2fbd0bc868ed82d1ecf279086bd9f793bbe827381bfd3c58).
Se comprobó contenido idéntico al APK de GitHub tras la firma, identidad Beta,
versión interna, ARM64 y firma válida. La copia de origen queda en Origen-GitHub/.

## Canales de compilación y nombre Beta — 2026-11-11

El autor pidió documentar cómo compilar Beta y Estable para próximos chats.
Leer COMPILAR.md y CANALES_COMPILACION.md antes de generar paquetes.
El nombre visible de producto y ventana pasa a LF Botonera de Efectos Beta.
Se conservan el ID lanprueba y el upgradeCode de pruebas, para no migrar sus
identidades al renombrar. Android fija también el nombre Beta en su override.
La estable de escritorio debe recuperar nombre, ID y upgradeCode originales
verificados contra main; no basta quitar -beta de la versión. La primera
estable Android exige revisar ID definitivo, firma y versionCode; sigue en
pruebas. No se generaron instaladores ni se publicó nada para esta nota.

## Ajuste local del diálogo del editor — 2026-11-11

En C:/OVERLAY/BOTONERA se ajustó norm-config-modal para limitarse al viewport,
con cabecera/pie visibles y cuerpo desplazable. Respeta zonas seguras en móvil.
El formulario solicita foco antes de ajustar números con rueda; sin foco la
rueda puede desplazar las opciones. No cambia el resto de campos de la app.
Prueba de DOM con cuatro idiomas, tres tamaños, claro/oscuro, ambos diseños y
presentación Android; verifica acceso a la última opción y acciones visibles.
El autor pidió probarlo con DEV.bat, sin generar instaladores. La prueba manual sigue
pendiente de esa confirmación; los paquetes anteriores permanecen en D:.

## Ruta principal unificada — 2026-11-11

El autor aprobó el experimento y pidió continuar desde C:/OVERLAY/BOTONERA.
Esta es ahora la copia principal de codex/desarrollo, versión 1.6.0-beta.5,
con todo el código de Windows, Linux y Android del mismo repositorio.
D:/BOTONERA/Desarrollo-Unificado conserva la copia adicional sincronizada.
Las menciones anteriores a una copia principal temporal o a codex/obs-control
son históricas. Main publicado permanece sin cambios.

Antes de promoverla se respaldaron y verificaron 1.332 archivos del proyecto
anterior y su .git en D:/BOTONERA/Respaldos/. También se guardaron sus cambios
en un stash local. Se compararon los 824 archivos versionados de la copia
unificada; solo hubo diferencias de finales de línea entre las rutas.

Los compilados, informes, SDK, emulador, herramientas y cachés continúan en D:.
La verificación Rust utiliza D:/BOTONERA/app/.tools/msvc/entorno-msvc.cmd,
que mantiene CARGO_TARGET_DIR y temporales en D:. El código Android nativo vive
en src-tauri/android/; gen/android se genera con el flujo Android y sus scripts.
El traslado de ruta conserva el código probado y la identidad de la edición de
pruebas; futuras publicaciones y cambios en main requieren petición del autor.
DEV.bat conserva el call de npm para que la pausa muestre los errores de arranque.

## Moderno beta.4 — 2026-11-11

El autor amplió Moderno a Linux: Windows y Linux comparten selector, persistencia
y mejoras visuales. Android conserva su UI. Esta decisión sustituye los límites
anteriores a Windows. Capa Vite __LF_DESKTOP_THEMES__, Rust habilitado en ambos.
Textos de peligro legibles en todos los menús contextuales. Hover/pulsación de
rejilla y panel fijo mantienen estable la superficie del botón; prueba de mouse
real valida bordes claros/oscuros y detecta el efecto anterior. 18 pruebas Node.
Copia vigente D:/BOTONERA/Desarrollo-Unificado, rama codex/desarrollo.
El ejecutor Ubuntu falló al resolver azure.archive.ubuntu.com. La preparación
APT de release-builds usa fuentes oficiales Ubuntu con HTTPS, firmas normales
y reintentos acotados, sin depender del espejo Azure de la imagen del runner.

## Moderno beta.3 — 2026-11-11

Copia actual: D:/BOTONERA/Desarrollo-Unificado, rama codex/desarrollo.
La raíz de D:/BOTONERA y app/ son copias anteriores con cambios locales.
No sobrescribirlas al continuar. Perfil y pestaña activa usan morado; nombres
de perfiles legibles en claro/oscuro; dock con altura por contenido y separación
VU/controles. Colores personales conservados; Predeterminado los recupera.
Prueba de DOM ampliada con los módulos reales. Ver APARIENCIA_WINDOWS.md.

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
