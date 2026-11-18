# Salidas de audio por pestaña — escritorio

## Estado — 2026-11-18

Implementación local autorizada sobre `codex/desarrollo`. Sin commits ni envío a
GitHub. Antes de empezar, el autor autorizó igualar main y desarrollo, local y
remoto, en 407b374c85e8ebb091f9d41856949d3a082c0edb. Ese ajuste ya está cerrado.
Los cambios anteriores sin commit se conservan. El autor prueba con DEV.bat de
C:/OVERLAY/BOTONERA; no abrir la app ni lanzar builds en GitHub desde herramientas.

## Uso y comportamiento

Crear/editar una pestaña de audio permite elegir «Salida principal», la
predeterminada del sistema o una tarjeta concreta. Las pestañas OBS no muestran
el selector. Android lo oculta y conserva las asignaciones guardadas desde PC,
pero no aplica ruteo múltiple en su backend.

La elección conserva el nombre en PaletaData.audio_out y añade audio_out_id,
con serde(default) y omisión del vacío. El formato compartido añade audioOutId
opcional y conserva audioOut legible. LFA puede ignorar el ID; si lo elimina al
reexportar, se vuelve a resolver por nombre único cuando la tarjeta esté presente.
Una pestaña sin asignación o que elige la tarjeta principal mantiene el bus
Efectos. El alias default se resuelve al nombre físico antes de comparar destinos.

Hay hasta ocho destinos adicionales. Varias pestañas con la misma tarjeta
comparten un canal. Los carriles conservan su número mientras mantengan su
destino; su fader se recuerda por identidad de tarjeta en ConsoleConfig.output_faders
(mapa global con serde(default), vacío para configuraciones anteriores).

La consola muestra una tira por destino adicional, con nombre de tarjeta,
fader y vúmetro L/R posterior a las ganancias. En modo acoplado, los canales
adicionales habilitan desplazamiento horizontal para conservar el tamaño útil.

## Ganancia, Mic y medición

Cada destino sigue este recorrido real:

fuentes → mezcla del destino → fader propio + atenuación Mic → máster → medidor → tarjeta

El máster se aplica una sola vez, con el mismo valor vivo que Programa. Llegar
a cero silencia todos estos destinos y el boost conserva su límite autorizado.
Mic aplica a cada destino la selección «botonera principal», con control de
atenuación independiente por carril para no acelerar rampas compartidas entre
tarjetas. El cálculo del techo tiene en cuenta el máster.

El fader Efectos controla la mezcla que permanece en Efectos; los destinos
independientes tienen sus propios faders. Programa y su vúmetro siguen midiendo
la señal de Programa: las otras tarjetas no se suman a un medidor ficticio.
CUE sigue separado del máster y de Mic; panel fijo y reproductor conservan sus
rutas, faders y transporte. Las locuciones y carpetas aleatorias disparadas desde
una pestaña siguen la salida de esa pestaña.

## Tarjetas ausentes y reconexión

Una asignación ausente o que no logra abrirse queda en silencio y se informa en
un aviso persistente de la ventana principal. No se desvía contenido a Programa.
Si se supera el límite de ocho destinos, los excedentes también quedan bloqueados
y se informa. La asignación se conserva para elegir otra tarjeta al editar.

Un worker Rust reconcilia cada dos segundos sin depender de que la consola esté
abierta. La desconexión libera el endpoint; la reconexión intenta abrirlo de nuevo.
«Reintentar salidas» permite reabrir un dispositivo que está enumerado pero falló
al abrir. Un fallo estable no reconstruye continuamente el grafo ni interrumpe
el resto del audio en cada sondeo. Los alias y nombres físicos comparten endpoint.

Cada bus conserva su último destino aplicado y una revisión propia. La
reconciliación compara conexiones reales y solo retira/abre buses afectados.
Si Programa cambia de tarjeta, también se actualizan sus hijos; los destinos
independientes intactos se conservan. Añadir una salida no recrea Programa.

Cada ButtonState guarda el handle del bus que recibió su fuente. Al cambiar la
asignación de una pestaña, el motor compara ese handle con su destino actual:
conserva los estados, fuentes, rampas y relojes intactos; solo migra los sonidos
que cambiaron. El reproductor observa la revisión de su bus, no la generación
general, y conserva también su siguiente pista precargada ante cambios ajenos.

Cambiar la salida de una pestaña que está sonando puede producir un breve salto
solo en sus sonidos. Las locuciones encadenadas intactas continúan; las movidas
se detienen porque no admiten reposicionarse. Los sonidos de una tarjeta que se
desconecta se retiran, sin desviarlos a Principal ni reanudarlos inesperadamente.

## Identidad del dispositivo y compatibilidad

Windows obtiene el ID opaco de MMDevice mediante IMMDevice::GetId y conserva el
nombre visible por separado. Fuente primaria:
https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-immdevice-getid

Linux conserva la referencia PCM de ALSA; cuando contiene un índice numérico de
tarjeta se sustituye por el identificador simbólico de /proc/asound/cardN/id.
El alias Predeterminada sigue al sistema; no se fija a una tarjeta concreta.

Las asignaciones antiguas se migran solo si el nombre identifica una única
salida. Se migran también sus faders por nombre, conservando los valores anteriores.
Un ID conocido tiene prioridad: si desaparece, no se sustituye por otra tarjeta
con el mismo nombre. Un renombrado actualiza la etiqueta sin cambiar la ruta.
Una tarjeta antigua desconectada debe reconectarse o seleccionarse una vez para
poder aprender su identidad; no se puede deducir un ID de un nombre ausente.

CPAL 0.15 no expone la apertura directa por ID de MMDevice. Por eso se enlaza el
ID nativo con la salida CPAL solo si ambos catálogos tienen un nombre único. Dos
salidas con el mismo nombre quedan marcadas como ambiguas; hay que renombrarlas
en el sistema para abrirlas de forma inequívoca. No se emparejan por índice.

En Windows se declara directamente windows 0.54.0, ya presente en Cargo.lock y
usado por CPAL. Son los bindings oficiales de Microsoft, con licencia MIT OR
Apache-2.0 compatible; evita inventar interfaces COM o duplicar FFI manual.
No se añade un paquete ni una versión a la resolución. Solo se habilita acceso
a enumeración/propiedades de audio en Windows, sin servicios ni procesos externos.


## Contratos y módulos

- domain/console/tab_lanes.rs: reparto puro, estable y sin fallback a Programa.
- engine/console/tab_outputs.rs: resolución de alias, estado y worker.
- engine/console/tab_routes.rs: aplicación por lote y selección por ID de botón.
- graph_plan/graph_update: invalidación y actualización incremental de buses.
- device_catalog/device_windows/device_linux/device_migration: identidades y migración.
- get_tab_output_devices: descriptores {id, name, usable} para el selector.
- get_tab_output_status(retry?): lista de devices e issues con pestaña, tarjeta y
  reason (missing, unavailable, ambiguous o too_many). retry reabre solo los
  carriles fallidos.
- tab-outputs-changed: estado actualizado para avisos y tiras, en todas las ventanas.
- get_console_view: tiras salida0…salida7 con device (identidad), device_name (etiqueta), available y follows_master.
- set_bus_fader: admite esos canales y persiste por dispositivo, nunca en cue.
  El argumento device valida que el carril conserve su dueño: una tira antigua
  no puede mover el volumen de otra tarjeta tras cambiar de perfil.
- meter-tick/audio-tick: buses.salidas contiene ocho lecturas L/R/dB independientes.
- Cambios de pestaña, perfil, importación y deshacer aplican rutas antes de regresar;
  los comandos con consulta/apertura de tarjetas usan el worker IPC de Tauri.
- Seek y reconstrucción consultan la misma asignación que el disparo inicial.

JS solo representa el selector, los avisos y los medidores. No decide disponibilidad,
destinos alternativos, ganancias ni reconexión. No se añaden dependencias.

## Procedencia y limpieza

Basado en el reparto de carriles, pruebas y selección por botón de Danilo Wenner,
GPL-3.0-or-later, commit 7c1f1ae484cdb4bd0110b4761b8a221023bdea6f:
https://github.com/danilo-wenner-1/LF-Botonera-de-efectos

La adaptación oficial añade control de máster, Mic, medición visible, aislamiento
ante fallos, persistencia de faders e integración con Android/OBS y consola acoplada.
No se importa el fork completo ni su convertidor. Las referencias descargadas son
material de consulta temporal; la procedencia queda en este documento y los avisos.
Retirada el 2026-11-18 la copia limpia C:/OVERLAY/BOTONERA/.references/danilo
(57.237.476 bytes). El proyecto y sus pruebas ya no dependen de esa carpeta.

## Prueba funcional del autor

1. Reiniciar DEV.bat de la carpeta principal para cargar el nuevo backend.
2. Con dos salidas distintas de Windows/Linux, dejar una pestaña en Principal y
   asignar otra a la segunda tarjeta. Disparar ambas y comprobar sus destinos.
3. Asignar dos pestañas a esa segunda tarjeta: comparten una tira y fader.
4. Comprobar máster a cero, volumen normal y boost; Mic con la botonera principal
   seleccionada y deseleccionada. CUE debe seguir separado.
5. Comprobar los medidores de cada destino y Programa, saltos, loop, Solo,
   Detener otros, Stop general, carpetas aleatorias y locuciones.
6. Reproducir música y un efecto en Principal mientras se crea/edita otra pestaña:
   deben continuar sin salto. Cambiar una pestaña desde el mismo bus compartido
   y comprobar que solo migra su sonido. Comprobar también locuciones ajenas.
   Desconectar USB: aviso y silencio sin fuga a Principal. Reconectar:
   desaparece el aviso y un nuevo disparo vuelve a sonar por USB.
7. Reiniciar, exportar/importar y cambiar perfil: conservar elección y faders.
   Renombrar la tarjeta en Windows y comprobar que conserva su asignación; otra
   tarjeta con el mismo nombre no debe reemplazar un ID ausente.
8. Revisar consola modal, ventana y acoplada. En Android, editar sin borrar salida.

La verificación automática no sustituye estas pruebas con tarjetas físicas.

## Validación automática

cargo test --lib: 485 aprobadas, 23 omitidas, sin fallos. cargo build --lib y
npm run build completados. Las pruebas en memoria comprueban continuidad muestra
a muestra de Programa/reproductor al añadir un destino, conservación de buses
ajenos al editar/reintentar y preservación de estados de locuciones, panel y CUE.
También cubren renombrado, migración de faders, rechazo de reemplazos con igual
nombre, exportación/importación opcional y referencias simbólicas ALSA.

Ejecutada además la prueba opcional de enumeración MMDevice en este equipo:
IDs nativos y al menos una salida enlazada con CPAL confirmados, sin abrir streams.
Doce pruebas HTML/consola aprobadas: selector con ID/nombre, renombrado, ID ausente,
faders y ventanas, a 640/1366 px en claro/oscuro. Sin abrir la botonera.
El autor confirmó la integración inicial; queda la prueba auditiva de estas dos
mejoras con dispositivos físicos en Windows/Linux mediante DEV.bat.
