# Plan — Modo Reproductor (reproductor auxiliar del panel fijo)

> Para retomar el hilo: [`CONTINUIDAD_SESION.md`](CONTINUIDAD_SESION.md).

Documento guía de trabajo. Define **qué queremos**, **cómo lo vamos a hacer** y
**qué resultado esperamos**. Sirve para retomar el flujo en próximas conversaciones
sin perder contexto. Se actualiza a medida que avanzan las fases.

- **Estado:** **completado**. Fases A, B, B.2, C (C.1–C.7) y D terminadas y verificadas.
  Lo que queda son mejoras opcionales: ver [§5 Qué falta](#5-qué-falta).
- **Rama:** `codex/panel-lateral-fijo`.
- **Origen de la idea:** reproductores auxiliares de LF Automatizador v1.0
  (`C:\LF Automatizador v1.0\frontend\auxiliary_playlists.js`, en JavaScript).
- **Guía principal de arquitectura:** el motor de reproducción **ya escrito en Rust** de
  LF Automatizador 2.0 (`C:\LF Automatizador v1.0\LF Automatizador 2.0\src-tauri\src\engine\`),
  en concreto `player.rs`, `players.rs` (el `PlayerPool` multi-reproductor) y
  `program/prefetch.rs` (la contabilidad de pre-carga). Es un reproductor principal, pero su
  mecánica es la misma que necesitamos para el auxiliar.
- **No se copia código:** se traduce y adapta a nuestra arquitectura y necesidades, y se
  reutilizan nuestras propias piezas de bajo nivel donde ya existen.

---

## 1. Qué queremos

El panel fijo tiene hoy dos vistas de los mismos botones: **modo botones** (rejilla) y
**modo lista** (lista compacta). Renombramos y reimplementamos el modo lista como
**modo reproductor**: un reproductor secuencial de una cola de pistas, pensado por
ejemplo para música de fondo mientras se disparan efectos.

### Decisiones tomadas (firmes)

1. **Motor propio.** El reproductor es un motor de audio **independiente** del motor de
   efectos: su propia salida, su propio hilo, su propio volumen y su propio dispositivo.
   No es un grupo dentro del motor de efectos.
2. **Colección propia.** El reproductor tiene su **propia cola de pistas**, separada de los
   botones fijos. El modo botones sigue mostrando los botones fijos; el modo reproductor
   muestra esta cola. Son dos herramientas distintas que conviven en el mismo panel.
3. **Un solo reproductor.** Uno en el panel. Ampliar a dos queda para el futuro.
4. **Tres modos de avance** (a nivel de lista, no de un botón suelto):
   - **Normal:** reproduce la lista en orden, una vez, y se detiene al final.
   - **Repetir:** reproduce en orden y al terminar vuelve al inicio, en bucle.
   - **Aleatorio:** elige pistas al azar de forma indefinida.

   El modo dice **qué** pista viene; que el reproductor se pare al acabar lo decide el botón
   "detener al finalizar", que se combina con los tres. _(Hubo un cuarto modo, `manual`, que
   hacía justo eso; se quitó en la Fase E.5 porque duplicaba el botón **y** limitaba: forzaba
   el orden normal, así que no convivía con aleatorio. Ver el registro de avance.)_
5. **Avance secuencial simple.** La siguiente pista empieza cuando la actual termina, sin
   solapamiento. Los fundidos/crossfade quedan como mejora futura opcional.
6. **Recorte por el editor de pistas existente.** Los puntos de inicio y fin se marcan con
   el editor de pistas que ya tenemos. Como el editor guarda los cue en `tracks.db` por
   ruta, la tubería de reproducción ya los respeta; no hay lógica nueva de cue.
7. **Salida propia: volumen + dispositivo.** Control de volumen independiente del master y
   posibilidad de enrutar a otro dispositivo físico de salida (como la pre-escucha).
8. **Tipos de contenido soportados:** audio normal, carpeta aleatoria, temperatura,
   humedad y locución horaria. **No** se incluye emisora por URL.
9. **Convención visual:** **verde** = pista que está sonando; **naranja** = pista marcada
   como siguiente. Respetar tema claro y oscuro con variables CSS.
10. **Independencia de los efectos.** El **Stop general** y el **Solo** de los efectos **no**
    detienen el reproductor. El reproductor tiene su propio botón de Stop.
11. **Alcance global.** Una sola cola compartida entre perfiles (no por perfil, por ahora).
12. **Arranque detenido.** Se persisten cola, modo, volumen y dispositivo; al abrir la app
    el reproductor empieza detenido.
13. **Dispositivo no disponible: se reutiliza la solución existente.** Por defecto el
    reproductor sale por el dispositivo predeterminado del sistema o por el mismo de los
    efectos, según configure el usuario, y se respeta su configuración. Si el dispositivo no
    está disponible, se reutiliza la recuperación de dispositivos que la botonera **ya tiene**
    (`audioDeviceRecovery.js`, `get_audio_device_status`, `device_available`/`find_device`).
    No se crea un mecanismo nuevo.

### Fuera de alcance (por ahora)

Doble deck con crossfade, ducking, filas de comando (saltos entre listas, ejecutar
eventos), multiselección compleja de edición, emisora por URL, salida por bus cue del LFA,
y un segundo reproductor. Se documentan aquí como posibles mejoras futuras.

---

## 2. Cómo lo vamos a hacer

Principios rectores: la UI es un control remoto (regla 4, toda la lógica en Rust);
soluciones desde la raíz sin parches (regla 2); límite de 200 líneas por archivo;
i18n en los cuatro idiomas; compatibilidad `.bdelf`/`.bdeplf` con campos nuevos opcionales
y `#[serde(default)]`. Dos recordatorios firmes del usuario:

- **Los archivos nacen modularizados por responsabilidad.** No se escribe todo en un archivo
  para trocearlo al llegar al límite; se separa por responsabilidad desde el inicio.
- **Reutilizar antes de crear.** Antes de escribir algo, buscar si ya existe en el código
  para no duplicar.

### Piezas existentes que se reutilizan (no se duplican)

- **Dispositivo y recuperación:** `engine/audio/device.rs` (`AudioDeviceRuntime`,
  `find_device`, `device_available`) y el flujo de recuperación (`audioDeviceRecovery.js`,
  `get_audio_device_status`). El reproductor abre su propio `OutputStream` con el mismo
  patrón que la pre-escucha.
  > **Desactualizado (2026-07-16).** `engine/audio/device.rs` y `AudioDeviceRuntime` ya no
  > existen: los absorbió `engine/console/`, que es la dueña de las tarjetas. `find_device` y
  > `device_available` viven ahora en `engine/console/device.rs`. El reproductor **sigue**
  > abriendo su propio `OutputStream`; deja de hacerlo en la Fase 4 de
  > [`PLAN_CONSOLA_VIRTUAL.md`](PLAN_CONSOLA_VIRTUAL.md).
- **Construcción de fuentes con cue y caché:** `engine/cache/preload.rs::build_play_source`
  devuelve un `BoxSource` (fuente rodio con cue de inicio/fin y caché ya aplicados),
  apto para `Sink::append`. Así el reproductor respeta el editor de pistas sin lógica nueva.
- **Resolución de cue/gain por pista:** el mismo camino que usa `cmd_button_playback` para
  leer `tracks.db` (cue de inicio/fin, ganancia, duración) por ruta.
- **Detección de formato/duración:** `engine/audio/formats`.
- **Pintor de progreso y tipos de icono:** `playbackPainter.js` y `typeIcons.js` en la UI.

### Arquitectura (patrón tomado del motor Rust de 2.0)

**Motor propio con dos reproductores (ping-pong) para arranque instantáneo.** El 2.0 mantiene
varios reproductores permanentes: mientras uno suena, los otros tienen la siguiente pista ya
**pre-cargada** (decodificada y en pausa). Al terminar, solo se hace `play()` de la que ya
estaba lista, sin esperar la decodificación. Para la botonera **dos reproductores bastan**:

```
   Reproductor A (suena)  ───fin──▶ play(B)
   Reproductor B (pre-cargado, en pausa, listo)
        ▲
        └── mientras A suena, el motor decide la siguiente pista y la pre-carga en B
```

Al reproducir B, el motor pre-carga la siguiente en A, y así se alterna. Nunca "se queda
pensando" al cambiar de pista.

- **Motor propio (`engine/player/`).** Instancia dedicada con su `OutputStream` en su propio
  dispositivo, dos `AudioPlayer` (cada uno envuelve un `Sink` de rodio), su hilo de control y
  su propio volumen. Es independiente del motor de efectos (`engine/audio/`). Reutiliza sin
  acoplarse nuestras piezas de bajo nivel: construcción de fuentes (`engine/cache`),
  resolución de cue/gain del editor (`domain/playback`) y detección de formato
  (`engine/audio/formats`).
- **Estados de cada reproductor** (como en `player.rs` de 2.0): `Stopped` → `Loaded`
  (pre-cargado, en pausa) → `Playing` → `Paused` → `Stopped`. `load()` decodifica y deja la
  pista lista en pausa; `play()` la arranca; un sondeo periódico detecta el fin (sink vacío).
- **Separación decisión/carga** (como `prefetch.rs` de 2.0). Tres pasos claros, todos en Rust:
  1. **Decidir**: según el modo, calcular el índice de la pista siguiente.
  2. **Pre-cargar**: hacer `load()` de esa pista en el reproductor ocioso (IO/decodificación).
  3. **Contabilidad**: recordar qué pista quedó cargada en qué reproductor.
  La interfaz no participa en ninguno de los tres.
- **Avance automático.** Un sondeo (nuestro hilo monitor u otro propio) detecta que la pista
  actual terminó y, según el modo, arranca la pre-cargada y prepara la siguiente. La interfaz
  nunca decide el avance.
- **Modelo (`model/player.rs`).** `PlayerConfig { tracks: Vec<ButtonData>, playback_mode:
  String, volume: f32, output_device: String }`, colgado de `AppConfig` con
  `#[serde(default)]`. Cada pista reutiliza `ButtonData` (ya lleva ruta, tipo, carpeta,
  volumen…), así que soporta directamente los cinco tipos de contenido sin inventar un tipo
  nuevo. El índice actual y el índice "siguiente" son estado de ejecución del motor, no
  necesariamente persistido.
- **Volumen y ganancia.** El 2.0 trabaja la ganancia en dB con fundidos (lineal y
  equal-power). Adoptamos su modelo de volumen propio del reproductor; los fundidos entre
  pistas (crossfade con los dos reproductores) quedan preparados para la mejora futura.
- **IPC (`ipc/cmd_player*.rs`).** Endpoints finos: obtener estado, reproducir, pausar,
  detener, siguiente, anterior, marcar siguiente, fijar modo, fijar volumen, fijar
  dispositivo, y editar la cola (añadir, quitar, reordenar).
- **Eventos.** Un tick informa a la UI: pista actual, estado, posición/duración y pista
  marcada como siguiente. La UI solo pinta, reutilizando el pintor común de progreso.

### Mapa de módulos propuesto

- `model/player.rs` — `PlayerConfig` y tipos puros.
- `engine/player/mod.rs` — el motor (pool de dos reproductores, salida propia, volumen).
- `engine/player/deck.rs` — un `AudioPlayer` (envoltura de `Sink`, estados, load/play/seek).
- `engine/player/advance.rs` — lógica de los cuatro modos y decisión de la siguiente pista.
- `ipc/cmd_player.rs` — controles de transporte y ajustes.
- `ipc/cmd_player_queue.rs` — edición de la cola (añadir, quitar, reordenar).
- `src/js/ui/player.js` — vista del reproductor en el panel.
- `src/css/player.css` — estilos (verde = sonando, naranja = siguiente).

Nombres de reproductores internos: `player-a` y `player-b` (convención heredada del 2.0).

### Glosario

- **Reproductor / deck:** una envoltura de `Sink` de rodio que reproduce una pista a la vez.
  El motor tiene dos: `player-a` y `player-b`.
- **Pre-carga (prefetch):** decodificar y dejar en pausa la siguiente pista en el deck ocioso,
  para que su arranque sea instantáneo.
- **Ping-pong:** alternancia entre los dos decks (mientras uno suena, el otro se pre-carga).
- **Pista siguiente:** la que el modo eligió como próxima; se muestra en naranja y suele estar
  ya pre-cargada.
- **Cola:** la lista ordenada de pistas del reproductor (`PlayerConfig.tracks`).

### Fases

- **Fase A — Fundación (motor + modelo).** `PlayerConfig`; el motor propio con salida propia,
  volumen y dispositivo propios, y sus dos decks; reproducir una pista suelta enrutada a ese
  motor. IPC mínimo. Pruebas con `cargo test --lib`.
- **Fase B — Cola, modos y pre-carga.** Añadir, quitar y reordenar pistas; avance automático
  en Rust para los cuatro modos con pre-carga ping-pong; controles reproducir, pausar,
  detener, siguiente, anterior y marcar siguiente.
- **Fase C — Interfaz del panel.** Dibujar la cola con la pista actual en verde y la
  siguiente en naranja, contador y barra de progreso (pintor común), controles de
  transporte, selector de modo, control de volumen y selector de dispositivo. Renombrar en
  ajustes "Presentación: Lista" por "Reproductor". Traducciones en los cuatro idiomas.
  Arrastrar y soltar desde el explorador reutilizando el flujo existente.
- **Fase D — Integración fina.** Verificar que el Stop general y el Solo de los efectos no
  cortan el reproductor; comprobar el recorte por cue del editor; **reutilizar** la
  recuperación de dispositivos existente si el dispositivo propio no está disponible; pulido
  visual en tema claro y oscuro.

---

## 3. Resultado esperado

- El panel fijo permite elegir entre **modo botones** y **modo reproductor**.
- En modo reproductor hay una cola propia de pistas, con transporte (reproducir, pausar,
  detener, siguiente, anterior) y selector de modo (Normal, Repetir, Aleatorio, Manual).
- La pista que suena se ve **verde**; la marcada como siguiente se ve **naranja**; ambas con
  contador y barra de progreso, en tema claro y oscuro.
- El reproductor tiene **volumen propio** y puede salir por un **dispositivo propio**,
  independiente de los efectos.
- Reproduce audio normal, carpeta aleatoria, temperatura, humedad y locución horaria, y
  respeta los cortes de inicio/fin marcados en el editor de pistas.
- Los efectos y el reproductor son independientes: detener todos los efectos no corta la
  música; el reproductor tiene su propio Stop.
- Verificación en cada fase: `cargo test --lib` y `npm run build` sin errores, archivos
  bajo 200 líneas, i18n completo, prueba visual en el equipo del usuario.

---

## 4. Registro de avance

- **Fase A — completada.**
  - Modelo `PlayerConfig` (`model/player.rs`) integrado en `AppConfig` con `#[serde(default)]`;
    constante `PLAYBACK_MODES` como fuente única de modos.
  - Motor propio en `engine/player/` (modular): `deck.rs` (un `Deck` que envuelve un `Sink`
    de rodio, con estados y load/play/pause/stop), `thread.rs` (hilo dedicado, único dueño
    del `OutputStream` y los dos decks; sondea fin de pista y refresca el snapshot),
    `mod.rs` (handle `PlayerEngine` con volumen y dispositivo propios). Reutiliza
    `find_device` y `build_play_source` (cue/cache); comparte la caché de precarga de efectos.
  - Integrado en `AppState` (`player: Mutex<PlayerEngine>`) y arrancado en `core/setup.rs`
    (dispositivo propio; "" = mismo que efectos; volumen propio).
  - IPC mínimo (`ipc/cmd_player.rs`): `get_player_state`, `player_play_path`, `player_stop`,
    `player_pause`, `player_resume`, `player_set_volume`, `player_set_device`. Reutiliza
    `resolve_edit` (cue/gain de tracks.db) hecho `pub(crate)`.
  - Verificación: `cargo test --lib` 68 aprobadas, 1 ignorada; sin warnings; archivos bajo 200.
- **Fase B — completada (backend).**
  - Lógica pura de avance en `domain/player/advance.rs`: enum `PlayerMode`
    (normal/repeat/random/manual) + `next_index` con "marcar siguiente" como ley,
    Normal se detiene al final, Repeat da la vuelta, Random evita repetir, Manual no avanza
    solo. Con pruebas.
  - Runtime de la cola en `engine/player/queue.rs` (datos) + `queue_ops.rs` (avance,
    pre-carga ping-pong, "detener al finalizar", "marcar siguiente" ley). Con pruebas.
  - Hilo del motor (`thread.rs`) reescrito para orquestar la cola sobre los dos decks:
    detecta el fin del deck activo y hace el relevo a la pista pre-cargada.
  - IPC de transporte (`cmd_player.rs`): `get_player`, `player_play_index`, `player_next`,
    `player_prev`, `player_mark_next`, `player_set_mode`, `player_set_stop_after`,
    stop/pause/resume, volumen, dispositivo.
  - IPC de cola (`cmd_player_queue.rs`): `player_add_track` (explorador),
    `player_add_button` (arrastrar un botón de la botonera/panel), `player_remove_track`,
    `player_reorder_tracks`, `player_clear_queue`. Reutiliza `resolve_edit` (cue/gain).
  - Arranque: `apply_startup` sincroniza modo y cola con el motor.
  - Verificación: `cargo test --lib` 77 aprobadas, 1 ignorada; `npm run build` correcto; sin
    warnings; archivos bajo 200 (dividí `queue` en datos + operaciones).
  - Pendiente para más adelante: tipos especiales en la cola (carpeta aleatoria, hora,
    clima) que requieren resolución en el momento (Fase B.2).
- **Fase C.1 — completada (vista + transporte).**
  - Renombrada la vista `list` → `player` (con migración de configuraciones antiguas en
    `cmd_fixed_panel::state`), en ajustes y en los cuatro idiomas.
  - `src/js/ui/playerView.js`: dibuja la cola (Título/Duración), pinta **verde** la pista que
    suena y **naranja** la marcada como siguiente, el total acumulado, y el transporte
    (Play, Pausa, Stop, Siguiente, Detener al finalizar). Clic = marcar siguiente;
    doble clic = reproducir ya.
  - `src/css/player.css` (verde `--success-strong`, naranja `--warning-color`, ambos temas) y
    `src/js/util/durationFormat.js` (mm:ss y hh:mm:ss).
  - Estado en vivo: `get_player_snapshot` (ligero) sondeado desde el tick existente solo
    cuando el modo reproductor está a la vista.
  - Verificación: `cargo test --lib` 68 aprobadas, 1 ignorada; `npm run build` correcto.
- **Fase C.2 — completada (arrastrar y soltar).**
  - `src/js/ui/playerDnd.js`: **reordenar** canciones con arrastre normal (sin Alt, porque en
    la lista el clic marca la siguiente y el doble clic reproduce); recibe lo soltado.
  - Soltar sobre una fila **inserta en esa posición**; soltar en el espacio vacío **añade al
    final**. Vale tanto para archivos del explorador como para un botón arrastrado desde la
    botonera principal o el panel fijo (Alt+arrastre, que es la convención existente).
  - Backend: `player_add_track` y `player_add_button` aceptan un `index` opcional; Rust
    inserta y renumera (la UI no calcula posiciones).
  - **Separación por responsabilidad** (regla: los archivos nacen modulares): `gridDnd.js`
    (arrastre interno con Alt) se separó de `fileDrop.js` (soltado de archivos externos, único
    oyente de `tauri://drag-*`), y el patrón repetido de errores se extrajo a `ipcError.js`.
    `gridDnd.js` pasó de 200 a 149 líneas.
  - Verificación: `cargo test --lib` 68 aprobadas; `npm run build` correcto.
- **Fase C.3 — completada (Limpiar / Abrir / Guardar).**
  - Formato **`.LFPlay`** compatible con LF Automatizador: array JSON de filas
    `{ruta, titulo, duracion, type, target}`. Adaptador en
    `domain/export/lfa_format/playlist.rs` (mismo patrón que `.bdelf`/`.bdeplf`). Las filas de
    comando del LFA (notas, saltos, eventos) se ignoran al importar; `random`↔`random_folder`.
  - `ipc/cmd_player_file.rs`: `player_save_playlist` y `player_open_playlist`. Cancelar el
    diálogo devuelve `false`/`None` en vez de error, para abortar sin ruido.
  - Abrir **reemplaza** la cola. Limpiar y Abrir preguntan antes con tres opciones
    (Guardar / No guardar / Cancelar) reutilizando `appConfirm3`; si el guardado se cancela,
    no se borra nada. Traducido a los cuatro idiomas, con sus errores.
  - Verificación: `cargo test --lib` 68 aprobadas; `npm run build` correcto.
- **Fase C.4 — completada (pulso propio, independencia del audio y compatibilidad).**
  - **La música no se detiene al editar la lista.** `set_entries` (`queue_ops.rs`) devolvía
    `StopAll` cuando la pista que sonaba desaparecía de la cola nueva, así que limpiar o abrir
    otra lista **cortaba la música**. Eliminado: ahora la pista sigue hasta su fin, queda
    "huérfana" (`current = None`; sin verde, porque ya no está en la lista) y al terminar entra
    la lista nueva desde el principio. Es el criterio del Automatizador, cuyo `clearList`
    vacía las filas sin tocar la reproducción.
  - **Pulso propio del reproductor.** `pollPlayerTick` se sondeaba desde el oyente de
    `"audio-tick"`, pero `engine/audio/monitor.rs` **solo emite si hay efectos sonando**
    (`if !is_empty || !was_empty`). Con música de fondo y sin efectos —el caso de uso
    principal— no había tick: ni verde, ni naranja, ni tiempo. Era un acoplamiento indebido
    entre dos motores independientes. Nuevo `engine/player/monitor.rs` (mismo patrón que el de
    efectos) que emite `"player-tick"`; arrancado en `core/setup.rs`. La UI escucha en vez de
    sondear (control remoto humilde). En reposo no emite: solo si suena o si algo cambió.
  - **Compatibilidad `.LFPlay`:** `duracion` acepta ahora el alias `duration`, porque el
    Automatizador lee `duracion ?? duration` y hay listas suyas con el nombre inglés.
  - **Pruebas:** `QueueState` **no tenía ninguna**, pese a que este registro afirmaba
    "Con pruebas" en la Fase B (era falso). Nuevo `queue_ops_tests.rs` (8 pruebas): la música
    no se corta al limpiar ni al abrir otra lista, el relevo de la huérfana a la lista nueva,
    y **"marcar siguiente" es ley al reorganizar** (la marca sigue a su canción por `id`, no a
    la posición; y se cae si se borra esa canción). Verificadas reintroduciendo la regresión
    a propósito para comprobar que fallaban. Más 4 pruebas del monitor y 2 del adaptador.
  - **Modularización:** `startup.js` llegó a 208 líneas (límite 200). Se extrajo el reparto de
    eventos en vivo a `runtimeEvents.js`; `startup.js` queda en 177 y solo orquesta el arranque.
  - Verificación: `cargo test --lib` 82 aprobadas, 1 ignorada, sin warnings; `npm run build`
    correcto; todos los archivos bajo 200 líneas.
- **Fase C.5 — completada (el naranja como guía y el doble clic).**
  - **El naranja ya no desaparece al parar.** `stop()` borraba `loaded_other` y, sin nada
    marcado, `next()` quedaba en `None`: el operador perdía la guía de qué venía. Ahora lo
    precargado pasa a marcado y, si no hay nada, se calcula lo que sonaría al pulsar Play.
    Mismo criterio al acabarse la lista en modo Normal.
  - **Lista vacía + añadir ⇒ la primera queda marcada sola.** Sale del mismo invariante:
    "detenido y con cola ⇒ hay naranja" (`ensure_upcoming_marked`, en `queue_select.rs`).
  - **Doble clic:** detenido reproduce; sonando marca como siguiente sin cortar la música.
    **Un solo clic ya no marca** (marcar sin querer al rozar una fila era problemático).
    La decisión la toma el MOTOR, no la UI (regla 4): nuevo `player_activate_index` →
    `QueueState::activate(index, is_playing)`. El `is_playing` lo aporta el hilo, que es quien
    conoce los decks: una pista huérfana puede sonar sin estar ya en la cola. Textos de ayuda
    actualizados en los cuatro idiomas.
  - **Modularización** (los archivos se pasaron del límite al crecer): `queue_ops.rs` (220)
    se separó en transporte (`queue_ops.rs`, 170) y elección/pre-carga (`queue_select.rs`, 64);
    `thread.rs` (204) soltó la ejecución de acciones sobre decks a `exec.rs` (60), quedando
    en 167.
  - Verificación: `cargo test --lib` 87 aprobadas, 1 ignorada, sin warnings; `npm run build`
    correcto; 13 pruebas en `queue_ops_tests.rs`.
- **Fase C.6 — completada (colores del LFA, listas reales y pulido).**
  - **Listas del Automatizador que no cargaban.** Su formato guarda `duracion` unas veces como
    número (`172`) y otras como **cadena** (`"31"`) — por eso el suyo la lee con `parseInt`.
    Nuestro `duracion: f64` reventaba con la cadena y, al fallar una fila, **se perdía el
    archivo entero** (`player_invalid_playlist`). Nuevo `flexible_secs`: acepta número o
    cadena, y una duración ilegible vale 0 en vez de tumbar la lista. Verificado contra una
    lista real de 30 filas: 30/30 importadas.
  - **Colores exactos del LFA v1** (`style.css`): verde sonando `#27ae60` (`.row-active`) y
    naranja siguiente `#d35400` (`.row-next`), con texto blanco y negrita. Como tokens
    `--player-playing-bg` / `--player-next-bg` / `--player-state-text`, con el MISMO valor en
    claro y oscuro: son colores de estado, no de tema.
  - **`--:--` en los tipos sin duración conocida** y fuera del total, igual que el
    Automatizador (`row.type === 'normal' ? formatTime(...) : '--:--'`). El total lo suma ahora
    **Rust** (`PlayerView.total_s`): qué cuenta y qué no es regla de negocio (regla 4).
  - **Ellipsis arreglado:** `text-overflow` no funciona sobre un contenedor flex, y
    `.player-row-title` era `inline-flex`, así que los títulos largos se cortaban en seco. El
    texto va ahora en un hijo propio, con el icono fuera.
  - **Selector muerto eliminado:** `.fixed-panel[data-view="list"]`, huérfano del renombrado
    `list` → `player`. Sin más restos de la vista antigua.
  - Verificación: `cargo test --lib` 89 aprobadas, 1 ignorada; `npm run build` correcto.
- **Fase B.2 — completada (tipos especiales en la cola).**
  - **Resolución tardía.** Los especiales no se pueden resolver al construir la cola: la hora
    avanza, el clima cambia y el aleatorio debe dar canción distinta en cada pasada. `QueueEntry`
    lleva ahora `kind` + `folder`, y `engine/player/resolve.rs` (`QueueResolver`) los traduce en
    archivos **al cargar en el deck**, que es "el momento de sonar".
  - **Todo reutilizado, nada inventado:** `RandomFolderState` (bolsa mezclada por fila, ya evita
    repetir), `resolve_time_files` / `resolve_climate_file` (locuciones), `weather_now` (clima) y
    `resolve_edit` (cue/ganancia). Las locuciones son varios archivos y suenan como una sola
    pista gracias a **`SequenceSource`**, que ya existía en `engine/audio/bus.rs`: el deck y el
    ping-pong no se tocaron.
  - **La hora y el clima NO se pre-cargan** (`needs_late_resolve`). La pre-carga ocurre mientras
    suena la pista anterior: precargar una locución horaria diría la hora de hace varios minutos.
    Se marcan en naranja igual, pero se cargan en el relevo. El aleatorio sí se pre-carga.
  - **Si la resolución falla** (carpeta vacía, sin internet, falta el archivo de esa hora), el
    deck queda `Failed`, que `poll_finished` trata como terminado: el motor releva y **la música
    sigue**, en vez de callarse esperando una locución que no existe.
  - **Sin ciclos:** `QueueResolver` recibe solo los `Arc` de config, carpetas aleatorias y
    pistas, nunca el `AppState` (que contiene el propio motor). `random_folders` pasa a `Arc`, y
    `resolve_edit` se movió de `ipc/cmd_button_playback.rs` a `domain/playback/edit.rs` para que
    el motor pueda usarlo sin depender de la capa IPC.
  - **Pruebas:** 101 aprobadas. Nuevas en `queue.rs` (qué es reproducible, qué se resuelve
    tarde), `resolve.rs` (carpeta de fila vs. global, tipo desconocido, secuencia vacía) y
    `queue_special_tests.rs` (la hora se marca pero no se precarga, y se carga fresca al
    relevar). Los dos guards de la locución se verificaron reintroduciendo la regresión.
  - `queue_ops_tests.rs` llegó a 248 líneas: se separó por tema en `queue_special_tests.rs`.
  - _Efecto secundario menor:_ rehacer la pre-carga (al marcar otra siguiente o editar la cola)
    consume un archivo de la bolsa aleatoria sin reproducirlo. Inocuo: la bolsa se rebaraja.
- **Fase C.7 — completada (ajustes del reproductor).**
  - `src/js/ui/settingsPlayer.js`: modo de avance, volumen propio y dispositivo propio en
    Ajustes → Panel fijo. Módulo aparte por responsabilidad, y porque `settingsModal.js` está
    en 197 líneas (límite 200) y no tiene margen: lo compone `settingsFixedPanel.js`, dueño de
    esa pestaña, igual que el panel de Reproducción compone la sección de fundidos. El modal
    solo cambió una línea ya existente, para pasar `config` y `devices` que **ya tenía pedidos**
    (cero IPC nuevo: `get_config` ya devuelve `player`).
  - **Volumen 0–100 %.** El backend admite hasta 1.5, pero ese margen solo existe con el modo
    boost del master; se deja sin usar por coherencia con el resto de la app.
  - **El dispositivo solo se envía si cambió**: reaplicarlo reabre la salida y **cortaría la
    música** (`open_device` hace `queue.stop()`). Mismo cuidado que ya tenía la tarjeta
    principal. El modo y el volumen también se envían solo si cambian.
  - **Se aplica al pulsar Guardar**, como el resto del modal: `player_set_volume` persiste en
    cada llamada y mandarlo mientras se arrastra el deslizador sería una tormenta de escrituras.
  - La sección se oculta cuando la presentación no es Reproductor, simétrico con "Columnas",
    que solo se ve en la presentación Botones.
  - Traducida a los cuatro idiomas (387 claves cuadradas en los cuatro).
  - Verificación: `npm run build` correcto; `cargo test --lib` 101 aprobadas.
- **Documentación al día (auditoría doc ↔ código, regla 13).** Se comparó lo que afirma cada
  documento con el código real, en vez de documentar de memoria. `ARCHITECTURE.md`,
  `LIBRO_PROYECTO.md`, `GLOSARIO.md`, `AGENTS.md` y `CLAUDE.md` ya cubrían la estructura; los
  huecos estaban en el **comportamiento**. Corregido lo que era **falso**:
  - `LIBRO_PROYECTO.md`: los colores del reproductor decían `--success-strong` / `--warning-color`
    (ahora son `--player-playing-bg` / `--player-next-bg`); y el `AppState` no incluía
    `player: Mutex<PlayerEngine>` y daba `random_folders` como `Mutex` en vez de `Arc`.
  - `GLOSARIO.md`: `ResolvedEdit` se situaba en `cmd_button_playback.rs` (está en
    `domain/playback/edit.rs`); `deck` no listaba `Finished` ni `Failed`; y `QueueEntry` se
    describía como "ya resuelta para sonar", cuando los especiales viajan sin resolver aposta.
  - Añadido en los tres documentos: el naranja como guía (no desaparece al parar; lista vacía ⇒
    primera marcada), la **pista huérfana** (editar la lista no corta la música), el doble clic
    (`activar fila`), `DeckStatus::Failed` (si la resolución falla, releva y sigue), y las
    duraciones desconocidas (`--:--` fuera del total, sumado por Rust).
  - `CHANGELOG.md`: como el reproductor entero está en `[Sin publicar]`, sus fallos nunca
    llegaron a un usuario; en vez de inventar entradas de "Corregido" se ajustaron las de
    "Añadido" para que describan el comportamiento **final**.
  - Verificado que los 11 símbolos que la documentación afirma (`ensure_upcoming_marked`,
    `needs_late_resolve`, `flexible_secs`, `QueueResolver`, `DeckStatus::Failed`, los tokens de
    color…) existen de verdad en el código.
- **Rescate del temporal de diseño del panel lateral.** Antes de borrar
  `LF_Botonera_conversacion_botones_fijos_TEMPORAL.md` (exportación de la conversación de diseño
  del panel, julio 2026) se auditó qué contenía que no estuviera en la documentación. Se comprobó
  contra el código que **todo lo acordado se implementó, salvo la política de colores**. Rescatado:
  - Al `GLOSARIO.md` y al `LIBRO_PROYECTO.md` (capítulo 13, renombrado a *El panel lateral fijo y
    el reproductor auxiliar*, porque el reproductor es solo una de sus dos vistas): el alcance es
    de **toda la barra**, no por botón; al cambiarlo la colección anterior **se conserva oculta**
    y solo se borra si se confirma; las colecciones **nunca se fusionan** (ids y atajos
    repetidos); la identidad depende del alcance (`fixed_global_btn_{i}` / `fixed_{perfil}_btn_{i}`,
    vía `button_prefix()`); el lado derecho es `row-reverse` por CSS, **sin reconstruir ni
    parpadeo**; la precarga incluye los fijos (globales siempre, por perfil solo el activo); y el
    panel no viaja en `.bdelf` porque ese formato representa *una pestaña*.
  - Corregido de paso: la entrada `panel fijo` del glosario aún decía que la presentación podía
    ser **"lista"** (resto del renombrado `list` → `player`).
  - Nuevo `PLAN_POLITICA_COLORES.md`: el diseño **acordado y nunca implementado** de la política
    de color de los botones nuevos (aleatorio / único / por filas / por columnas). Verificado que
    no existe en el código, y que `random_color()` se llama hoy desde **7 sitios** (el diseño
    original decía 3: no existían el panel fijo ni el reproductor), lo que refuerza centralizarla
    en `domain/colors` en vez de repetirla (regla 2). Enlazado desde los pendientes de `AGENTS.md`
    y `CLAUDE.md`.
- _(pendiente)_ **Limpieza (regla 14):** `Prueba.LFPlay` **se conserva** por decisión del autor.
  `LF_Botonera_conversacion_botones_fijos_TEMPORAL.md` ya tiene su contenido rescatado y puede
  borrarse cuando el autor lo confirme (no está versionado en git y nadie lo enlaza).
- **`model/config.rs` separado.** Estaba en 204 líneas (límite 200). Se partió por
  responsabilidad, no por tamaño: `config.rs` (136) son las **preferencias** (`AppConfig`) y
  `content.rs` (95) el **contenido** del usuario (`ButtonData`, `PaletaData`, `ProfileData`).
  Como `model/mod.rs` re-exporta, los 29 archivos que usan esos tipos no cambiaron.
  Verificado con un round-trip contra la configuración REAL del usuario: **las 928 claves
  sobreviven** sin perder ni añadir ninguna. (El round-trip destapó que `master_volume`, al ser
  `f32`, se reescribe como `0.4499999…` en vez de `0.45` cada vez que se guarda: es
  **preexistente** y ajeno a este cambio, pero queda anotado abajo.)
- **Fase D — verificada por el autor**: el Stop general de los efectos **no** detiene la lista del
  reproductor, que era la decisión 10. El pulido visual de ambos temas también quedó cubierto.
- **El total ya no se recorta** en la práctica (confirmado por el autor en su panel de 243 px). El
  `ellipsis` roto y el selector muerto `data-view="list"` ya están arreglados (Fase C.6). Si el
  panel se estrecha hacia su mínimo (180 px), el transporte podría volver a no caber: entonces
  tocaría dejar que el total baje de línea.

- **Fase B.3 — completada (locuciones del LFA: el marcador no era una carpeta).**
  - **El bug:** el LFA no guarda carpeta en las filas de locución; escribe un **marcador**
    (`time_locution`, `temperature_locution`, `humidity_locution`) en `ruta`, porque **cada
    aplicación resuelve la locución con SUS carpetas** — eso es lo que hace la compatibilidad
    bidireccional. `from_lfa_row` lo metía como `folder`, así que el reproductor buscaba un
    directorio llamado `time_locution`, no lo encontraba y **se saltaba la locución**.
    Detectado en la configuración real del autor: `('time', 'time_locution')`.
  - **Arreglado en los dos sentidos:** al importar, un marcador deja la carpeta **vacía** (manda
    la de Ajustes, como el LFA con las suyas); al exportar, una locución sin carpeta propia
    escribe el marcador, porque una `ruta` vacía le dejaría al LFA una fila irresoluble. Una
    carpeta real sí se conserva: la botonera admite carpeta por fila.
  - **Migración automática** (`config_io::clear_locution_markers`): las configuraciones que ya
    guardaron el marcador como carpeta se limpian al cargar. El operador no reimporta nada.
  - **Duplicación eliminada:** `resolve.rs` re-implementaba la resolución de carpeta como
    `folder_or_config`. Ahora usa `resolve_time_folder` / `resolve_climate_folder`, las MISMAS
    que los botones (hechas `pub(crate)`), así que una locución se ubica igual venga de un botón
    o de la cola — y de paso hereda gratis el respeto a los interruptores del módulo.
  - Verificación: `cargo test --lib` 109 aprobadas, sin warnings. Tests nuevos en
    `playlist_tests.rs` (los tres marcadores, la carpeta real que se conserva, el marcador al
    exportar y la ida y vuelta) y `config_io_tests.rs` (la migración, y que no toca carpetas
    reales ni otros tipos). El del marcador se verificó reintroduciendo la regresión.
  - `playlist.rs` llegó a 242 líneas: sus pruebas se separaron a `playlist_tests.rs`.

- **Fase E.1 — completada (barra de progreso con seek, Loop y contador).**
  - **Botón Loop** (repetir la canción actual). No confundir con el modo `repeat`, que repite la
    LISTA: por eso el modo repetir usa **∞** y el Loop **🔂**. Es de transporte, como
    "detener al finalizar": sigue puesto al cambiar de canción y no se persiste.
    - **No toca la fuente de audio**: en vez de recrearla con `loop_mode`, `advance` recarga la
      misma pista al terminar. Así ponerlo o quitarlo a mitad **no corta lo que suena**.
    - Reglas acordadas con el autor y cubiertas por tests: manda sobre "detener al finalizar"
      (la canción no termina, así que el stop no llega a actuar); **cede ante el botón
      Siguiente**, que es del operador; y **no toca lo marcado**, que sigue en naranja
      esperando — el Loop dice *cuándo* acaba esta, no *qué* viene después.
  - **Barra de progreso con seek.** Rodio no sabe reposicionar un `Sink`, así que un seek es
    reconstruir la fuente desde el punto pedido, reutilizando `build_play_source` (O(1) si está
    en caché). El `Deck` guarda ahora un `DeckTrack` con lo necesario para rehacerla, y un
    `position_offset_s`, porque la fuente nueva cuenta desde cero y el tiempo retrocedería.
    - La ruta para recargar sale del **deck** (ya resuelta), no de la cola: en una carpeta
      aleatoria la cola solo conoce la carpeta, y recargar debe sonar **la misma** canción.
    - **Las locuciones no se pueden reposicionar** (son varios archivos encadenados en una sola
      fuente): `DeckTrack.seekable` es falso y el snapshot lo publica como `can_seek`, así que la
      barra se muestra pero no deja arrastrar.
  - **El contador sustituye al "Total"** y se muda a la fila de la barra: con los controles
    nuevos no cabía todo en una fila de 243 px. Al pulsarlo alterna transcurrido ↔ restante, y
    **con el reproductor parado enseña el total de la lista** (en apagado, para distinguirlo).
    La preferencia la decide y recuerda **Rust** (`PlayerConfig.time_display`,
    `player_toggle_time_display`): la UI solo pide el cambio y pinta.
  - **Modularización** (los tres se pasaron del límite al crecer): `queue_ops.rs` (205) soltó la
    configuración de la cola a `queue_edit.rs` (67) y el seek a `queue_select.rs`, quedando en
    144; los tests del Loop salieron a `queue_loop_tests.rs`; y la fila de progreso vive en
    `src/js/ui/playerProgress.js`, porque `playerView.js` no tenía margen.
  - Verificación: `cargo test --lib` 113 aprobadas, sin warnings; `npm run build` correcto;
    389 claves i18n cuadradas en los cuatro idiomas; ningún archivo sobre 200 líneas.
- **Fase E.2 — completada (menú de modos y volumen desde el panel).**
  - **Menú de modos** en la cabecera (`playerModes.js`): despliega los cuatro con un check en el
    activo. En vez de unas rayas mudas, **el icono es el modo vigente** (➡ normal, ∞ repetir,
    🔀 aleatorio, ✋ manual): así se sabe en qué modo estás sin abrir nada, que en directo
    importa. El **∞** distingue "repetir la lista" del **🔂** del Loop, que repite una canción.
    Los modos siguen también en Ajustes: se sincronizan solos, porque ambos leen de Rust.
  - **Volumen** (`playerVolume.js`): el panel se abre **encima** del botón, no a los lados —el
    panel lateral es estrecho—, con `placeMenu`, que además lo mete hacia dentro si no cupiera.
    El icono cambia con el nivel (🔇 / 🔉 / 🔊) y se pinta desde el tick (`PlayerSnapshot.volume`).
  - **`player_set_volume` gana `persist`**: mientras se arrastra el deslizador se aplica pero
    **no se guarda** (aplicar es un atómico; guardar en cada píxel sería una escritura a disco
    por movimiento), y al soltar se persiste. Sin el parámetro guarda, así que los ajustes no
    cambian. Es el mismo cuidado que ya tenía el volumen master, que solo guarda si "recordar".
  - **Modularización:** `cmd_player.rs` llegó a 205 líneas y se separó por lo que de verdad los
    distingue: `cmd_player.rs` es transporte (estado de ejecución, no se guarda) y
    `cmd_player_config.rs` los ajustes que **sí se persisten** (modo, volumen, dispositivo,
    contador). En el frontend, `playerModes.js` y `playerVolume.js` nacen aparte.
  - Verificación: `cargo build --lib` **sin avisos** (la comprobación nueva, que es como compila
    el usuario), `cargo test --lib` 113 aprobadas, `npm run build` correcto, 393 claves i18n
    cuadradas en los cuatro idiomas, ningún archivo sobre 200 líneas.

- **Fase E.3 — completada (pulido de la interfaz).**
  - **Orden de la cabecera:** Limpiar, Abrir, Guardar, Modo (el modo iba primero por error).
  - **Iconos coherentes.** El transporte usa símbolos monocromos (▶ ⏸ ■ ⏭) y los demás eran
    emojis a color: chirriaba. Ahora "detener al finalizar" es **`▶■`** ("suena esto y para") y
    los modos van en el mismo estilo: **`→`** normal, **`∞`** repetir la lista, **`⇄`** aleatorio
    y **`▶■`** manual. Manual comparte icono con "detener al finalizar" **a propósito**
    (decisión del autor): los dos significan "para al acabar esta", uno como modo permanente y
    el otro de una vez. Limpiar 🧹, Abrir 📂, Guardar 💾 y el volumen 🔊 siguen siendo emojis,
    para que destaquen: son de otra naturaleza.
  - **Columnas y Filas se ocultan en modo reproductor.** No se pudieron "eliminar", como se pidió
    en un principio, porque **no son opciones del reproductor**: son la CAPACIDAD de la rejilla
    de botones fijos (`columnas × filas` limita cuántos caben, ver
    `cmd_fixed_panel::set_fixed_panel_settings`). Borrarlas rompería el modo botones. Columnas ya
    se ocultaba; Filas no, y ese era el resto que sobraba en la pantalla.
  - **La salida del reproductor se mudó a Ajustes → Principal**, junto a la principal y la de
    pre-escucha: todas las salidas de audio en un mismo sitio. El modo y el volumen se quedan en
    Panel fijo. La salida **no** se oculta con la presentación: el reproductor puede seguir
    sonando con el panel enseñando los botones fijos.
  - Verificación: `cargo build --lib` sin avisos, 113 pruebas, `npm run build` correcto, 393
    claves i18n cuadradas, ningún archivo sobre 200 líneas.
- **Salida propia del panel fijo — DESCARTADA** por decisión del autor (2026-07-16). Se estudió
  y se documenta aquí para no volver a plantearlo: no existe (`FixedPanelConfig` no tiene
  `audio_out`) y los botones fijos suenan por el motor de efectos con el `out_main` del perfil;
  `PlaybackGroup::Fixed` solo agrupa (Solo y Stop), **no enruta**. Tampoco hay precedente: el
  `audio_out` de las paletas **no enruta nada**, solo se guarda y se exporta por compatibilidad
  con el LFA. Habría hecho falta un tercer `AudioDeviceRuntime` y cambiar el booleano `to_pre`
  por una decisión de tres — cirugía en el corazón del motor de efectos.

- **Fase E.4 — completada (el salto de posición, arreglado de raíz).**
  - **El síntoma:** adelantar la canción dejaba un silencio largo. Medido antes de tocar nada:
    0,5 s para saltar 10 s · 1,6 s para 30 s · 3,3 s para 60 s · **6,6 s para 120 s**.
  - **La causa:** el salto real **nunca funcionó, en ningún formato**. `rodio::Decoder` envuelve
    el lector en su `ReadSeekSource`, que informa `byte_len() = None`; symphonia necesita el
    tamaño del archivo para posicionarse en formatos sin índice, así que `try_seek` fallaba
    siempre (FLAC: `Unseekable`; MP3: `end of stream`) y se caía a `CuedSource`, que llega al
    punto **descartando las muestras una a una** (~55 ms por segundo saltado). En la botonera
    principal no se notaba porque sus efectos duran segundos y **están en la caché de RAM**,
    donde `CachedSource::new_at` ya era O(1): no era mejor código, eran otros datos.
  - **El arreglo:** `engine/audio/seek_source.rs`, que usa symphonia directamente pasándole el
    `File` — `File` **sí** implementa `MediaSource` informando del tamaño. Va enchufado en
    `source_from_path_at`, el punto único por el que pasan **los dos motores**: arregla el
    reproductor *y* la botonera principal con canciones largas, que era lo pedido.
  - **Resultado medido:** saltar cuesta **~10 ms sea cual sea la distancia** (de 6,6 s a 10 ms
    para ir al minuto 2), y la diferencia con el audio de referencia es **0,000000** muestra a
    muestra: cae exacto.
  - **Un bug que cazó una prueba que ya existía:** el salto aterriza al principio del bloque que
    contiene el punto (en FLAC, ~90 ms antes), no en el punto. Sin descartar ese sobrante
    (`skip`) se devolvía audio ANTERIOR al pedido, y
    `source_from_path_at_reads_near_requested_position` lo detectó al instante.
  - Se descartaron: **actualizar rodio a 0.22** (donde `with_byte_len` ya lo resuelve) por ser
    cirugía en la librería que mueve todo el audio — queda como mejora futura; y **precargar la
    canción entera**, medido en 9 s de decodificación y 25 MB por canción.
  - Verificación: 118 pruebas (5 nuevas en `seek_source_tests.rs`, con un WAV generado al vuelo
    para no depender de ningún archivo del equipo), `cargo build --lib` sin avisos. Los tests se
    comprobaron reintroduciendo la regresión. `decode.rs` llegó a 206 líneas y sus pruebas
    salieron a `decode_tests.rs`.

- **Fase E.5 — completada (fuera el modo `manual`: eran tres modos, no cuatro).**
  - **Por qué.** `manual` (no avanzar solo) y el botón "detener al finalizar" hacían lo mismo, y
    el código lo confirmaba: `manual` **además limitaba**, porque para elegir la siguiente
    forzaba `PlayerMode::Normal` — o sea que "manual + aleatorio" era **imposible**. El botón,
    en cambio, usa `peek_next()`, que respeta el modo vigente. Era duplicado *y* peor.
  - **Ahora:** tres modos (`normal`, `repeat`, `random`) que dicen **qué** pista viene, y un
    interruptor que decide **si se para al acabar**, combinable con los tres. Se gana lo que
    antes no existía: pararse en cada pista con la siguiente elegida al azar.
  - Decisión del autor: el interruptor **no se persiste**, igual que el Loop, aunque el modo al
    que sustituye sí se guardaba. Es de transporte y empezar apagado es predecible.
  - **Migración:** una configuración con `playback_mode = "manual"` pasa a `"normal"` al cargar
    (`config_io::normalize_playback_modes`), que es lo que `manual` hacía para elegir la
    siguiente. `PlayerMode::from_config` ya era tolerante, pero se migra explícitamente para no
    dejar en disco un valor que no existe.
  - Al quitar `manual` sobraron dos ramas en `advance` y el `match` de `ensure_upcoming_marked`
    que existía solo para sortearlo: `queue_ops.rs` ya no necesita ni importar la regla pura.
  - Verificación: 121 pruebas (3 nuevas, incluida la combinación que antes era imposible),
    `cargo build --lib` sin avisos, 391 claves i18n cuadradas en los cuatro idiomas.

---

## 5. Qué falta

**El modo reproductor está terminado**: fases A, B, B.2, C (C.1–C.7) y D. Lo que queda es opcional
o ajeno a esta función.

### Mejoras futuras del reproductor (opcionales, nunca acordadas como obligatorias)

1. **Fundidos / crossfade entre pistas.** Hoy el avance es secuencial simple (decisión 7). Los dos
   decks ya están en ping-pong, que es la mitad del trabajo: faltaría la rampa de ganancia. El LF
   Automatizador 2.0 ya lo tiene resuelto (lineal y *equal-power*) y serviría de guía.
2. **Un segundo reproductor.** El LFA v1 tiene dos; se decidió que uno bastaba (decisión 3). El
   motor no lo impide: habría que pasar de un `PlayerEngine` a una colección.
3. **Barra de progreso por pista en la lista.** Hoy el `player-tick` ya trae `position_s` y
   `duration_s`; se podría pintar como en la botonera principal reutilizando `playbackPainter.js`.

### Deuda técnica conocida (ajena al reproductor)

- **`master_volume` es `f32`**: su representación en JSON crece sola al guardar
  (`0.45` → `0.4499999...`). Inocuo funcionalmente, pero ensucia el fichero. Afecta a
  `AudioConfig` y a `vol` de `ButtonData`.
- ~~**Política de colores de los botones nuevos**~~ — **DESCARTADA** (2026-07-16): el autor la vio
  complicada de explicar y de usar. En su lugar hay **selección múltiple** (Ctrl+clic y clic
  derecho → pintar). No volver a proponerla; [`PLAN_POLITICA_COLORES.md`](PLAN_POLITICA_COLORES.md)
  se conserva solo como registro.
