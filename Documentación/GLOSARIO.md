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
Struct Rust (también JSON serializable) que representa la configuración completa de la aplicación: tema, idioma, perfiles, precarga, locuciones, caché persistente de waveform, etc. Se persiste en `botonera_config.json`. Ver [`config.rs`](../src-tauri/src/model/config.rs).

**`AppState`**
Objeto inyectado por Tauri en cada comando IPC. Contiene referencias a todo el estado compartido del backend: configuración, motor de audio, base de datos, caché de formas de onda, historial de reproducción. Ver [`state.rs`](../src-tauri/src/core/state.rs).

**`Arc<Mutex<T>>`**
Patrón Rust para compartir estado entre hilos. `Arc` es un contador de referencias atómico (compartir el puntero); `Mutex` garantiza exclusión mutua (solo un hilo accede a la vez). Se usa extensivamente en `AppState` para pasar el estado a los hilos de audio, monitor y flusher.

**`analyze_track`**
Comando IPC (`cmd_tracks.rs`) que analiza una pista para el editor y devuelve: envolvente de onda, LUFS, pico en dBFS, ganancia sugerida, duración y metadatos de cue ya guardados. Delega en `engine::dsp::editor_analysis` mediante `spawn_blocking`, emite `track-analysis-progress`, reutiliza `TrackAnalysisCache`, `tracks.db` y caché persistente de waveform antes de decodificar el audio completo. Nunca corre en el hilo de audio.

**`audio-tick`**
Evento Tauri emitido por `engine/audio/monitor.rs` cada ~100 ms mientras hay audio reproduciéndose. Payload: `{buttons[], display_remaining, display_duration, master_level_l, master_level_r}`. Startup.js lo re-emite como `CustomEvent('lf-audio-tick')` en el DOM (distinto al evento Tauri).

**`AudioCommand`**
Enum Rust en `engine/audio/command.rs`. Variantes: `Play`, `Stop`, `StopAll`, `SetDevice`, `SetPreDevice`, `SetVolume`, `PlaySequence`. Se envía por un canal `mpsc` desde `AudioEngine` al hilo de audio (`engine/audio/thread.rs`).

**`AudioConfig`**
Struct de configuración de audio por perfil. Contiene los dispositivos de salida (`out_main`, `out_pre`), atajos globales, modo de reproducción y volumen master. Ver [`audio.rs`](../src-tauri/src/model/audio.rs).

**`AudioDeviceRuntime`**
Struct en `engine/audio/device.rs` que gestiona el ciclo de vida de un `OutputStream` y su `MasterBus`. Al cambiar de dispositivo, recrea el stream sin interrumpir el hilo de audio.

**`AudioEngine`**
Fachada pública del motor de audio en `engine/audio/engine.rs`. Posee el `Sender<AudioCommand>` hacia el hilo de audio. También posee los `Arc` de atómicos (nivel VU, volumen master) y el `Preloader`. No toca rodio directamente.

---

## B

**`bdelf`**
Extensión de archivo para exportar una paleta (pestaña) de la Botonera. JSON compatible con el LF Automatizador. Puede contener el campo opcional `bdelf_tracks` con metadatos de cue y dB que el LFA ignora.

**`bdeplf`**
Extensión de archivo para exportar un perfil completo. Similar al `.bdelf` pero incluye todas las paletas del perfil.

**`botón`**
Celda de la rejilla a la que se asigna un archivo de audio (u otro tipo). Representado por `ButtonData` en Rust. Tiene id único, tipo, ruta, volumen, flags de comportamiento y atajo de teclado.

**`ButtonData`**
Struct Rust (`model/config.rs`) que representa un botón. Campo `type_field` se serializa como `"type"` en JSON. El campo `vol` es un multiplicador lineal 0–1, no en dB (ver [trim](#trim)).

**`ButtonSource`**
Struct Rust (`engine/audio/button.rs`) que implementa `Iterator<Item=f32>`. Envuelve cualquier fuente de audio y aplica la cadena de ganancia: `muestra × file_gain × vol_botón × master`. Puede pararse por `stop_flag` o marcarse como terminado por `done_flag`.

**`ButtonState`**
Struct Rust (`engine/audio/button.rs`) que rastrea el estado de reproducción de una instancia de un botón: `start_time`, `duration`, `done_flag`, `stop_flag`, `volume`. Permite calcular `position()` y `remaining()` sin consultar el hilo de audio.

**`ButtonStateMap`**
`HashMap<String, Vec<ButtonState>>` — una entrada por id de botón, con un Vec para los modos `overlap` (varias instancias simultáneas del mismo botón). Ver [`button.rs`](../src-tauri/src/engine/audio/button.rs).

---

## C

**`CachedPcm`**
Struct en `engine/cache/cached_source.rs` que almacena el PCM decodificado como `Vec<i16>`, más la tasa de muestreo y el número de canales. Compartido via `Arc<CachedPcm>` entre la caché y las fuentes activas.

**`CachedSource`**
Struct en `engine/cache/cached_source.rs` que implementa `Source<Item=f32>`. Lee desde un `Arc<CachedPcm>` y permite crear instancias en cualquier posición con `new_at(pcm, offset)` en tiempo O(1). Solución al problema de seek con latencia creciente.

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

**`decode_pcm`**
Función en `engine/cache/preload.rs` que decodifica un archivo a `Vec<i16>` para guardarlo en la `PreloadCache`. Solo se llama desde el hilo del `Preloader` o desde el análisis de pistas. Nunca desde el hilo de audio.

**`DynamicMixer`**
Componente de rodio que mezcla múltiples fuentes de audio en tiempo real. El `MasterBus` usa uno para combinar todos los botones activos en una sola señal antes de enviarla al dispositivo.

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

**`file_gain`**
Ganancia por archivo en multiplicador lineal. Representa la ganancia total calculada por el editor de pistas: `10^((norm_gain_db + gain_db) / 20)`. Es la primera capa del modelo de ganancia de 3 capas. Ver `TrackMeta::effective_gain_linear()`.

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

**`master_volume`**
Volumen global del perfil activo. Rango 0–1 (o 0–1.5 en modo boost). Es la tercera capa del modelo de ganancia. Se aplica atómicamente en cada muestra dentro de `ButtonSource`.

**`MasterBus`**
Struct en `engine/audio/bus.rs`. Combina un `DynamicMixer<f32>` (mezcla todas las fuentes) con un `LevelSource` (mide el PICO del audio sumado) y un `Sink` (envía al dispositivo de audio). Existe uno para la salida principal y otro para la pre-escucha.

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

**`open-meteo`**
API de clima gratuita y de código abierto usada por `engine/weather/client.rs` para obtener temperatura y humedad actuales a partir de coordenadas geográficas. La caché es de 10 minutos.

**`overlap`**
Flag de `ButtonData`. Si está activo, reproducir el botón mientras ya está sonando crea una nueva instancia simultánea en lugar de pararlo o ignorarlo.

---

## P

**`paleta`**
Pestaña de la rejilla de botones. Un perfil puede tener múltiples paletas. Cada paleta tiene su propia cuadrícula (filas × columnas), nombre, colores y opcionalmente un dispositivo de audio independiente. Representada por `PaletaData`.

**`PaletaData`**
Struct Rust (`model/config.rs`) que representa una paleta. Su id tiene el formato `"paleta_1"`, `"paleta_2"`, etc.

**`perfil`**
Configuración raíz que agrupa una o más paletas. Un perfil tiene nombre, colores, ajustes de audio (dispositivos, atajos, modo de reproducción) y una lista de paletas. Representado por `ProfileData`.

**`playback_mode`**
Modo de reproducción global del perfil. Valores: `"normal"`, `"loop"`, `"overlap"`, `"restart"`. Se combina con los flags individuales de cada botón en `domain/playback/mode.rs`.

**`pop-out`**
Acción de sacar el editor de pistas del modal y abrirlo en una ventana flotante nativa (`WebviewWindow`). La URL de esa ventana contiene `?editor=<ruta>` para que `startup.js` la detecte y arranque en modo editor exclusivo.

**`precarga de audio`**
Sistema que decodifica archivos de audio cortos en RAM antes de que el usuario los pulse, eliminando la latencia del disco. Configurable por estrategia (`FullProfile`, `VisibleTabs`, `OnPlay`), presupuesto de RAM (32–256 MB) y umbral de duración. Ver `engine/cache/preload.rs`, `engine/cache/preloader.rs`, `engine/cache/warm.rs`.

**`PreloadCache`**
Struct en `engine/cache/preload.rs`. `HashMap<String, Arc<CachedPcm>>` con lista LRU y contador de bytes usados. La función `build_play_source()` consulta esta caché antes de decodificar del disco.

**`PreloadStrategy`**
Enum Rust: `FullProfile`, `VisibleTabs`, `OnPlay`. Controla qué archivos se precalientan automáticamente. Ver [`preload.rs`](../src-tauri/src/model/preload.rs).

**`prelisten`**
Pre-escucha: reproducir un audio en un dispositivo de salida separado (auriculares del locutor) sin que salga al aire. Se usa el ID especial `__prelisten__` para el panel de pre-escucha, y `__track_preview__` para la previa dentro del editor. Los comandos con `to_pre=true` se enrutan al `device_pre` del hilo de audio.

**`ProfileData`**
Struct Rust (`model/config.rs`) que representa un perfil. Contiene `AudioConfig`, `active_paleta_id` y `Vec<PaletaData>`.

---

## R

**`random_folder`**
Tipo de botón que reproduce archivos de una carpeta de forma secuencial. `random_folder.rs` mantiene el estado de qué archivo toca a continuación por cada botón. El nombre histórico es `random_folder` pero el comportamiento es secuencial (avanza uno a uno).

**`ResolvedEdit`**
Struct interna de `cmd_button_playback.rs` con el resultado de consultar `tracks.db` para un archivo: `cue_start_s`, `cue_end_s`, `file_gain`, `duration`. Si el archivo cambió (mtime/size), devuelve valores neutros.

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
Componente de rodio que consume una fuente de audio y la envía al dispositivo de salida CPAL. El `MasterBus` crea un único `Sink` para todo el audio del bus.

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
Indicador visual del nivel de audio en decibelios. En la Botonera muestra los canales estéreo L/R de la señal sumada en el `MasterBus`. Implementado en `vu_meter.rs` (medición en Rust) y `vuMeter.js` (animación en el frontend con balística de release suave).

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
