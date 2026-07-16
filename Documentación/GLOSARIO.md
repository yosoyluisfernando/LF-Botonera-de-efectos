# Glosario — LF Botonera de Efectos

Definiciones de todos los términos técnicos y conceptos propios del proyecto.

> **Documentos relacionados:**
> - Mapa completo del código → [`LIBRO_PROYECTO.md`](LIBRO_PROYECTO.md)
> - Arquitectura técnica → [`ARCHITECTURE.md`](ARCHITECTURE.md)
> - Guía para IAs → [`../AGENTS.md`](../AGENTS.md)

---

## A

**`api.js`**
Módulo JS que envuelve todas las llamadas a `window.__TAURI__`. Es el único punto de contacto entre el frontend y el backend. Exporta `invoke()`, `listen()`, `emit()` y `waitForTauri()`. El resto de módulos JS importan de aquí; ninguno accede directamente a `window.__TAURI__`.

**`AppConfig`**
Struct Rust (también JSON serializable) que representa la configuración completa de la aplicación: tema, idioma, perfiles, precarga, locuciones, caché persistente de waveform, etc. Se persiste en `botonera_config.json`. Ver [`config.rs`](../src-tauri/src/model/config.rs), que contiene las **preferencias**; el **contenido** que crea el usuario (perfil → paleta → botón) vive aparte, en [`content.rs`](../src-tauri/src/model/content.rs).

**`AppState`**
Objeto inyectado por Tauri en cada comando IPC. Contiene referencias a todo el estado compartido del backend: configuración, motor de audio, base de datos, caché de formas de onda, historial de reproducción. Ver [`state.rs`](../src-tauri/src/core/state.rs).

**`Arc<Mutex<T>>`**
Patrón Rust para compartir estado entre hilos. `Arc` es un contador de referencias atómico (compartir el puntero); `Mutex` garantiza exclusión mutua (solo un hilo accede a la vez). Se usa extensivamente en `AppState` para pasar el estado a los hilos de audio, monitor y flusher.

**`activar fila`**
Lo que hace el **doble clic** sobre una canción de la cola del reproductor: si está detenido, la reproduce; si algo está sonando, la **marca como siguiente** sin cortar la música. **La decisión la toma el motor, no la interfaz** (regla 4): el IPC es `player_activate_index` y la lógica, `QueueState::activate(index, is_playing)`. El `is_playing` lo aporta el hilo, que es quien conoce los decks, porque una [pista huérfana](#p) puede sonar sin estar ya en la cola. Un **clic simple no marca nada**: marcar sin querer al rozar una fila era problemático en directo.

**`analyze_track`**
Comando IPC (`cmd_tracks.rs`) que analiza una pista para el editor y devuelve: envolvente de onda, LUFS, pico en dBFS, ganancia sugerida, duración y metadatos de cue ya guardados. Delega en `engine::dsp::editor_analysis` mediante `spawn_blocking`, emite `track-analysis-progress`, reutiliza `TrackAnalysisCache`, `tracks.db` y caché persistente de waveform antes de decodificar el audio completo. Nunca corre en el hilo de audio.

**`audio-tick`**
Evento Tauri emitido por `engine/audio/monitor.rs` cada ~100 ms mientras hay audio reproduciéndose. Payload: `{buttons[], display_remaining, display_duration, master_level_l, master_level_r}`. Startup.js lo re-emite como `CustomEvent('lf-audio-tick')` en el DOM (distinto al evento Tauri).

**`AudioCommand`**
Enum Rust en `engine/audio/command.rs`. Variantes: `Play`, `Stop`, `StopAll`, `SetDevice`, `SetPreDevice`, `SetVolume`, `PlaySequence`. Se envía por un canal `mpsc` desde `AudioEngine` al hilo de audio (`engine/audio/thread.rs`).

**`AudioConfig`**
Struct de configuración de audio por perfil. Contiene los dispositivos de salida (`out_main`, `out_pre`), atajos globales, modo de reproducción y volumen master. Ver [`audio.rs`](../src-tauri/src/model/audio.rs).

**`AudioDeviceRuntime`** — *eliminado*
Struct que gestionaba el ciclo de vida de un `OutputStream` y su `MasterBus`. Desapareció al nacer [`engine/console/`](#c): las tarjetas son de la consola, no del motor de efectos. Su sustituto es [`OutputEndpoint`](#o).

**`AudioEngine`**
Fachada pública del motor de efectos en `engine/audio/engine.rs`. Posee el `Sender<AudioCommand>` hacia el hilo de audio y el `Preloader`. **Ya no posee tarjetas ni crea sus atómicos**: los niveles y el volumen del bus se los pide a la [consola](#c), porque son del bus y sobreviven a que la tarjeta se cambie. No toca rodio directamente.

---

## B

**`bdelf`**
Extensión de archivo para exportar una paleta (pestaña) de la Botonera. JSON compatible con el LF Automatizador. Puede contener el campo opcional `bdelf_tracks` con metadatos de cue y dB que el LFA ignora.

**`bdeplf`**
Extensión de archivo para exportar un perfil completo. Similar al `.bdelf` pero incluye todas las paletas del perfil.

**`botón`**
Celda de la rejilla a la que se asigna un archivo de audio (u otro tipo). Representado por `ButtonData` en Rust. Tiene id único, tipo, ruta, volumen, flags de comportamiento y atajo de teclado.

**`Bus`**
Struct Rust (`engine/console/bus.rs`). Un punto de suma **con nombre**: un `DynamicMixer` con un [`LevelSource`](#v) detrás que mide su pico, enchufado a un [`OutputEndpoint`](#o) con `play_raw`. Sustituye al antiguo `MasterBus`, del que se diferencia en que **no lleva `Sink`**.

Un bus es una **señal**, no una tarjeta. Esa distinción es la razón de ser de la [consola](#c): dos buses pueden salir por el mismo altavoz sin ser el mismo bus, porque se suman en el conector y no entre ellos. Se clona barato (solo son `Arc`) y se le añaden fuentes desde cualquier hilo.

**Trampa: soltar un `Bus` no lo retira de su destino.** A un mixer de rodio no se le quita una fuente; la única forma de sacar algo de ahí es que **se agote**. Por eso existe `close()`: cierra el grifo del bus (`BusOutlet` devuelve `None`) y el padre lo deja caer. `graph::rebuild` cierra los viejos antes de soltarlos.

Sin eso, un bus soltado sigue vivo dentro de la tarjeta. Y como los atómicos del medidor son del `BusSlot` y sobreviven a la reconstrucción, **el viejo y el nuevo escriben el mismo nivel a la vez** — el viejo, ya sin fuentes, escribiendo cero. El vúmetro parpadea. Fue un bug real (2026-07-16), lo cubre `bus_tests.rs`.

**`BusId`**
Enum de los buses de la consola (`domain/console/routing.rs`): `Efectos` (botones de la botonera), `Panel` (botones del panel fijo), `Cue` ([pre-escucha](#p)) y `Programa` (la suma de lo que va al aire). El bus `Reproductor` llega en la Fase 4.

`Programa` es especial: su fader **es** el volumen máster y su medidor **es** el vúmetro de la barra inferior. Que sean el mismo bus no es casualidad — es la única forma de que la aguja no mienta sobre lo que el fader controla.

**`BusSlot`**
Lo que un bus conserva **aunque su tarjeta no exista**: los atómicos de nivel y volumen. Se crean una vez en `ConsoleEngine::new()` y viven siempre, porque el monitor y las fuentes que ya suenan los tienen cogidos; un cambio de tarjeta no debe dejarlos apuntando a la nada. Que un `BusId` falte en `ConsoleState::live` significa exactamente "ese bus no existe ahora mismo".

**`ButtonData`**
Struct Rust (`model/content.rs`) que representa un botón. Campo `type_field` se serializa como `"type"` en JSON. El campo `vol` es un multiplicador lineal 0–1, no en dB (ver [trim](#trim)). Es el ladrillo común: lo usan la rejilla, el panel fijo y la cola del reproductor, por eso vive en el contenido y no cuelga de ningún módulo concreto.

**`ButtonSource`**
Struct Rust (`engine/audio/button.rs`) que implementa `Iterator<Item=f32>`. Es el **canal** de la consola: envuelve una fuente y aplica lo que es suyo — `muestra × file_gain × vol_botón × fade` — y nada más. Puede pararse por `stop_flag` o marcarse como terminado por `done_flag`.

**El master ya no está aquí** (desde la Fase 2): lo pone el [fader](#f) del [bus](#b), una sola vez sobre la suma. Antes cada fuente lo leía y se lo aplicaba a sí misma, lo que hacía que el "programa" no existiera en ningún punto del código.

**`ButtonState`**
Struct Rust (`engine/audio/button.rs`) que rastrea el estado de reproducción de una instancia de un botón: `start_time`, `duration`, `done_flag`, `stop_flag`, `volume`. Permite calcular `position()` y `remaining()` sin consultar el hilo de audio.

**`ButtonStateMap`**
`HashMap<String, Vec<ButtonState>>` — una entrada por id de botón, con un Vec para los modos `overlap` (varias instancias simultáneas del mismo botón). Ver [`button.rs`](../src-tauri/src/engine/audio/button.rs).

---

## C

**`consola` / `ConsoleEngine`**
El motor `engine/console/`: **dueño de las salidas físicas y de los buses**. No produce audio, lo recibe y lo encamina; por eso no es un motor *al lado* de `audio/` y `player/`, sino *debajo*: ambos son sus clientes.

**Por qué es un motor propio:** `OutputStream` no es `Send` (lleva un `cpal::Stream` dentro), así que alguien tiene que ser su dueño y quedarse quieto en un hilo. Cuando cada motor abría su propia tarjeta había *dos* hilos dueños de salidas, y por eso no podía existir un punto de suma común. La consola centraliza esa propiedad en un hilo guardián y reparte lo que sí viaja: `OutputStreamHandle` (`Clone + Send + Sync`) y el controller de cada [bus](#b) (`Arc`, `Send + Sync`).

**Reproducir no pasa por su hilo.** `DynamicMixerController::add` toma `&self`, así que cada motor añade fuentes a su bus desde el suyo. El hilo guardián solo atiende cambios de ruteo.

Plan y fases: [`PLAN_CONSOLA_VIRTUAL.md`](PLAN_CONSOLA_VIRTUAL.md).

**`CachedPcm`**
Struct en `engine/cache/cached_source.rs` que almacena el PCM decodificado como `Vec<i16>`, más la tasa de muestreo y el número de canales. Compartido via `Arc<CachedPcm>` entre la caché y las fuentes activas.

**`CachedSource`**
Struct en `engine/cache/cached_source.rs` que implementa `Source<Item=f32>`. Lee desde un `Arc<CachedPcm>` y permite crear instancias en cualquier posición con `new_at(pcm, offset)` en tiempo O(1). Solución al problema de seek con latencia creciente.

**`probe_duration_secs`**
Lee la duración de un archivo de audio (`engine/audio/formats.rs`). Devuelve `-1.0` si no puede.

**Trampa:** pide las propiedades **sin las etiquetas** (`read_tags(false)`), y es a propósito. Solo se necesita la duración, pero leer las etiquetas de paso hacía perder el archivo entero: un MP3 con el título o el artista mal codificados —frecuente en rippeos viejos con acentos— devolvía `TextDecode("Found invalid encoding")` y con él la duración, que no tiene nada que ver con el texto. El audio estaba perfecto. Los `-1` que quedaron guardados se recuperan al cargar (`config_migrate::recover_missing_durations`).

**Cuesta ~40 ms por archivo**, así que no se llama a la ligera: por eso la carga de una carpeta grande va en segundo plano, y la migración solo reintenta lo que está sin leer.

**`paleta de colores`**
Los 24 colores que la Botonera ofrece para los botones (`domain/palette.rs`). **Varían solo en MATIZ, y es a propósito.**

**La trampa:** `adapt_color` recorta para garantizar contraste en los dos temas (regla 8) — en oscuro la luminosidad no pasa de `0.30`, y en claro la saturación no baja de `0.90`. Eso deja el **matiz como lo único que sobrevive**: dos colores con el mismo matiz se ven **iguales**, por distinta que sea su base. La paleta anterior eran 16 matices de Material en dos intensidades (600 y 800): aparentaba 32 colores, pero el recorte igualaba cada pareja. Medido: 26 parejas por debajo de 12° de matiz (donde el ojo deja de separarlas), 6 azules, 6 rojos y **un solo verde**. Si algún día se añaden colores, deben separarse en **matiz**; variar la intensidad no sirve de nada.

El reparto no es a intervalos iguales: el ojo distingue mal entre verdes y entre azules (pasos de hasta 25°) y muy bien entre naranjas y amarillos (pasos de 12°). `colors_tests.rs` lo blinda: matices distintos, sin amontonar, distintos **tras adaptar a cada tema**, y con contraste suficiente.

**`cmd_*.rs`**
Convención de nombres para módulos en `src-tauri/src/ipc/` que contienen comandos IPC de Tauri. Cada `#[tauri::command]` se registra en `ipc/register.rs`. Estos módulos coordinan, no tienen lógica de negocio pesada.

**`clock-tick`**
Evento Tauri emitido por el hilo del reloj (`cmd_meta.rs`) cada segundo. Payload: `{time_str, date_str}`. El reloj formatea la hora según el idioma activo en Rust (sin reinicio al cambiar idioma).

**`cue`**
Punto de inicio (y opcionalmente de fin) fijado manualmente por el usuario en el editor de pistas. Permite saltar silencios iniciales o recortar un audio sin editar el archivo. El cue se aplica al construir la fuente en `engine/cache/preload.rs::build_play_source()`. Ver `cue_start_s`, `cue_end_s` en `TrackMeta`.

**`CuedSource`**
Struct en `cue_source.rs` que implementa seek saltando muestras (O(n)). Se usa cuando el archivo no está en la caché RAM. Para archivos cacheados se usa `CachedSource::new_at()` (O(1)).

---

## D

**`dBFS`**
*Decibeles relativos a la escala completa* (Full Scale). El valor 0 dBFS es el máximo sin distorsión. Los valores medidos son negativos (ej. −6 dBFS). El objetivo de pico en la normalización de la Botonera es −1 dBFS.

**`deck`**
Cada una de las dos "pletinas" del reproductor auxiliar (`engine/player/deck.rs`). Reproduce **una** pista a la vez, con estados `Empty`, `Loaded`, `Playing`, `Paused`, `Finished` y `Failed`. `Loaded` significa pre-cargado y en pausa, listo para arrancar al instante. `Finished` es fin natural a la espera de relevo.

`Failed` es "no se pudo cargar" (carpeta aleatoria vacía, sin clima, archivo ilegible): `poll_finished` lo trata como terminado para que el motor releve y **la música siga**, en vez de callarse esperando una pista que no existe. El motor alterna los dos decks en *ping-pong*. Concepto adaptado de `player.rs` del LF Automatizador 2.0.

**Ya no envuelve un `Sink` de rodio:** entrega su fuente al bus `Reproductor` de la [consola](#c), como el motor de efectos entrega las suyas. Lo que el `Sink` daba gratis lo sostiene ahora `DeckSource`.

**`DeckSource` / `DeckHandle`**
La fuente de un [deck](#d) dentro del bus, y el mando para controlarla desde fuera (`engine/player/source.rs`). Sustituyen al `Sink` de rodio, del que el reproductor dependía para tres cosas que una fuente metida en un mixer ya no puede "responder":

- **Posición** (`sink.get_pos`): se cuentan las muestras consumidas.
- **Terminó** (`sink.empty`): un flag que la fuente marca al agotarse.
- **Pausa** (`sink.pause`): un flag; en pausa devuelve silencio **sin pedir muestra** a la fuente, así que al reanudar sigue por donde iba. Devolver `None` la retiraría del mixer y pausar sería en realidad parar.

El volumen no está aquí: es el fader del bus. La fuente solo aplica la ganancia de **su** pista, que es del archivo y no del reproductor.

**`SeekSource`**
Fuente de audio (`engine/audio/seek_source.rs`) que abre un archivo **ya posicionado**, con salto de verdad. La usa `source_from_path_at`, así que sirve a los **dos** motores: efectos y reproductor.

**Por qué existe:** `rodio::Decoder` envuelve el lector en su `ReadSeekSource`, que informa `byte_len() = None`. Symphonia necesita el tamaño del archivo para posicionarse en formatos sin índice, así que su `try_seek` **fallaba siempre** (FLAC: `Unseekable`; MP3: `end of stream`) y se caía a `CuedSource`, que llega al punto **descartando las muestras una a una**: medido, ~55 ms por segundo saltado (6,6 s de silencio para ir al minuto 2). Solo no se notaba en los efectos, porque están en la caché de RAM y allí `CachedSource::new_at` ya era O(1). La solución es no pasar por rodio: symphonia acepta el `File` directamente y `File` **sí** implementa `MediaSource` informando del tamaño. Con eso, saltar cuesta ~10 ms **sea cual sea la distancia**.

**Trampa:** el salto cae al principio del **bloque que contiene** el punto, no al punto exacto (en FLAC, ~90 ms antes). Hay que descartar ese sobrante (`skip`), o se devuelve audio anterior al pedido.

**`decode_pcm`**
Función en `engine/cache/preload.rs` que decodifica un archivo a `Vec<i16>` para guardarlo en la `PreloadCache`. Solo se llama desde el hilo del `Preloader` o desde el análisis de pistas. Nunca desde el hilo de audio.

**`detener al finalizar`**
Interruptor del reproductor auxiliar: cuando está activo, al terminar la pista actual la siguiente **no arranca sola** hasta que se pulsa reproducir. Funciona en cualquier modo y **conserva** la pista marcada como siguiente. En el LF Automatizador 2.0 el concepto equivalente es `SilenceReason::StopAfterCurrent`. En Rust es el campo `stop_after` de `QueueState`; en el IPC, `player_set_stop_after`.

**`DynamicMixer`**
Componente de rodio que mezcla múltiples fuentes de audio en tiempo real. Cada [`Bus`](#b) usa uno para combinar sus fuentes en una sola señal. Su controller es `Arc<DynamicMixerController<f32>>` y `add()` toma `&self`: es `Send + Sync`, así que cualquier motor puede añadir fuentes a un bus desde su propio hilo sin pasar por la consola. `OutputStream` **también lleva uno dentro**, y por eso varios buses enchufados a la misma tarjeta se suman solos, en el conector.

---

## E

**`ebur128`**
Crate Rust que implementa la norma EBU R128 para medir loudness integrado (LUFS). Se usa en `engine/dsp/analysis.rs` para analizar el nivel perceptivo de un archivo y sugerir una ganancia de normalización.

**`effective_gain_db`**
Método de `TrackMeta`. Suma la ganancia del normalizador automático (`norm_gain_db`) y el ajuste manual del usuario (`gain_db`). Este valor se convierte a lineal y se pasa como `file_gain` al reproducir.

**`editor de pistas`**
Componente que permite al usuario ver la forma de onda de un archivo de audio, fijar puntos de cue (inicio/fin), ajustar el volumen en dB y normalizar el audio. Puede abrirse en modal (dentro de la app) o en ventana flotante (pop-out). Ver `trackEditor.js`, `waveformCanvas.js`, `cmd_tracks.rs`.

**`envolvente`** (de onda)
Representación de la forma de onda como pares (mínimo, máximo) por segmento de tiempo. No es el PCM completo, sino una versión comprimida para visualización. Se calcula en `audio_analysis.rs` y se almacena en `waveform::WaveEnvelope`. El frontend la dibuja en `waveformCanvas.js`.

**`domain/export/tracks.rs`**
Módulo actual `domain/export/tracks.rs`. Inyecta el campo `bdelf_tracks` al exportar (cue+dB por archivo) y lo restaura al importar, escribiendo en `tracks.db`. Permite que los ajustes del editor de pistas viajen con el archivo `.bdelf`.

---

## F

**`FaderSource`**
Struct Rust (`engine/console/fader.rs`). El **fader de un bus**: multiplica su señal por un atómico que se puede mover mientras suena. Parece trivial y es el punto entero de la Fase 2 de la [consola](#c): convierte el master en **una etapa real**, por la que pasa toda la señal del bus una sola vez, en lugar de un número que cada fuente leía y se aplicaba a sí misma.

Va **antes** del [`LevelSource`](#v), para que el vúmetro enseñe lo que de verdad sale. **No tiene rampa**, a propósito: el master ya saltaba de golpe antes, y suavizarlo en la misma fase que mueve la aritmética de sitio impediría saber qué causó qué.

**`file_gain`**
Ganancia por archivo en multiplicador lineal. Representa la ganancia total calculada por el editor de pistas: `10^((norm_gain_db + gain_db) / 20)`. Es lo primero que se aplica en el canal. Ver `TrackMeta::effective_gain_linear()`.

**`flush`**
Volcar el buffer en memoria de `last_played` a `tracks.db`. Ocurre cada 30 s (debounce del flusher) y al cerrar la ventana. Así se evita escribir en SQLite en cada reproducción individual.

**`FullProfile`**
Estrategia de precarga que carga en RAM todos los audios cortos del perfil activo al arrancar la app. Una de las tres opciones de `PreloadStrategy`. Ver [precarga](#precarga-de-audio).

---

## G

**`gain_db`**
Campo de `TrackMeta`. Ajuste manual de volumen en dB que el usuario fija en el editor de pistas. Se suma a `norm_gain_db` para obtener la ganancia total del archivo.

**`get_config`**
Comando IPC principal del arranque. Devuelve el `AppConfig` completo al frontend. Es la primera llamada que hace `startup.js`.

**`get_grid_state`**
Comando IPC que devuelve la rejilla (paleta activa): filas, columnas y todos los botones con su estado actual. El frontend lo usa para dibujar la pantalla.

---

## I

**`i18n`**
*Internacionalización*. El sistema usa archivos JSON en `src/public/i18n/` (es, en, pt-BR, pt-PT). `es.json` es la fuente de verdad. Función `t(key)` en `i18n.js`. El HTML usa atributo `data-i18n="clave"` para actualizarse automáticamente.

**`id de botón`**
Identificador único en formato `{paleta_id}_btn_{index}`. Ejemplo: `paleta_1_btn_3`. Antes del formato actual se usaba `btn_{index}`, que colisionaba entre paletas; `engine/persist/config_io.rs` migra automáticamente.

**`invoke`**
Función de `api.js` que llama un comando Rust por IPC. Equivale a `window.__TAURI__.core.invoke(cmd, args)`. Retorna una `Promise` con el resultado.

**`IPC`**
*Inter-Process Communication*. En Tauri, el mecanismo por el que el frontend JS llama funciones en el backend Rust (`invoke`) y recibe eventos desde Rust (`listen`). La capa de abstracción está en `api.js`.

**`is_first_boot`**
Campo de `AppConfig`. `true` mientras el usuario no ha completado el wizard de primer arranque. Al terminar el wizard, `cmd_profiles::set_first_boot_complete()` lo pone a `false`.

---

## J

**`jitter`**
Variación en la latencia de reproducción causada por el acceso al disco al decodificar el audio en el momento de la pulsación. Se elimina con la [precarga de audio](#precarga-de-audio).

---

## L

**`last_played`**
Módulo (`last_played.rs`) que mantiene en memoria un buffer de rutas reproducidas con su epoch. El flusher lo vuelca a `tracks.db` cada 30 s. Se usa para la estrategia de precarga `OnPlay` (recalentar lo reproducido recientemente).

**`LFA`** / **LF Automatizador**
Aplicación hermana desarrollada por el mismo autor. Automatizador de radio que comparte los formatos de archivo `.bdelf`/`.bdeplf` con la Botonera. La instalación local está en `C:\LF Automatizador v1.0`.

**`domain/export/lfa_format/`**
Subdirectorio en `src-tauri/src/domain/export/` con tres módulos (`types.rs`, `paleta.rs`, `profile.rs`) que implementan la conversión entre los tipos internos de la Botonera y el formato JSON del LFA.

**`.LFPlay`**
Formato de lista de reproducción del LF Automatizador, que la Botonera **también** guarda y abre desde el reproductor auxiliar. Es un array JSON de filas `{ruta, titulo, duracion, type, target}`. La conversión vive en `domain/export/lfa_format/playlist.rs`. Las filas de automatización del LFA (notas, saltos, ejecutar evento) y las emisoras por URL (`type: "stream"`) no aplican aquí: `from_lfa_row` devuelve `None` y se ignoran al importar. Los campos propios del LFA que no declaramos (pisadores, `customMix`, `eventId`, `temp`…) los descarta serde solo, sin configurar nada.

**Trampa de la duración:** puede venir como `duracion` o `duration`, y como número (`172`) o como **cadena** (`"31"`), según la versión del LFA que guardó la lista — por eso el suyo la lee con `parseInt`. `flexible_secs` acepta todas las variantes, y una duración ilegible vale `0` en vez de hacer fallar la fila: si una sola fila revienta al deserializar, **se pierde el archivo entero**, y vale más una canción sin duración que perder la lista.

**Trampa de las locuciones:** en una fila de hora o clima, `ruta` **no es una carpeta**: es un [marcador de locución](#m). Tomarlo por carpeta hacía buscar un directorio llamado `time_locution` y la locución no sonaba nunca.

**`marcador de locución`**
Los valores `time_locution`, `temperature_locution` y `humidity_locution` que el LF Automatizador escribe en el campo `ruta` de una fila de locución. No son rutas: son una **etiqueta de tipo**. Ninguna de las dos aplicaciones guarda ahí una carpeta, porque **cada una resuelve la locución con SUS propias carpetas**: es lo que hace que una lista creada en el LFA suene aquí y viceversa. Al importar, `from_lfa_row` los reconoce y deja la carpeta **vacía** (así manda la de Ajustes); al exportar, `to_lfa_row` escribe el marcador si la fila no tiene carpeta propia, porque una `ruta` vacía le dejaría al LFA una fila que no sabría resolver. Las configuraciones que ya los guardaron como carpeta se limpian solas al cargar (`config_io::clear_locution_markers`).

**`lf-audio-tick`**
`CustomEvent` del DOM (no evento Tauri) que `startup.js` dispara cada vez que llega un `"audio-tick"` de Rust. Los módulos que necesitan escuchar el progreso de audio (como `trackTransport.js`) usan `window.addEventListener('lf-audio-tick', ...)`.

**`listen`**
Función de `api.js` que suscribe un handler a un evento emitido por Rust. Equivale a `window.__TAURI__.event.listen(event, handler)`.

**`locución`**
Archivo de audio que representa un valor de texto (una hora, una temperatura, un número). Los botones de tipo `time`, `temperature` y `humidity` construyen una secuencia de locuciones y la reproducen en orden. El patrón de nombre de archivo lo define `engine/weather/resolver.rs`.

**`loop_mode`**
Flag de `ButtonData`. Si está activo, el archivo se reproduce en bucle infinito hasta que el usuario lo para. El bucle repite la región entre `cue_start_s` y `cue_end_s` (si están definidos).

**`LRU`**
*Least Recently Used*. Política de expulsión de la caché: cuando se supera el presupuesto de RAM, se elimina el elemento que lleva más tiempo sin usarse. Implementado en `PreloadCache` con una `VecDeque` como cola de prioridad.

**`LUFS`**
*Loudness Units relative to Full Scale*. Unidad de medida de loudness integrado según la norma EBU R128. El objetivo de la normalización automática de la Botonera es −14 LUFS, lo que equivale a una sonoridad similar a la de las plataformas de streaming. Se mide con el crate `ebur128`.

---

## M

**`marcar siguiente`**
Señalar qué pista de la cola sonará a continuación en el reproductor auxiliar. Se pinta en **naranja**. Es **ley**: si hay una pista marcada, se respeta siempre, sin importar el modo de avance. La marca sigue a **su canción**, no a la posición: al reordenar la lista se conserva por `id`, y si esa canción se borra, la marca se cae (no salta a otra). En Rust es el campo `marked` de `QueueState`, y la regla vive en `domain/player/advance.rs`. En el IPC, `player_mark_next`.

El naranja es además la **guía de qué viene**, así que no desaparece con el reproductor detenido: al pulsar Stop, lo que estaba pre-cargado pasa a marcado, y si no hay nada marcado se calcula lo que sonaría al pulsar reproducir (`ensure_upcoming_marked`, en `engine/player/queue_select.rs`). De ahí sale también que, al añadir canciones a una lista vacía, la primera quede marcada sola: el invariante es "**detenido y con cola ⇒ hay naranja**".

**`master_volume`**
Volumen global del perfil activo. Rango 0–1 (o 0–1.5 en modo boost). Es la tercera capa del modelo de ganancia. Se aplica atómicamente en cada muestra dentro de `ButtonSource`.

**`MasterBus`** — *eliminado*
Combinaba un `DynamicMixer<f32>`, un `LevelSource` y un `Sink` en un solo objeto. Ese era justo el problema: fundía la *señal* con el *conector*. Su sustituto es [`Bus`](#b), que es lo mismo **menos el `Sink`** y se enchufa a un [`OutputEndpoint`](#o) con `play_raw`. Un bus nunca se pausa, así que la capa de control del `Sink` sobraba.

**`modo de avance`**
Cómo recorre la cola el reproductor auxiliar. Es un modo **de lista**, no de un botón. Tres valores: `normal` (recorre y se detiene al final), `repeat` (da la vuelta) y `random` (al azar, evitando repetir la actual). El modo dice **qué** pista viene; que el reproductor se pare al acabar lo decide **detener al finalizar**, que es un interruptor aparte y se combina con los tres. Por encima del modo manda además **marcar siguiente**. La regla pura es `domain::player::next_index`.

**Hubo un cuarto modo, `manual`**, que no avanzaba solo. Se quitó porque duplicaba el interruptor **y además limitaba**: para elegir la siguiente forzaba el orden normal, así que "manual + aleatorio" era imposible. Con el interruptor, cualquier combinación funciona (por ejemplo, pararse en cada pista y que la siguiente salga al azar). Una configuración antigua con `manual` se migra a `normal` al cargar (`config_io::normalize_playback_modes`); lo de "no avanzar solo" lo da ahora el botón, que **no** se persiste, igual que el Loop.

**`modo reproductor`**
Una de las dos presentaciones del panel lateral (`fixed_panel.view = "player"`); la otra es `"buttons"`. Muestra el reproductor auxiliar con su lista. Sustituyó a la antigua vista `"list"`, que enseñaba los botones fijos en lista compacta; las configuraciones antiguas se migran solas en `cmd_fixed_panel::state`.

**`mtime`**
*Modification time*. Fecha de modificación del archivo en época Unix (segundos). La Botonera usa `mtime` junto con `size` para detectar si un archivo fue reemplazado y así invalidar la fila correspondiente en `tracks.db`.

---

## N

**`normalize_key`**
Función en `db.rs` que normaliza una ruta de archivo para usarla como clave en SQLite. En Windows convierte a minúsculas (sistema de archivos insensible a mayúsculas); en Linux la deja tal cual.

**`norm_gain_db`**
Ganancia calculada por el normalizador automático para llevar el audio a −14 LUFS sin superar −1 dBFS. Se guarda en `TrackMeta`. El usuario puede activarla o desactivarla con `norm_enabled`.

---

## O

**`OnPlay`**
Estrategia de precarga que encola un archivo en la caché RAM justo cuando se reproduce por primera vez. En las siguientes reproducciones, el archivo ya está en caché y el seek es O(1).

**`OutputEndpoint`**
Struct Rust (`engine/console/endpoint.rs`). Una tarjeta física abierta **exactamente una vez**. Un `EndpointRegistry` las indexa por nombre, así que pedir dos veces la misma tarjeta devuelve la misma: eso es lo que impide que dos motores la abran por separado y dejen que el sistema operativo sume sus flujos fuera de nuestro control.

Un endpoint **no construye ningún mixer**: `OutputStream` ya lleva uno dentro (`play_raw` añade a él). Solo mantiene el stream vivo —si se suelta, el `Weak` del handle deja de resolver— y reparte su handle. Un endpoint es un **conector**; lo que lleva señal es el [bus](#b).

**`open-meteo`**
API de clima gratuita y de código abierto usada por `engine/weather/client.rs` para obtener temperatura y humedad actuales a partir de coordenadas geográficas. La caché es de 10 minutos.

**`overlap`**
Flag de `ButtonData`. Si está activo, reproducir el botón mientras ya está sonando crea una nueva instancia simultánea en lugar de pararlo o ignorarlo.

---

## P

**`paleta`**
Pestaña de la rejilla de botones. Un perfil puede tener múltiples paletas. Cada paleta tiene su propia cuadrícula (filas × columnas), nombre, colores y opcionalmente un dispositivo de audio independiente. Representada por `PaletaData`.

**`PaletaData`**
Struct Rust (`model/content.rs`) que representa una paleta. Su id tiene el formato `"paleta_1"`, `"paleta_2"`, etc.

**Trampa:** su campo `audio_out` **no enruta nada**. Existe solo por compatibilidad con LF Automatizador (se guarda y se exporta en `.bdelf`), pero el motor de la Botonera lo ignora: todo suena por el `out_main` del perfil. Hoy solo hay dos buses de efectos (`BusId::Main` y `BusId::Pre` en [`engine/console/`](#c)), elegidos con el booleano `to_pre`.

**`perfil`**
Configuración raíz que agrupa una o más paletas. Un perfil tiene nombre, colores, ajustes de audio (dispositivos, atajos, modo de reproducción) y una lista de paletas. Representado por `ProfileData`.

**`panel fijo`**
Panel lateral persistente e independiente de la pestaña activa. Puede mostrar una colección global compartida por todos los perfiles o una colección propia de cada perfil. Su presentación es **`buttons`** (botones fijos) o **`player`** ([modo reproductor](#m)); se coloca a izquierda o derecha. Los botones específicos de perfil viajan en `.bdeplf`; los globales permanecen en la configuración de la aplicación, porque no pertenecen a ningún perfil.
Tiene sus propios modos Normal, Loop, Multi, Reset y Solo. Las operaciones Solo,
Detener otros y Stop del panel no detienen fuentes de la botonera principal.
Puede usar filas ilimitadas con desplazamiento o limitar su capacidad a
`columnas × filas` (hasta cinco columnas y veinte filas), sin eliminar contenido existente.

**Columnas y Filas son la CAPACIDAD de la rejilla de botones fijos**, no opciones del reproductor: `columnas × filas` limita cuántos botones caben (`cmd_fixed_panel::set_fixed_panel_settings`). En la presentación `player` no pintan nada y por eso se ocultan en Ajustes; la cola tiene desplazamiento propio.

**El panel fijo no tiene salida de audio propia** (decisión del autor, 2026-07-16): sus botones suenan por el motor de efectos con el `out_main` del perfil. `PlaybackGroup::Fixed` solo agrupa (Solo, Detener otros), **no enruta**. El [reproductor](#r) sí tiene salida propia, porque es un motor aparte.

**El alcance es de toda la barra**, no de cada botón: mezclar colecciones por botón sería más difícil de entender. Al cambiarlo, la colección anterior **se conserva guardada y oculta**; solo se borra si el usuario lo confirma (`clear_fixed_scope`). Las dos colecciones **nunca se fusionan**: pueden tener nombres, atajos e identificadores repetidos.

**Identidad de los botones fijos:** el prefijo lo da `button_prefix()` según el alcance vigente — `fixed_global_btn_{index}` o `fixed_{profile_id}_btn_{index}` —, así que los globales y los de cada perfil nunca colisionan.

**El lado no reconstruye nada:** `right` invierte las columnas con `flex-direction:row-reverse` sobre `.workspace-shell[data-fixed-side]`. Es CSS puro, sin rehacer la interfaz y sin parpadeo (regla 8).

**Precarga:** los audios del panel entran en la precarga (`engine/cache/warm.rs`) — los globales siempre; en alcance por perfil, solo los del perfil activo.

**`playback_mode`**
Modo de reproducción global del perfil. Valores: `"normal"`, `"loop"`, `"overlap"`, `"restart"`. Se combina con los flags individuales de cada botón en `domain/playback/mode.rs`.

**`player-tick`**
Evento Tauri que emite `engine/player/monitor.rs` cada 100 ms con el `PlayerSnapshot`. Lo escucha `runtimeEvents.js` y lo pinta `playerView.js`. Es **independiente** del `audio-tick` de los efectos por una razón concreta: el `audio-tick` calla cuando no hay efectos sonando, y la música de fondo suele sonar sola; colgar la lista de aquel tick la dejaba sin pintar.

**`PlayerEngine`**
Handle del motor del reproductor auxiliar (`engine/player/mod.rs`). Es `Send + Sync` y no toca audio: envía comandos por un canal al hilo dedicado, dueño de los dos decks. Vive en `AppState.player`. Ya no posee tarjeta: entrega sus fuentes al bus `Reproductor` de la [consola](#c).

Su `set_volume` **no pasa por el canal**: mueve el fader del bus, que es un atómico, y aplicar el volumen debe ser inmediato aunque el motor esté ocupado resolviendo la siguiente pista.

**`PlayerSnapshot`**
Estado en vivo del reproductor que Rust entrega a la interfaz: `playing`, `path`, `position_s`, `duration_s`, `current_index` (verde), `next_index` (naranja), `mode`, `stop_after` y `queue_len`. La UI solo lo pinta.

**`ping-pong`** (pre-carga)
Alternancia entre los dos decks del reproductor: mientras uno suena, la siguiente pista queda **pre-cargada** (decodificada y en pausa) en el otro, así el relevo es instantáneo y el motor nunca "se queda pensando". Patrón tomado del `PlayerPool` del LF Automatizador 2.0, reducido a dos decks.

**`pista huérfana`**
La pista que sigue sonando en el reproductor auxiliar después de desaparecer de la cola, al limpiar la lista o al abrir otra. **Editar la lista nunca corta la música**: la canción termina, pero queda sin verde (ya no está en la lista, así que `current = None`), y al acabar entra la lista nueva desde el principio. Es el criterio del LF Automatizador v1, cuyo `clearList` vacía las filas sin tocar la reproducción. En Rust lo garantiza `QueueState::set_entries`, que **no** emite `StopAll` cuando la pista actual ya no está. Nota: `PlayerSnapshot.playing` se lee del deck, no de `current`, así que una huérfana sigue reportando su tiempo correctamente.

**`pop-out`**
Acción de sacar el editor de pistas del modal y abrirlo en una ventana flotante nativa (`WebviewWindow`). La URL de esa ventana contiene `?editor=<ruta>` para que `startup.js` la detecte y arranque en modo editor exclusivo.

**`precarga de audio`**
Sistema que decodifica archivos de audio cortos en RAM antes de que el usuario los pulse, eliminando la latencia del disco. Configurable por estrategia (`FullProfile`, `VisibleTabs`, `OnPlay`), presupuesto de RAM (32–256 MB) y umbral de duración. Ver `engine/cache/preload.rs`, `engine/cache/preloader.rs`, `engine/cache/warm.rs`.

**`PreloadCache`**
Struct en `engine/cache/preload.rs`. `HashMap<String, Arc<CachedPcm>>` con lista LRU y contador de bytes usados. La función `build_play_source()` consulta esta caché antes de decodificar del disco.

**`PreloadStrategy`**
Enum Rust: `FullProfile`, `VisibleTabs`, `OnPlay`. Controla qué archivos se precalientan automáticamente. Ver [`preload.rs`](../src-tauri/src/model/preload.rs).

**`prelisten`**
Pre-escucha: reproducir un audio sin que salga al aire (los auriculares del locutor). Se usa el ID especial `__prelisten__` para el panel de pre-escucha, y `__track_preview__` para la previa dentro del editor. Los comandos con `to_pre=true` se enrutan **siempre** al bus `BusId::Cue` de la [consola](#c), sin fallback.

**Sin tarjeta dedicada sigue siendo privada.** El bus `Cue` existe siempre; si no tiene tarjeta propia usa [`Routing::ProgramDevice`](#r): sale por la tarjeta del programa pero con su propio enchufe, su fader y su medidor. Se suma con el programa **en el conector, no en el bus**, así que no le pega el máster ni cuenta en el vúmetro. `sanitize` impide rutear el CUE a `Program` aunque se pida.

**Cómo era antes (corregido en la Fase 3):** si `out_pre` estaba vacío —el caso por defecto— ese bus no existía y la pre-escucha **caía al principal**, donde le pegaba el máster y movía el vúmetro. Con tarjeta dedicada no pasaba, así que el mismo botón se comportaba distinto según el equipo.

**`PlayerConfig`**
Configuración persistida del reproductor auxiliar, colgada de `AppConfig.player`. Es **global**: hay un solo reproductor compartido entre perfiles. Contiene `tracks` (la cola, que reutiliza `ButtonData`), `playback_mode`, `volume` (0.0–1.5) y `output_device`.

`volume` **es el fader del bus `Reproductor`**: baja la música sin tocar los efectos, que es lo que hace falta para hablar encima. Es distinto del máster, que baja los tres buses a la vez; los dos se multiplican.

`output_device` vacío = [`Routing::Program`](#r): suma en el programa y **obedece al máster**. Un nombre = `Routing::Device(x)`: sale por esa tarjeta y deja de obedecerlo. **Trampa:** traducir `""` al nombre de la salida principal en el camino lo sacaría del programa sin querer — parecería lo mismo y no lo es.

**`ProfileData`**
Struct Rust (`model/content.rs`) que representa un perfil. Contiene `AudioConfig`, `active_paleta_id`, `Vec<PaletaData>` y `fixed_buttons` (los botones del panel fijo cuando el alcance es por perfil).

---

## Q

**`QueueEntry`**
Una fila de la cola del reproductor (`engine/player/queue.rs`): `id` (identidad estable, no depende de la posición), `kind`, `path`, `folder`, `cue_start_s`, `cue_end_s`, `gain` y `duration_s`. El `id` estable permite conservar la pista que suena y la marcada aunque se reordene o se borren otras filas.

Solo el audio normal viaja **resuelto** (ruta, cue y ganancia ya leídos de `tracks.db`). Los tipos especiales viajan con su `kind` y su `folder` **sin resolver**, a propósito: ver [resolución tardía](#r). De ahí que `is_playable()` mire la ruta para el audio y la carpeta para los demás, y que `needs_late_resolve()` distinga la hora y el clima (que dependen del momento) de la carpeta aleatoria (que no).

**`QueueState`**
Estado de la cola del reproductor: entradas, modo, `stop_after`, pista actual, cursor, marcada, deck activo y qué hay pre-cargado. No toca audio: **decide y devuelve `DeckAction`** que el hilo ejecuta. Los datos viven en `queue.rs`, el transporte en `queue_ops.rs` y la elección/pre-carga en `queue_select.rs`.

---

## R

**`Routing`**
Enum de `domain/console/routing.rs`. Dice a dónde entrega un [bus](#b) su señal:

- **`Program`** — suma en el bus [`Programa`](#b): le pega el máster y lo cuenta su vúmetro. Es lo que va al aire.
- **`ProgramDevice`** — sale por la **misma tarjeta** que el programa, pero **sin sumar en él**. Esta variante es la idea entera de la consola: que dos cosas salgan por el mismo altavoz no las convierte en la misma señal — se suman en el *conector*, no en el *bus*. Es lo que mantiene privada la [pre-escucha](#p) cuando solo hay una tarjeta de sonido.
- **`Device(x)`** — su propia tarjeta, ajeno al programa y al máster. Sacar un bus aquí es lo que "rompe" la regla de que todo pase por el máster.

**`effective(bus, routing, program_device)`** resuelve el ruteo pedido contra la tarjeta del programa: **pedir la tarjeta del programa es pedir el programa**. Sin eso habría dos formas de decir "por los altavoces" que suenan distinto — elegirlos por su nombre daría una salida directa (sin máster y fuera del vúmetro) y elegir "la misma que los efectos" sumaría en el programa. Nadie quiere dos señales en el mismo altavoz, una con máster y otra sin. El **CUE es la excepción**, y es la razón de ser de la consola: comparte el altavoz *a propósito* sin sumar en el programa.

Si el programa se muda a otra tarjeta, un bus que estaba en la suya se queda ahí solo y pasa a ser salida directa sin que nadie lo toque.

`sanitize(bus, routing)` corrige los ruteos imposibles antes de que lleguen al motor: el CUE no puede ser `Program`, y el `Programa` no puede sumarse a sí mismo. No es programación defensiva — es la regla de negocio: pedir que la pre-escucha suene "en el programa" no es un error que haya que silenciar, es una petición que la consola traduce a lo único que puede significar.

**`random_folder`**
Tipo de botón que reproduce archivos de una carpeta de forma secuencial. `random_folder.rs` mantiene el estado de qué archivo toca a continuación por cada botón. El nombre histórico es `random_folder` pero el comportamiento es secuencial (avanza uno a uno).

**`reproductor auxiliar`**
La lista de reproducción del panel lateral, pensada para dejar **música de fondo** sonando mientras se disparan los efectos. Tiene **motor propio** (`engine/player/`) e independiente del de efectos en lo que importa: su hilo, su cola, su avance y su transporte. Por eso el Stop general y el Solo de los efectos no lo cortan; tiene su propio Stop. Es uno solo y global.

**Lo que sí comparte es la salida.** Desde la Fase 4 de la [consola](#c) no tiene tarjeta propia: entrega sus fuentes al bus `Reproductor`, que suma en el programa. De ahí que **obedezca al máster** (decisión del autor, 2026-07-16) y aparezca en el vúmetro. La distinción que importa: el máster lo gobierna en **volumen**, no en **transporte** — bajarlo baja la música, pero pararlo todo no la para.

Idea tomada de los reproductores auxiliares del LF Automatizador v1; la arquitectura, del motor Rust del 2.0.

**`resolución tardía`**
Resolver una pista **en el momento de sonar** en vez de al pre-cargarla (`engine/player/resolve.rs`). Es obligatorio para la locución horaria y el clima: la pre-carga ocurre mientras suena la pista anterior, así que precargarlas diría la hora de hace varios minutos. La carpeta aleatoria sí se precarga, porque elegir la canción por adelantado no la estropea.

**`ResolvedEdit`**
Struct de `domain/playback/edit.rs` con el resultado de consultar `tracks.db` para un archivo: `cue_start_s`, `cue_end_s`, `file_gain`, `duration`. Si el archivo cambió (mtime/size), devuelve valores neutros. Es la **fuente única** del recorte y la ganancia: la usan tanto los botones como el reproductor auxiliar. Vive en `domain/` y depende solo del almacén de pistas (no del `AppState`) para que el motor del reproductor pueda llamarla sin depender de la capa IPC.

**`restart`**
Flag de `ButtonData`. Si está activo y el botón ya está sonando, volver a pulsarlo reinicia la reproducción desde el principio en lugar de crear una nueva instancia.

---

## S

**`sanitized_cue`**
Método de `TrackMeta`. Recorta los valores de `cue_start_s` y `cue_end_s` al rango `[0, duration]`. Evita que un cue fuera de rango (por ejemplo, si el archivo fue reemplazado por uno más corto) deje el botón en silencio.

**`seed_preload`**
Función interna de `cmd_button_playback.rs`. Al reproducir un botón, si la precarga está activa: marca el archivo en `last_played` y, si la estrategia es `OnPlay` y el archivo es corto, lo encola en el `Preloader`.

**`serde`**
Crate Rust de serialización/deserialización. Se usa con `serde_json` para convertir structs Rust ↔ JSON. `#[serde(default)]` es obligatorio en todo campo nuevo del modelo de datos para mantener compatibilidad con archivos más antiguos (y con el LFA).

**`Sink`**
Componente de rodio que consume una fuente de audio y la envía al dispositivo de salida CPAL. Aporta control de transporte (pausa, reanudación, volumen) sobre `play_raw`. Los [buses](#b) **no usan `Sink`**: un bus nunca se pausa, así que esa capa sobraba y se enchufan con `play_raw` directo. Quien sí los usa son los decks del [reproductor](#r), que sí paran y reanudan.

**`solo_mode`**
Campo de `AudioConfig`. Si es `true`, reproducir cualquier botón para automáticamente todos los demás (equivalente a que todos los botones tengan `stop_other=true`).

**`Source`** (rodio)
Trait de rodio que cualquier generador de audio debe implementar. Proporciona `channels()`, `sample_rate()` y `Iterator<Item=f32>`. `ButtonSource`, `CachedSource`, `CuedSource` y `SequenceSource` son implementaciones de este trait.

**`stop_other`**
Flag de `ButtonData`. Si está activo, reproducir el botón detiene todos los demás botones activos.

**`symphonia`**
Crate Rust de decodificación de audio pura Rust. Soporta MP3, WAV, FLAC, OGG/Vorbis, M4A, AIFF y más. Usado en `audio_decode.rs` para abrir cualquier archivo soportado.

---

## T

**`Tauri`**
Framework que empaqueta una aplicación web (Vite + JS) con un backend nativo Rust. Proporciona un WebView2 (Windows) o WebKit (Linux/macOS) para el frontend, y expone las APIs de sistema a través del IPC. Versión usada: Tauri v2.

**`TrackMeta`**
Struct Rust (`model/track.rs`) que representa una fila de `tracks.db`. Contiene todos los metadatos persistidos por el editor de pistas: cue, dB, normalización, LUFS, pico, mtime/size para validación.

**`TrackStore`**
Struct en `engine/persist/tracks.rs` que encapsula la conexión SQLite y ofrece métodos CRUD para `TrackMeta`: `upsert`, `get`, `set_cue`, `set_gain`, `set_normalization`, `touch_last_played`, `recent_paths`.

**`trim`**
El campo `vol` de `ButtonData`. Es un multiplicador lineal 0–1 que el usuario asigna por botón para ajustar el volumen relativo sin afectar la ganancia del archivo. Se preserva intacto en los exports `.bdelf` para compatibilidad con el LFA.

---

## V

**`VisibleTabs`**
Estrategia de precarga que carga en RAM los audios cortos de la pestaña activa, y vuelve a cargar al cambiar de pestaña. Intermedia entre `FullProfile` (más RAM) y `OnPlay` (más laxa).

**`vol`**
Ver [trim](#trim).

**`vúmetro`**
Indicador visual del nivel de audio en decibelios. En la Botonera muestra los canales estéreo L/R de la señal sumada en el bus principal. Implementado en `engine/console/level.rs` (medición en Rust: pico puro sobre PCM, ventana de ~1024 muestras) y `vuMeter.js` (animación en el frontend con balística de release suave). El [reproductor](#r) todavía no tiene vúmetro: sus decks van directos al `Sink`, sin pasar por un bus medido. Lo gana en la Fase 5 de la [consola](#c).

---

## W

**`WAL`**
*Write-Ahead Logging*. Modo de `tracks.db` (`PRAGMA journal_mode=WAL`) que permite escrituras frecuentes (debounce de `last_played`) sin bloquear lecturas concurrentes. Reduce la latencia de escritura a SQLite.

**`WaveEnvelope`**
Struct en `waveform.rs` que almacena la envolvente de onda de un archivo: pares `(min, max)` por segmento, con hasta 120.000 puntos. El método `view(start_s, end_s, buckets)` agrega la ventana visible para el zoom actual del canvas.

**`WaveformCache`**
Caché LRU en memoria (capacidad 6) de `WaveEnvelope`. Acelera vistas recientes durante la sesión del editor y se complementa con la caché persistente en disco.

**`WaveformDiskCache`**
Caché persistente de `WaveEnvelope` para el editor de pistas. Vive en disco, se invalida por `mtime`/`size`, y respeta los límites configurados de tamaño máximo y antigüedad máxima.

**`WebView2`**
Motor de renderizado web de Microsoft basado en Chromium. Tauri v2 lo usa en Windows para mostrar el frontend. Requiere Windows 10 20H2+ (normalmente ya viene instalado). En Linux se usa WebKit.

**`wizard`**
Asistente de primer arranque que aparece cuando `AppConfig.is_first_boot == true`. Guía al usuario en la configuración inicial (nombre del perfil, dispositivo de audio, módulo de locuciones).

---

## Z

**`zoom`**
En el editor de pistas, factor de ampliación horizontal del canvas de la onda. Controlado por un slider (1–30) y Ctrl+Rueda. `WaveEnvelope::view()` devuelve solo los buckets de la ventana visible para mantener la nitidez a cualquier zoom.
