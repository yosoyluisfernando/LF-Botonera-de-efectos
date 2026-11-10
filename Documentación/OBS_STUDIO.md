# Integración con OBS Studio

Implementación iniciada el 2026-10-22. La integración experimental forma parte
de la beta 1.6.0-beta.1 de Windows y Linux.

## Respaldo y alcance autorizado

Antes de modificar código se creó en GitHub `respaldo/pre-obs-2026-10-22`, desde
`935d788e5d795c6bd6902f9f75cdb307e7aa389f`. El remoto `main` coincidía con ese commit.
El desarrollo local está en `codex/obs-control`; no subir cambios ni publicar
versiones posteriores sin una nueva petición del autor. `Capturas_Tienda/` es
material local ajeno al código y no forma parte del respaldo de Git.

Esta etapa incluye conexión, estado real, escenas a Programa/al aire o Vista previa,
selección de transición y duración opcional, paso de previa a programa, modo estudio,
inicio/parada/alternancia de grabación y transmisión, silencio de entradas de audio,
visibilidad de elementos, búfer de repetición, activación de filtros, pausa/reanudación
y división de grabación. No incluye macros de sonido + OBS, edición de parámetros
de filtros, cambios de volumen ni reproducción multimedia.

## Arquitectura: Núcleo + Motores

- `model/obs.rs`: preferencias globales `ObsConfig`, acción `ObsAction` y snapshot.
- `domain/obs/`: validación y órdenes; `resource_actions.rs` resuelve entradas,
  elementos y búfer. `button_state.rs` y `resource_states.rs` resuelven indicadores.
- `engine/obs/resources.rs`: consulta catálogos de audio, elementos de escenas/grupos
  y estado del búfer. `sync.rs` lo integra en cada reconciliación.
- `engine/obs/`: motor autónomo. La fachada recibe órdenes; un worker posee la
  conexión TCP/WebSocket, autentica, correlaciona respuestas y reconcilia estados.
- `core/state.rs`: contiene `ObsEngine`; `core/setup.rs` inicia su worker con un
  `Arc<Mutex<AppConfig>>`, sin entregar el `AppState` completo al motor.
- `ipc/cmd_obs.rs`: consulta, prueba, guardado de preferencias y botones; reutiliza
  las reglas de conflicto de teclado/MIDI y la validación de identificadores visuales.
- `settingsObs.js`, `obsEditor.js`, `obsView.js`: presentación y gestos. No abren
  sockets ni deciden el estado de OBS. `obsTargets.js` dibuja los selectores de
  entrada y escena/grupo/elemento. Toda comunicación pasa por `bridge/api.js`.
- `obs.css`: indicadores con custom properties, sin sustituir colores personales.

Deshacer/rehacer resincroniza el motor si restaura preferencias OBS diferentes.
No se comparte hilo, salida, temporizador ni transporte con los motores de audio.
Con la integración desactivada, el worker espera una configuración sin hacer red.
No hay descarga ni modificación del código fuente de OBS.

## Conexión y estado

Se usa obs-websocket 5.x, integrado en OBS Studio 28 o posterior. El usuario habilita
el servidor desde Herramientas en OBS. Por defecto: `127.0.0.1:4455`, autenticación
según lo que exija OBS. Esta etapa admite `ws://`, no añade TLS, descubrimiento de
servidores ni exposición del puerto en Internet. El control no transporta audio.

La autenticación usa el challenge SHA-256/Base64 oficial. La contraseña solo vive
en RAM durante la sesión: no entra en `AppConfig`, `get_config`, exports ni backups.
Guardar una contraseña nueva la sustituye; dejar el campo vacío conserva la de la
sesión. Al reiniciar la aplicación se debe introducir de nuevo si OBS la exige.
Probar conexión autentica una conexión temporal y no ejecuta acciones ni guarda el
borrador. Los cambios se aplican al guardar Ajustes generales.

Las escenas se presentan en el orden del panel Escenas de OBS, reconstruido por
`sceneIndex` descendente; la enumeración WebSocket llega invertida.
El motor consulta `GetVersion` para comprobar capacidades y obtiene escenas,
transiciones, modo estudio, grabación y transmisión. Escucha eventos y reconcilia
con consultas, también cada dos segundos para recuperar eventos que llegan durante
una solicitud. Emite `obs-state` solo cuando cambia la vista serializada. Los cambios
realizados desde OBS o desde otro controlador también llegan a la botonera.

Solo se admite una acción en curso. No hay cola de órdenes para cuando vuelva la
conexión y nunca se reintenta automáticamente una orden de control. Al reconfigurar,
una generación invalida los comandos pendientes. Una orden ya enviada puede haber
sido aceptada aunque se pierda su respuesta: tras timeout/desconexión se debe
comprobar OBS antes de repetir. La reconexión, si está habilitada, intenta de nuevo
cada tres segundos. Antes de alternar, se consulta el estado real.

Los tiempos TCP y de solicitud están acotados; la resolución DNS depende del sistema.
Las transiciones de duración fija no admiten ajuste. Si se selecciona una transición
fija con una duración solicitada, la selección puede completarse y el ajuste ser
rechazado; la interfaz informa del error. Dejar la duración vacía conserva la de OBS.

## Paletas y entradas

`PaletaData.obs`, con `#[serde(default)]`, distingue Sonidos y locuciones de Control
OBS Studio. El tipo se elige al crear y no se convierte después. Las paletas antiguas
son de sonidos. Nombre, filas, columnas, colores y redimensionado siguen disponibles.
Se pueden crear varias paletas OBS. La nueva opción requiere habilitar la integración.
Desactivarla conserva paletas y asignaciones, pero impide ejecutar o guardar acciones.

`ButtonData.obs: Option<ObsAction>` es opcional y se omite si no existe. Un control
usa `type = "obs"`, sin archivo, carpeta ni banderas de reproducción. Los controles combinados se llaman
Activar/desactivar modo estudio e Iniciar/detener grabación o transmisión; se
conservan también las acciones independientes y los identificadores existentes.
El nombre se completa con la escena/transición elegida o con la acción; sigue los
cambios de selección hasta que el usuario escribe un nombre propio.
Su editor usa
las opciones entregadas por Rust y listas consultadas a OBS; al editar desconectado
conserva la referencia configurada. No se infiere el éxito a partir de un clic.

Rust resuelve los estados visuales y JS los representa. Verde se reserva a escenas
al aire y amarillo a escenas en previa. Los controles de modo estudio usan borde
azul y texto «Modo estudio activado»; grabación, rojo y «Grabando»; transmisión,
cian y «Transmitiendo». Al desactivarse, el borde de actividad desaparece y el texto
indica modo estudio desactivado, grabación detenida o transmisión detenida. Esto
representa el estado real también en botones que solo inician o solo detienen.
La selección de transición antigua usa azul cuando coincide con la seleccionada.

Conservan además los estados desactivado, desconectado, pendiente, error y preparado.
Un error tiene borde discontinuo para distinguirlo del rojo continuo de grabación.
Los indicadores conservan el fondo y texto personales de la zona principal.
Ajustes → OBS → Vista de botones ofrece borde alrededor del botón (predeterminado),
franja inferior con texto contrastado o franja derecha estrecha con texto inferior.
Las franjas llevan separación neutra y muestran reposo en tono neutro. El borde
original solo muestra color cuando hay actividad. La selección múltiple mantiene
su aro. Los colores son custom properties con variantes para ambos temas.
El tick de audio no los modifica.

`ObsConfig.button_view: ObsButtonView` admite `border`, `bottom` y `right`, con
`#[serde(default)]` y valor predeterminado `border` para configuraciones anteriores.
`ObsView.button_view` comunica la preferencia junto con los estados. El módulo
`obsAppearance.js` construye muestras visuales usando el mismo CSS de la rejilla:
cambiar la selección actualiza solo la muestra; guardar aplica la vista a todas
las paletas OBS. No ejecuta órdenes ni necesita conexión para previsualizar.
Rust persiste la preferencia y solo reconfigura el motor si cambian los datos de
conexión o la contraseña. Deshacer/rehacer usa el mismo criterio de conexión.

Clic, teclado local/global y MIDI llegan al mismo despachador. Se mantiene el alcance
de paleta visible del perfil activo. Mantener una tecla local no repite acciones OBS.
Los atajos de pestaña permiten abrir una paleta antes de ejecutar sus controles.
No hay ejecución de botones OBS ocultos ni una nueva asignación MIDI global.

El panel fijo no admite OBS en esta etapa. El reproductor auxiliar queda excluido
permanentemente: rechaza los controles arrastrados y filtra tipos OBS al construir
su cola. Rust bloquea mezcla de paletas, asignación de archivos/Biblioteca a OBS y
movimientos entre controles OBS y panel fijo, incluidos intercambios.

## Transiciones rápidas de OBS

Para nuevos botones, «Transiciones rápidas» sustituye a «Seleccionar transición».
Consulta `GetHotkeyList` y ofrece exclusivamente las entradas reales
`OBSBasic.QuickTransition.<id>` de la colección cargada, ordenadas por número interno.
No se generan Corte, Mover ni otras opciones supuestas. Las rápidas son configurables
en OBS y su registro existe independientemente de que el usuario les asigne teclas.

`quick_transition` utiliza `TriggerHotkeyByName`: OBS ejecuta su transición rápida con
su duración y comportamiento originales. No se cambia la transición general ni se
reproduce el atajo con teclas del sistema. Rust exige modo estudio y comprueba que el
identificador siga disponible antes de enviarlo. Ocultar los controles visuales no
impide invocar el registro; fuera de modo estudio no se ejecuta.

Limitación del protocolo: `GetHotkeyList` entrega nombres internos, sin la descripción
traducida ni duración. La UI muestra «Transición rápida N» y explica esta limitación;
no adivina que un número corresponda a Corte o Mover. El usuario puede dar un nombre
propio al botón. Las referencias pertenecen a la colección actual: al cambiarla hay
que revisar sus correspondencias. Si no hay entradas, se configuran primero en OBS.

Los botones antiguos `transition_select` conservan su comportamiento de seleccionar
la transición y su duración opcional. Esa opción solo se muestra al editar un botón
que ya la usaba; no se convierte silenciosamente en una orden de salida al aire.
No cambian los comandos IPC ni el formato de `ObsAction`. El snapshot incorpora
`quick_transitions`; `obs-state` comunica esta lista junto al resto del estado.

Fuentes: [registro y ejecución en OBS](https://github.com/obsproject/obs-studio/blob/master/frontend/widgets/OBSBasic_Transitions.cpp)
y [consulta e invocación desde obs-websocket](https://github.com/obsproject/obs-websocket/blob/master/src/requesthandler/RequestHandler_General.cpp).

## Audio, visibilidad y repeticiones

El autor autorizó estas tres ampliaciones el 2026-10-22. No se añaden comandos IPC:
`obs_get` y `obs-state` amplían el snapshot con `inputs`, `scene_items` y
`replay_active: Option<bool>`. La interfaz no deduce estados a partir del clic.
Todos los disparos usan el mismo motor y guardan el alcance de paleta visible.

- `input_toggle`, `input_mute`, `input_unmute`: `SetInputMute` sobre una entrada.
  `GetInputList` más `GetInputMute` filtra entradas que no admiten audio. Silenciar
  afecta a esa entrada en todo OBS, no solo a una escena. Es independiente de Mic.
- `item_toggle`, `item_show`, `item_hide`: `SetSceneItemEnabled` sobre un elemento
  concreto. `ObsAction` añade `scene: String` y `scene_item_id: Option<i64>`, ambos
  con default y omitidos cuando están vacíos. `target` guarda el nombre de la fuente.
  La combinación contenedor + ID + nombre impide actuar sobre otro elemento si
  se reutiliza el ID después de borrarlo. Una referencia renombrada o retirada
  exige volver a seleccionarla; no se reasigna por nombre automáticamente.
- `replay_toggle`, `replay_start`, `replay_stop`, `replay_save`: inicio, parada y
  guardado del búfer. `None` significa no disponible; `Some(false)`, detenido;
  `Some(true)`, activo. Guardar requiere activo y nunca lo inicia implícitamente.
  La duración y la carpeta de los clips se configuran en OBS. La respuesta confirma
  la solicitud de guardado, no se inventa una confirmación de archivo en disco.

La enumeración de elementos recorre cada escena y sus grupos con un conjunto de
visitados; el ID solo tiene sentido dentro del contenedor. Se conserva el orden del
panel de fuentes por `sceneItemIndex` descendente. El usuario elige la escena o el
grupo propietario y el elemento; se muestra el ID para distinguir instancias.
La visibilidad es la propia del elemento. Un grupo padre oculto puede impedir que
un hijo se vea; mostrarlo no cambia Programa ni la visibilidad del padre.

La reconciliación reemplaza los catálogos completos para retirar referencias
obsoletas. Se consulta audio por entrada y elementos por contenedor; su coste crece
con la colección de OBS. No hay sondeo de vúmetros ni procesamiento de audio. Antes
de enviar una orden se reconcilia el estado real como en los controles anteriores.
`request_optional` omite exclusivamente códigos conocidos: 600 si un recurso se
eliminó durante el listado, 604 si una entrada no admite audio o el búfer no está
disponible. Otros rechazos y errores de transporte siguen siendo errores.
La ausencia de estas capacidades no impide usar escenas u otros controles.

Los estados incluyen audio activado/silenciado, elemento visible/oculto, destino
no disponible y búfer activo/detenido/no disponible/listo para guardar. Las tres
vistas representan estos estados; audio usa verde azulado o naranja al silenciar,
elemento visible azul y búfer activo violeta. Se conserva verde para Programa.

Referencias: [entradas de audio](https://github.com/obsproject/obs-websocket/blob/master/src/requesthandler/RequestHandler_Inputs.cpp),
[elementos de escena](https://github.com/obsproject/obs-websocket/blob/master/src/requesthandler/RequestHandler_SceneItems.cpp),
[búfer de repetición](https://github.com/obsproject/obs-websocket/blob/master/src/requesthandler/RequestHandler_Outputs.cpp)
y [códigos de error](https://github.com/obsproject/obs-websocket/blob/master/src/requesthandler/types/RequestStatus.h).

## Filtros, pausa y división de grabación (2026-10-26)

El autor autorizó estas tres funciones tras comprobar que no estaban implementadas.
La búsqueda previa en Rust, JS y traducciones no encontró sus comandos ni acciones.
Se reutilizan catálogo de acciones, IPC, editor, worker, teclado/MIDI y las tres vistas.
No hay un motor, transporte o comando IPC paralelo.

- `filter_toggle`, `filter_on`, `filter_off`: seleccionan fuente y filtro existente.
  `ObsAction.filter: String` tiene default y se omite vacío; `target` identifica la
  fuente. `ObsSnapshot.filters` contiene fuente, nombre y estado. `GetInputList` se
  consulta una sola vez por reconciliación y se comparte entre audio y filtros,
  incluidas entradas sin audio. `GetSourceFilterList` consulta cada fuente una vez,
  más escenas y fuentes/grupos presentes en el catálogo de elementos. El código
  dedicado vive en `engine/obs/filters.rs`; el dominio en `filter_record.rs`.
  `SetSourceFilterEnabled` afecta a todas las instancias de esa fuente. No crea
  filtros ni modifica sus parámetros. Retirar o renombrar el destino exige reasignar.
- `record_pause_toggle`, `record_pause`, `record_resume`: requieren grabación activa
  y usan PauseRecord/ResumeRecord según el estado consultado. `recording_paused`
  se obtiene junto a `recording` mediante la única consulta `GetRecordStatus`.
  Los botones anteriores de grabación también muestran En pausa. OBS puede aceptar
  una solicitud sin aplicarla si la salida no admite pausa: `engine/obs/record.rs`
  comprueba el estado real hasta diez veces, con 50 ms entre consultas, y si no se
  confirma informa del fallo. No se anuncia éxito basándose solo en la respuesta.
  Una grabación detenida nunca se representa como pausada.
- `record_split`: exige grabación activa y sin pausa y envía únicamente SplitRecordFile.
  No se simula con StopRecord + StartRecord. OBS debe ofrecer el método y tener
  una salida compatible con la división habilitada; un rechazo muestra un mensaje
  específico para revisar estos ajustes. No se cambia su configuración en secreto
  ni se reintenta la orden. La respuesta es aceptación de la división, no una
  comprobación de archivo escrito por la botonera.

`obsTargets.js` reutiliza el selector de propietario para Fuente al elegir filtros,
seguido del selector de filtro. Completa el nombre como Fuente · Filtro, conservando
nombres manuales y referencias guardadas al editar desconectado. Al cambiar a otra
acción limpia los campos que no corresponden. El estado de filtro activo es azul;
pausa, naranja; listo para dividir, rojo. Se conservan las variantes de tema y vistas.

Pruebas adicionales: unicidad de acciones, filtros homónimos en distintas fuentes,
validación/defaults de campos nuevos, pausa real/no confirmada, división rechazada,
órdenes exactas sin reiniciar, estados de botones anteriores y consulta compartida
sin duplicar GetInputList ni las fuentes. El servidor de pruebas es local simulado;
no se usa OBS real. Los selectores se verifican con DOM simulado.

Verificación del 2026-10-26: `npm run build`, igualdad y ausencia de duplicados de
904 claves i18n, DOM simulado y catálogo de 31 acciones únicas correctos. La nueva
suite Rust no llegó a ejecutarse: `cargo test --lib` falla al enlazar por falta de
`link.exe` en el entorno; `cargo build --lib` falla por la misma causa. La comprobación
`cargo check --lib --tests` se detiene por falta de `RC.EXE` del SDK de Windows.
No confundirlo con los resultados anteriores. Debe
repetirse la verificación ejecutable cuando esté disponible el enlazador C++.

Referencias oficiales: [filtros](https://github.com/obsproject/obs-websocket/blob/master/src/requesthandler/RequestHandler_Filters.cpp)
y [grabación](https://github.com/obsproject/obs-websocket/blob/master/src/requesthandler/RequestHandler_Record.cpp).

## Formatos compartidos

Los adaptadores `.bdelf` y `.bdeplf` preservan la marca opcional de paleta y la acción
opcional de botón. Los imports validan que no haya mezcla de tipos ni controles en
botones fijos antes de incorporarlos. No se exporta información de conexión.

Los archivos históricos siguen siendo legibles y hay prueba de ida/vuelta OBS dentro
de la Botonera. LF Automatizador no implementa estas acciones: no se garantiza que
una versión de LFA preserve los campos nuevos después de volver a guardar. No se ha
modificado ni verificado funcionalmente LFA. La compatibilidad estructural no equivale
a soporte de control OBS en la aplicación hermana. `.LFPlay` sigue siendo solo audio.

## Dependencias y mantenimiento

Se añade `tungstenite 0.28.0`, biblioteca WebSocket Rust con licencia MIT/Apache-2.0,
solo con `handshake`. Evita reimplementar enmascarado, fragmentación, control y límites
WebSocket. No se necesita un plugin de Tauri ni un runtime asíncrono adicional.
`sha2 0.10.9` (MIT/Apache-2.0) y `base64 0.22.1` (MIT/Apache-2.0) ya estaban resueltos
transitivamente y pasan a ser dependencias directas para autenticación oficial.

El lockfile añade seis paquetes: tungstenite, sha1, rand 0.9, rand_core 0.9,
rand_chacha 0.9 y ppv-lite86. Se conserva la línea 0.28 para este cambio; evaluar
actualizaciones por separado, sin actualizarlas automáticamente. Las licencias son
compatibles con GPL-3.0-or-later; no se copian fuentes de OBS. El coste es código de
red/handshake y aleatoriedad, sin bibliotecas nativas nuevas ni TLS adicional.
La superficie nueva es un cliente que procesa respuestas del servidor elegido:
mensajes/tramas limitados a 2 MiB, solicitudes identificadas y tiempos de espera.
El informe Rust se regenera desde `Cargo.lock` antes de distribuir.

## Verificación

Sin abrir la aplicación: `cargo test --lib`, `cargo build --lib` y `npm run build`.
Las pruebas OBS usan un servidor WebSocket local simulado, sin conectarse al OBS
real ni ejecutar grabación/transmisión. También cubren autenticación con un vector
calculado independientemente, reservas de órdenes, generaciones, defaults, formato
compartido y exclusión del panel fijo. Las pruebas nuevas cubren silencio real,
identidad compuesta de elementos, IDs reutilizados, exportación y catálogo con
entradas sin audio/grupos/búfer deshabilitado. Se comprobaron los selectores con DOM
simulado, sin lanzar la aplicación. Resultado anterior a la ampliación del 2026-10-26: 377 aprobadas, 19 ignoradas, cero
fallidas; ambas compilaciones correctas y claves i18n iguales en los cuatro idiomas.
Los módulos de código no superan 200 líneas.

Prueba funcional pendiente del autor en Windows y Linux:

1. Habilitar OBS, probar contraseña incorrecta/correcta, guardar y comprobar estado.
2. Crear paletas de cada tipo y comprobar que no mezclan contenido por edición/arrastre.
3. Enviar escenas a previa y programa, y pasar previa a programa con transición.
4. Cambiar estados desde OBS y comprobar su reflejo en los botones.
5. Probar grabación/transmisión, clics, teclado y MIDI con la paleta visible/oculta.
6. Cerrar/reabrir OBS; comprobar reconexión y ausencia de órdenes atrasadas.
7. Desactivar integración y comprobar que nada se ejecuta ni se pierde contenido.
8. Reiniciar la botonera y comprobar la solicitud de contraseña de sesión.
9. Exportar/importar paleta y perfil; probar redimensionado e indicadores en ambos temas.
10. Probar las tres vistas y su muestra, cerrar sin guardar, guardar y reiniciar;
    comprobar persistencia y que cambiar solo la vista no reconecta OBS.
11. Probar silencio desde OBS y botonera; revisar que no se ofrezcan imágenes sin audio.
12. Probar elementos con IDs repetidos en escenas/grupos diferentes, duplicados de
    una fuente, grupo oculto y eliminación/renombrado del destino.
13. Con el búfer deshabilitado, comprobar que siguen funcionando las otras acciones.
    Habilitarlo en OBS; iniciar, guardar, comprobar el clip en OBS y detener.
14. Probar filtros de audio/video, filtros homónimos en fuentes distintas y cambios
    desde OBS. Editar desconectado y revisar destinos retirados/renombrados.
15. Grabar, pausar/reanudar desde ambas aplicaciones y verificar los botones antiguos.
    Probar una configuración que no admita pausa: debe informar sin fingir éxito.
16. Probar división detenida/en pausa (rechazada), salida sin división habilitada
    y división compatible; comprobar archivos y continuidad en OBS.

Fuentes: [guía oficial](https://obsproject.com/kb/remote-control-guide) y
[protocolo oficial](https://github.com/obsproject/obs-websocket/blob/master/docs/generated/protocol.md).
