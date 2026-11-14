# Continuidad de sesión — estado actual


## Consola y atajo de pestaña — 2026-11-13

Petición nueva del autor: el botón de consola restaura/muestra/enfoca la ventana
existente, sin intentar crearla de nuevo ni cerrarla. consoleWindow.js ahora
espera getByLabel/unminimize/show/setFocus; mantiene modal/fallback y Android.
Importado de Danilo el menú «Asignar atajo de teclado…» con captureTab en
1d1f16c, conservando su autoría y comportamiento de teclado. MIDI se adapta al
modal existente con details plegado y visible solo si Rust informa disponible.
Se usa get_shortcut_editor_state y midi-devices-changed, sin nuevos campos/IPC.
Ocultar MIDI no borra asignaciones. Desconexión, plegado y cierre cancelan captura.
El modal acota tamaño y desplaza contenido para no perder acciones.
Guardar atajo no activa otra pestaña ni reproduce audio; valida conflictos
mediante las reglas existentes de teclado y MIDI. Créditos y changelog al día.
Detalles y pruebas en ATAJOS_PESTANA_Y_CONSOLA.md. Trabajo en variante-danilo;
no abrir la app ni compilar en GitHub. El usuario prueba con DEV.bat del worktree.

Validación local: 453 pruebas Rust aprobadas, 22 omitidas; cargo build --lib
correcto. Cinco pruebas de consola y dos pruebas DOM de pestaña/MIDI aprobadas;
npm run build correcto. El foco nativo y el dispositivo MIDI real quedan para
la prueba funcional del autor. No se han ejecutado compilaciones en GitHub.

## Ajustes de colores: modal, favoritos y reemplazo de sonido

El autor pidió adaptar el modal a la ventana, guardar combinaciones favoritas y
conservar colores al cambiar sonidos. Aprobó favoritos globales con nombre.
Implementados get_color_favorites, save_color_favorite, remove_color_favorite
y AppConfig.color_favorites opcional; contrato detallado en EDITOR_COLORES.md.
Favoritos es la segunda opción de Relleno; «Guardar combinación en favoritos»
debajo de la vista previa despliega nombre y guardado independiente del botón.
Eliminar un favorito no cambia botones y requiere confirmación en la app.
El modal tiene altura/ancho limitados por la ventana; cuerpo con scroll y
cabecera/acciones visibles. En ventanas estrechas las filas/acciones se adaptan.
Elegir archivo dentro del editor es un borrador hasta Aceptar, también al editar
una celda vacía. Cancelar conserva el audio anterior y no toca colores.
Asignaciones directas de audio/carpeta preservan los colores/estilo existentes;
new_button_preserving_colors sortea únicamente si no hay botón anterior.
No cambiar la semántica de la copia explícita desde el reproductor.
Traducciones en cuatro idiomas, documentación y changelog actualizados.
Validación final: 453 pruebas Rust aprobadas, 22 omitidas; cargo build --lib y
npm run build correctos. Diez pruebas DOM del editor aprobadas a 1366×900,
900×600, 640×420, 360×640 y 568×320 en claro/oscuro; dos de regresión del
buscador aprobadas. Comprobados guardado/reapertura/eliminación de favoritos,
colores al cambiar archivo/tipo, límites del modal, scroll y acciones visibles.
Prueba Rust con WAV real conserva colores/estilo al cambiar ruta y duración.
Se mantienen los créditos originales de Danilo separados de estas adaptaciones.
Trabajar en variante-danilo; no tocar main ni cambios pendientes de la raíz,
no abrir la app ni compilar en GitHub. El autor prueba desde DEV.bat del worktree.

## Editor de colores unificado — autorizado e implementado para prueba

El autor aprobó EDITOR_COLORES.md y podrá solicitar ajustes al probar.
Selector único: Predefinidos primero, sólido y cuatro direcciones de degradado.
Controles condicionales, HEX, segundo color y texto en degradado; todos los
cambios repintan la vista previa antes de aceptar. Los personalizados conservan
el color exacto; los predefinidos mantienen adaptación a tema.
ButtonData.style es opcional, omite el predeterminado y valida en Rust incluso
al deserializar. Edición de rejilla, panel fijo y OBS recibe style opcional;
cmd_fixed_update.rs separa la edición fija para respetar 200 líneas.
Exportación .bdelf/.bdeplf conserva style; versiones de LFA que lo ignoren no
conservan degradados en una ida y vuelta. Pintura múltiple vuelve al estilo
predefinido únicamente en los botones seleccionados. No se importa font_size.
Base original de Danilo importada en 75096df con su autoría; adaptaciones aparte.
Validación: 445 pruebas Rust aprobadas, 22 omitidas; cargo build --lib correcto.
Dos pruebas DOM aprobadas en tema claro/oscuro, incluyendo paleta, sólido,
cuatro degradados, HEX inválido, persistencia del borrador y Aceptar/Cancelar
en los editores reales de rejilla/panel con IPC simulado. npm run build correcto.
No se abrió la app ni se lanzaron compilaciones en GitHub; probar con DEV.bat
del worktree variante-danilo, reiniciando la sesión de desarrollo por el IPC.
Solo queda el ruteo de salidas por pestaña de la lista de integraciones.

## Buscador: clic izquierdo Danilo y derecho panel fijo — 2026-11-13

El autor autorizó integrar el buscador sin cambiar la interacción izquierda del
fork. Añadido el botón de lupa a la izquierda de pestañas y search_buttons en Rust.
Clic derecho en esa misma lupa: cerrar el flotante y abrir/fijar el panel existente
en view=search, visible=true, manteniendo las otras opciones y enfocando el campo.
No alternar el panel ni abrir otra Biblioteca; no cambiar transporte ni cola.
El clic izquierdo conserva búsqueda, resultados, navegación y resaltado de Danilo.
Procedencia, alcance y pruebas en BUSCADOR_BOTONES.md. Créditos añadidos a
CHANGELOG, README y avisos; colores y salidas por pestaña siguen pendientes.
No restaurar FFmpeg, no abrir la app ni compilar en GitHub. Trabajo exclusivamente
en el worktree variante-danilo; raíz/desarrollo y cambios previos conservados.
Archivos originales de Danilo importados en 758505b con su autoría; adaptación
separada para registro IPC, modelo oficial y clic derecho. Validación local:
5 pruebas Rust originales del buscador aprobadas; suite general 438 aprobadas,
22 omitidas; cargo build --lib y npm run build correctos. Dos pruebas DOM
aprobadas (768/1366 px), con cinco estados iniciales del panel y sin órdenes de audio.


## Estado vigente: sin conversión externa — 2026-11-13

El autor decidió retirar FFmpeg y todo el flujo de conversión. Eliminados módulo,
procesos, caché del producto, IPC/eventos, modal, porcentajes, scripts y binario
local preparado. Las secciones anteriores sobre conversión quedan históricas;
no restaurar esa función. No se borran cachés de datos personales ni originales.
Se conservan rueda sobre pestañas, barra inferior ocultable, File/SeekSource
para audio nativo compatible y la corrección del panic SeekError de Rodio 0.19.
MP4 con vídeo utiliza solo audio; sus canales/frecuencia vienen del bloque real.
Detector AAC conservado con crédito de Danilo para rechazar perfiles no admitidos
sin reproducir solo el núcleo. No hay fallback para HE-AAC/SBR/PS.
Ajustes → Acerca de muestra MP4 en los cuatro idiomas, también README/HTML.
Los créditos vigentes reconocen rueda, barra y detección AAC de Danilo.
La rama sigue en variante-danilo; raíz/desarrollo y cambios previos se preservan.
No abrir la app ni compilar en GitHub; el autor prueba con DEV.bat del worktree.
Validación: 433 pruebas Rust aprobadas, 22 omitidas; suite dirigida nativa con
13 aprobadas incluidas las tres opcionales. MP4 con vídeo y AAC-LC, AAC real
ADTS MPEG-2, seek/loop/análisis y rechazo de HE-AAC comprobados sin procesos
externos en las pruebas. Rueda y barra: 5 pruebas JS/DOM aprobadas; frontend
correcto. ADTS MPEG-2 adapta solo el ID de cabeceras indexadas al leer; payload
y originales intactos, sin crear copias de audio. Transporte, cue/ganancia,
vistas y caché de ondas comparados con dbdc75f: restaurados a la base inicial.


## Conversión: panic de apertura y seguimiento — 2026-11-13

El autor reportó «Audio thread died». El registro identifica el panic de
Rodio 0.19 (`decoder/symphonia.rs:44`, SeekError inesperado) y el PoisonError
secundario del monitor. La apertura MP4/M4A/AAC/M4B/3GP compatible usa ahora
File/SeekSource directamente, sin catch_unwind ni sustituir el error por silencio.
Inicializa frecuencia y canales a partir del primer bloque decodificado; un MP4
con vídeo selecciona solo audio. HE-AAC sigue usando FFmpeg como respaldo.
El autor solicitó modal ocultable y «Conversión x%» en botones: implementados
con progreso real de FFmpeg calculado por Rust, cola, estados indeterminados,
100 solo al validar y revisión de snapshots. Ocultar no cancela; acceso flotante
para reabrir. La lista oculta no se redibuja a cada actualización.
Validación: 435 pruebas Rust aprobadas, 24 omitidas; 16 pruebas de conversión
aprobadas incluidas opcionales, muestras reales en buses de memoria y MP4 con
vídeo/AAC-LC nativo. Dos pruebas DOM de modal/botones aprobadas (390/1366 px).
Sin abrir la aplicación, tocar sus datos, crear instaladores ni compilar en GitHub.
Reiniciar la sesión DEV.bat que sufrió el panic para cargar la corrección.


## Danilo: primera fase autorizada — 2026-11-13

El autor aprobó rueda de pestañas, barra inferior ocultable y conversión FFmpeg.
Implementadas en `variante-danilo`, worktree dentro de C:/OVERLAY/BOTONERA.
La aplicación se prueba manualmente con el DEV.bat de ese worktree; no ejecutar
el de la raíz esperando estos cambios. No abrir la botonera desde herramientas.
Sin compilaciones en GitHub, tags, cambios de versión ni instaladores.
Detector y pruebas originales importados en 7788c6b conservando a Danilo Wenner
como autor; ajustes posteriores en un commit separado del mantenedor.
Crédito de Danilo en CHANGELOG/README/avisos; procedencia y adaptaciones en
INTEGRACION_VARIANTE_DANILO.md. FFmpeg local 9.0.2 preparado desde el instalado
por el autor, excluido de Git; sin descarga automática ni empaquetado.
Ver CONVERSION_AUDIO.md. Validación local: 434 pruebas Rust aprobadas, 22
omitidas; 13 pruebas de conversión incluidas las 3 opcionales aprobadas;
frontend y prueba DOM de barra/apariencia correctos, 4 pruebas de rueda y
3 de notas de versión aprobadas. Sin prueba funcional manual de la botonera.
Caché FLAC compartida por reproducción y análisis;
Android conserva su ruta nativa. Colores, buscador y salidas por pestaña siguen
pendientes de trabajo individual. Conservar cambios de la raíz sin sobrescribir.


## Variante Danilo: entorno de integración — 2026-11-13

Rama de esta copia: `variante-danilo`, base oficial 1.6.0 en `3502072`.
Preparación inicial ya superada para las tres funciones autorizadas arriba.
Leer [INTEGRACION_VARIANTE_DANILO.md](INTEGRACION_VARIANTE_DANILO.md).
Seis aportaciones seleccionadas y crédito de Danilo reservados en CHANGELOG.
Buscador: izquierdo = botones; derecho = archivos del catálogo. El ruteo por
pestaña debe integrarse en consola, máster, Mic y medición. No fusionar el fork
completo ni cambiar la identidad oficial. La copia principal sigue en
`codex/desarrollo` con cambios sin commit que deben preservarse y conciliarse.
Este apartado prevalece para el worktree de la variante sobre la instrucción
histórica de trabajar exclusivamente en `codex/desarrollo`.

## Preparación estable multiplataforma y borrador GitHub — 2026-11-12

El autor autorizó explícitamente ESTABLE 1.6.0 para Windows/Linux, manteniendo
OBS y Control remoto en desarrollo. Android 1.6.0 pasa a versionCode 1006002.
Se restauran nombre, título, identificador y upgradeCode PC desde v1.5.0; main
no se mueve. Compilar en GitHub y reunir MSI/EXE/DEB/RPM/AppImage/APK firmado en
un RELEASE EN BORRADOR, sin publicarlo. No usar emulador Android.
Últimos ajustes: + del reproductor con estilo normal del tema y selección múltiple
nativa vía pick_audio_files; cola por player_add_tracks existente, sin sustituirla.
Invitación discreta al final de las novedades de PC para valorar Microsoft Store
y dar estrella en GitHub. Traducciones en los cuatro idiomas. En el borrador,
poner invitación después de descargas en ES/EN/pt-BR/pt-PT. 37 pruebas Node/DOM
aprobadas y frontend Windows compilado. Rust queda sujeto a los checks de CI.
Borrador fuente actualizado: Documentación/BORRADOR_RELEASE_1.6.0.md. El HTML
anterior estaba en D:/BOTONERA/Informacion para el usuario/Borrador-release-1.6.0
y describía Android como alfa; el autor permite mejorar y trasladar a GitHub.
No reutilizar NOTAS_RELEASE_GITHUB_SIGUIENTE.md: es histórico de 1.3.0.
Código enviado: 18f27b728302e73627d7f75ee1b510c9a2e85945.
Release DRAFT creado: ID 442932997, URL
https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/releases/tag/untagged-acd7d16faf618af8bded
No existe tag final v1.6.0 ni publicación. El autor ya puede leer/editar el borrador.
CI escritorio: 43220382889; Android sin emulador: 43220387765. En curso.
Estado persistido en D:/BOTONERA/Compilados/1.6.0/estado-compilacion.json.
Helper temporal de seguimiento/descarga/subida:
C:/Users/Streaming/AppData/Local/Temp/lf-release-assets.py (poll,
download-desktop RUN, register-android RUN, upload). No imprime credenciales.
CI terminado correctamente en las tres plataformas. Windows: 423 pruebas Rust
aprobadas, 19 omitidas; Linux: 425 aprobadas, 19 omitidas. Biblioteca sin tests y
empaquetado correctos en ambos. Android de producción sin emulador correcto.
Entrega local: D:/BOTONERA/Compilados/1.6.0. Seis instaladores canónicos más
SHA256SUMS.txt, archivos.json y metadatos de verificación. Los seis instaladores
y hashes ya están adjuntos al borrador; sigue draft=true y published_at=null.
MSI/EXE: LF Botonera de Efectos 1.6.0; UpgradeCode estable
43888972-c5a4-5e8d-a996-ca913f3b6d8e. DEB 1.6.0/amd64, RPM 1.6.0/x86_64,
AppImage ELF64/x86_64. APK 1.6.0/1006002/ARM64, no depuración, misma firma de
producción b64c372578ac7749c70742bc018f6cbc072d900ac943b9b5a94fca8a200a212f.
APK SHA256: 61d5c7c1c78cccc89a86aca2b4b08cf29d4fba9c1e62d938f5e9170ad67ef2d0.
Payload APK coincide con CI salvo firma. Eliminados APK anterior 1006001 y copia
intermedia: solo queda el APK final en Compilados. No se crearon tags finales,
no se publicó ni se movió main ni se envió una actualización a Microsoft Store.
La instalación/uso en los equipos del autor y MIDI físico siguen siendo manuales.
La copia HTML/Markdown del borrador en D: ya está sincronizada con el texto nuevo.

## Panel fijo: ayuda vacía y añadir canciones — 2026-11-12

El autor pidió texto en Botones fijos vacío: Arrastra aquí tus sonidos desde el
explorador de archivos, equivalente al aviso de canciones del reproductor.
Añadido con traducciones en los cuatro idiomas. El reproductor muestra un + azul
inmediatamente antes de Abrir playlist. Reutiliza pick_audio_file/player_add_track
para añadir una canción al final de la cola sin reemplazarla. La acción de importar
que solo estaba habilitada en Android ahora se usa también en PC. Cabecera del
reproductor permite ajuste de fila si el panel es estrecho. Tres pruebas Node
aprobadas: añadir, cancelar y error; sin compilar ni abrir la aplicación.
No ampliar el changelog de bienvenida con cada detalle menor de esta revisión.

## Aviso visual de Control remoto — 2026-11-12

El autor pidió el mismo aviso en OBS Studio: añadido encima de su casilla, fuera
del fieldset desactivable, con estilo compartido y traducciones en los cuatro idiomas.

Solicitud del autor: FUNCIÓN EN DESARROLLO al principio del panel principal de
Ajustes → Control remoto. Añadido fuera de las opciones desactivables, con color
de acento y traducciones es/en/pt-BR/pt-PT. No añadirlo como novedad al changelog.
Sin compilar ni iniciar la aplicación.

## MIDI de PC: sistema primero y editor según disponibilidad — 2026-11-12

El autor propuso vincular Bluetooth desde Windows/Linux y actualizar la lista,
retirando conexión directa solo si el sistema garantiza que aparece como MIDI.
Auditoría: Windows usa WinMM y Linux ALSA Sequencer; vincular Bluetooth no garantiza
que esos adaptadores reciban una entrada MIDI. Se consultaron fuentes primarias:
https://microsoft.github.io/MIDI/kb/api-back-compat/ y
https://pipewire.pages.freedesktop.org/wireplumber/daemon/configuration/bluetooth.html
Decisión implementada sin cambiar backend/IPC: mostrar en PC la orientación de
vincular en el sistema y usar Actualizar dispositivos MIDI; mantener la conexión
BLE directa plegada, rotulada Bluetooth MIDI: conexión alternativa. No retirarla
completamente sin comprobar el hardware y la compatibilidad de cada sistema.
El editor de botones de PC, incluidos OBS y botones fijos que comparten controles,
oculta la sección MIDI si get_shortcut_editor_state no informa una entrada activa
(MIDI apagado o ninguna entrada seleccionada/conectada). Teclado siempre visible.
Se conservan asignaciones al ocultar, se cancela captura y se actualiza en caliente.
Android conserva sus desplegables y conexión directa. Seis últimas pruebas Node/DOM
aprobadas, más dos de conexión MIDI; sin compilar, abrir la app ni empaquetar.
No extender la bienvenida con estos detalles: conservar sus cinco novedades breves.

## Bienvenida breve para el usuario final — 2026-11-12

El autor volvió a señalar que las notas contaban ajustes internos y resultaban
abrumadoras. Se consultó CHANGELOG.md del tag v1.5.0 directamente en GitHub y
se comparó el código actual con ese tag local. OBS, control remoto, BLE MIDI,
diseño Moderno y visibilidad de etiquetas son novedades frente a esa versión.
La bienvenida ahora muestra solo esas cinco funciones, con frases orientadas
al uso y unas 90 palabras. No narra casillas, controles atenuados, reconexiones,
IDs de dispositivos ni correcciones de funciones aún inéditas. Los detalles
técnicos permanecen en esta continuidad y los pendientes, fuera de la bienvenida.
Mantener este criterio para futuras ediciones: comparar con la última estable
publicada, priorizar beneficios y evitar convertir cada ajuste del desarrollo
en una novedad. Android conserva sus notas independientes. No compilar.

## Casillas y opciones dependientes como Precarga — 2026-11-11

El autor probó el DEV correcto y pidió: control remoto con casilla en lugar de
dos botones; eliminar avisos duplicados y simplificar el aviso de HTTP; atenuar
y bloquear controles dependientes en remoto/OBS/MIDI/Reproducción; Mapeo primero
y etiquetas después. Implementado con fieldsets disabled/inert y opacidad 0.45.
Remoto: marcar prepara los campos, Guardar activa el servidor y desmarcar lo
detiene inmediatamente. El estado del servidor lo sigue devolviendo Rust.
Reproducción bloquea las opciones de avance al desmarcar su casilla; los fundidos
son independientes de esa función. MIDI incluye búsqueda, dispositivos y captura.
Las notas Sin publicar describen novedades, sin presentar ajustes de funciones
inéditas como correcciones. Se añadió resumen de OBS a las novedades actuales.
34 pruebas Node/DOM aprobadas; repetición dirigida tras últimos cambios: 6/6.
Sin compilar ni abrir la app, sin emulador/Actions/instaladores; Rust pendiente de
compilar en el próximo dev del autor. DEV.bat consume un marcador temporal para
mostrar Qué hay de nuevo una vez en el próximo arranque. Marcador preparado en
TEMP de Streaming: LF-Botonera-Desarrollo-Novedades.once. Rust solo lo permite
en debug y una vez por proceso; no modifica perfiles ni reinicia el asistente.
También se puede repetir manualmente con DEV.bat novedades. El APK cerrado no
se recompiló. No iniciar desde la antigua raíz D:/BOTONERA.

## Evitar iniciar la copia antigua de desarrollo — 2026-11-11

El autor mostró una ventana 1.5.0 sin los cambios nuevos. Se comprobó el proceso:
D:/BOTONERA/src-tauri/target/debug/tauri-app.exe; Vite se ejecutaba desde
D:/BOTONERA/node_modules y ocupaba localhost:1420. Esa raíz conserva 1.5.0.
El DEV.bat vigente es C:/OVERLAY/BOTONERA/DEV.bat, proyecto 1.6.0-beta.5.
Cerrar la ventana y consola antiguas antes de iniciar el actual para liberar 1420.
El lanzador actual muestra carpeta y versión al abrirse. No se cerraron procesos
del usuario, no se lanzó otra instancia ni se compiló durante este diagnóstico.

## Control remoto optativo y aviso de cifrado — 2026-11-11

El autor pidió control remoto apagado por defecto y sin recursos de red activos
cuando no se usa, más aviso permanente al final sobre conexión no cifrada.
Auditoría: servidor PC ya era por sesión, sin inicio automático; start abre TCP
y crea el hilo, stop marca sesión inactiva, espera al hilo y descarta su clave.
Se mantiene ese mecanismo sin nueva preferencia/IPC. El aviso de HTTP sin cifrar
se sacó del bloque ocultable y queda al final, visible también con servidor apagado.
Cliente remoto: serde/default y JS pasan a false; preferencias explícitas existentes
se respetan. Apagar cierra el diálogo y corta el sondeo al momento, aun antes de
Guardar. Tres pruebas JS/DOM aprobadas; no compilar ni ejecutar emuladores.
Regresión Rust añadida para defaults y preferencia explícita, aún sin ejecutar.
Continuar prueba manual desde dev.bat. Android entregado no se recompiló.

## Ajustes de escritorio para dev.bat — 2026-11-11

El autor corrigió el inventario: editor con escalado y temas Predeterminado/Moderno
ya están probados; OBS y remoto también. Pruebas físicas generales quedan pendientes.
Pidió confirmar nombres Bluetooth MIDI en PC, completar las etiquetas y empezar
por Acerca de al final. Se aplicaron ajustes de orden, presentación de preferencias,
recuperación de controles remotos y mensajes de error de OBS/About. Auditoría del
código confirmó BLE MIDI y conservación de nombres para Windows/Linux; falta hardware.
Notas de PC/Android separadas, incluido historial alfa; párrafos reales probados en
390/768/1366px. Siete pruebas Node/DOM aprobadas; sintaxis/i18n/diff correctos.
El autor pidió NO compilar; se respetó: sin cargo test/build, npm build, Actions ni
instaladores. Probar personalmente desde dev.bat. Leer PENDIENTES_ESCRITORIO.md.
Las notas ya leídas no reaparecen automáticamente si se mantiene la misma versión;
no se alteró last_seen_version ni la configuración del usuario.

## Android cerrado; retomar escritorio — 2026-11-11

El autor confirmó listo el APK y cerró los cambios de la primera versión Android.
La entrega vigente es 1.6.0/1006001, única en Compilados; no recompilarla ahora.
Solicitó que Acerca de quede al final también en Windows/Linux y pidió recordar
pendientes y recomendaciones directamente en el chat, sin HTML informativo.
Ver PENDIENTES_ESCRITORIO.md para solicitudes confirmadas, comprobaciones y
recomendaciones aún no aprobadas. Este turno solo registra y explica prioridades;
no aplica cambios de interfaz ni genera instaladores de escritorio.

## Entrega única Android y verificación de producción — 2026-11-11

El autor informó que seguía viendo los cortes de notas y Acerca de antes
del control remoto. Solicitó retirar todos los APK anteriores y dejar solo
una entrega comprobada. Se conserva versión visible 1.6.0 y firma; se incrementa
versionCode a 1006001 para identificar inequívocamente la nueva instalación.
La revisión anterior probó esos detalles en debug; producción solo tenía
prueba de instalación/arranque. Se añade verificación nativa por accesibilidad
del APK release: viñeta completa sin párrafos partidos, Acerca de después de
Control remoto, Android/GitHub y Atrás. Se guardan XML y capturas de producción.
Control remoto Android inserta su pestaña antes de Acerca de desde su creación.
El retiro de APK se limita a los directorios de Compilados del proyecto; no
se borran SDK, emuladores, claves de firma ni instaladores de escritorio.
Retirados los 19 APK locales antiguos (incluidas copias intermedias) de
C:/OVERLAY/BOTONERA/Compilados y D:/BOTONERA/Compilados. Falta dejar solo el
nuevo APK firmado cuando pase producción. No copiar otra entrega antigua.
La primera prueba por accesibilidad falló al completar el primer inicio en el
AVD mínimo de 320x640; no demuestra un bloqueo de la aplicación. Diagnóstico
nativo confirmó IPC, configuración, idioma e interfaz inicializados; la captura
mostró el selector de precarga abierto por una pulsación de la automatización.
La prueba release ahora fija 1080x2400/densidad 420, registra cada pulsación y
captura antes de actuar. Se permite repetir solo producción después de comprobar
debug. El autor pidió después detener las pruebas en emulador por consumo de
tokens y probar personalmente en su teléfono. Se canceló 43146658641. No seguir
intentando automatizar la prueba funcional de esta entrega. Compilación sin
emulador: 43147367027, código 4ba1ce3. Verificar solo firma, identidad e integridad
y entregar un único APK; la prueba funcional queda a cargo del autor.
Entrega final firmada: D:/BOTONERA/Compilados/LF-Botonera-1.6.0-Android-1006001.apk.
GitHub 43147367027 completado correctamente, sin emulador. Versión 1.6.0,
versionCode 1006001, ARM64, identidad estable, sin depuración y misma firma
de producción. Las 937 entradas de aplicación coinciden con CI tras firmar.
SHA-256: baa4b7028dec74d302480821e815ffe0e172a4c58dec9771c550e062568a4765.
Comprobado: un único APK en ambos directorios Compilados; retirada también la
copia intermedia de esta entrega. No afirmar que pasó la prueba funcional:
el autor la hará en su teléfono. No se publicó ningún release.

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
