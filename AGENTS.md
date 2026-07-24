# AGENTS.md — LF Botonera de Efectos

Guía para cualquier IA colaboradora (independientemente del modelo o plataforma).
Lee este archivo completo antes de proponer o escribir código.

> **Empieza por [`Documentación/CONTINUIDAD_SESION.md`](Documentación/CONTINUIDAD_SESION.md)**: estado del trabajo, decisiones cerradas y trampas ya verificadas.
> Si usas Claude Code, lee también [`CLAUDE.md`](CLAUDE.md) para instrucciones específicas de esa herramienta.
> Para una narrativa completa del proyecto, consulta [`Documentación/LIBRO_PROYECTO.md`](Documentación/LIBRO_PROYECTO.md).
> Para el glosario de términos, consulta [`Documentación/GLOSARIO.md`](Documentación/GLOSARIO.md).

---

## 1. Qué es este proyecto

**LF Botonera de Efectos** es una botonera de sonidos (*soundboard*) para radio y *streaming* en directo. Los operadores de radio asignan archivos de audio a botones en una rejilla, organizados en pestañas (paletas) dentro de perfiles, y los disparan en tiempo real durante transmisiones.

- **Versión:** 1.2.1
- **Stack:** Tauri v2 (backend Rust) + Vanilla JS + Vite (frontend)
- **Repositorio local:** `C:\OVERLAY\BOTONERA`
- **GitHub:** https://github.com/yosoyluisfernando/LF-Botonera-de-efectos
- **Licencia:** GPL-3.0-or-later
- **App hermana:** LF Automatizador v1.0 — comparte los formatos `.bdelf` y `.bdeplf`

---

## 2. Principios de desarrollo

Estos principios definen los estándares de calidad del proyecto y aplican a todo cambio de código.
El documento completo está en [`Documentación/REGLAS_PROYECTO.md`](Documentación/REGLAS_PROYECTO.md).

1. **Adaptar, no transcribir.** Las ideas del LF Automatizador se reimplementan ajustadas al contexto de esta app. No copiar código sin entenderlo y adaptarlo.

2. **Soluciones desde la raíz.** Cuando aparece un bug, se busca y corrige la causa real. Añadir condiciones defensivas, silenciar errores o rodear lógica rota con código adicional desplaza el problema sin resolverlo.

3. **200 líneas por archivo, máximo.** Si un módulo crece, se divide. Medir con `wc -l` (POSIX); PowerShell `Measure-Object` puede descontar la última línea y dar un resultado menor al real.

4. **La lógica de negocio vive en Rust.** El frontend dibuja, envía comandos IPC y muestra respuestas. Ningún cálculo de audio, estado crítico ni validación real pertenece a JavaScript.

5. **JavaScript requiere justificación.** Antes de escribir lógica en JS, la pregunta es: *¿puede esto resolverse en Rust?* El camino fácil no es un argumento técnico. JS está justificado para interacciones visuales inmediatas o para APIs del navegador sin equivalente en el backend.

6. **Compatibilidad bidireccional con LF Automatizador.** Todo campo nuevo en el modelo de datos debe tener `#[serde(default)]`. No añadir campos obligatorios al esquema de exportación sin coordinar el cambio en ambas apps.

7. **Cero strings de UI hardcodeados.** Todo texto visible pasa por `t(key)`. Al añadir texto nuevo, añadir la clave en los 4 idiomas: `es.json` (fuente), `en.json`, `pt-BR.json`, `pt-PT.json`.

8. **Tema claro/oscuro sin parpadeo.** Solo CSS custom properties. Los colores de usuario se adaptan con `colorAdapter.js`. Sin cambios de clase que causen flash de pantalla blanca al cargar.

9. **Proponer antes de reestructurar.** Si el cambio afecta la estructura de módulos, el esquema de datos o el flujo IPC, describir el plan y esperar aprobación antes de implementar.

10. **Verificar sin lanzar la aplicación.** Verificar con `cargo test --lib` **y `cargo build --lib`** (backend) y `npm run build` (frontend). Las dos de Rust: `cargo test` compila con los tests, y un `use super::*` de un fichero de pruebas puede tapar un import que ya no usa nadie; el usuario compila sin tests y ahí sí sale el aviso. La prueba visual la hace el usuario en su equipo.

11. **Las IAs no son colaboradoras del proyecto.** Commits, PRs y cualquier contribución registrada van únicamente a nombre de usuarios humanos reales con cuenta de GitHub (ejemplo: "Juan Pérez", "Yo Soy Luis Fernando"). Sin trailers `Co-Authored-By: <nombre de IA>`, sin firmas de asistente, sin atribuciones de IA en mensajes de commit, descripciones de PR ni comentarios de código. El historial de git debe reflejar exclusivamente a personas reales. El reconocimiento al uso de herramientas de IA en el desarrollo está documentado en la sección "Créditos de desarrollo" de `README.md`.

12. **Dependencias con justificación.** Antes de añadir una dependencia, justificar necesidad, mantenimiento, licencia compatible, impacto en build y superficie de seguridad. Preferir código existente, estándar o local si resuelve el problema con menor riesgo.

13. **Documentación junto al cambio.** Si cambia arquitectura, estructura, IPC, modelo de datos, formatos, reglas de negocio o flujos importantes, actualizar la documentación relacionada en el mismo cambio: `ARCHITECTURE.md`, `GLOSARIO.md`, `LIBRO_PROYECTO.md`, `CHANGELOG.md`, `AGENTS.md` o `CLAUDE.md` según corresponda.

14. **Espacio de trabajo limpio.** No dejar planes temporales, artefactos generados, pruebas descartables ni archivos de trabajo en curso dentro del repositorio. Solo código, documentación permanente y configuración necesaria.

---

## 3. Arquitectura en una página

```
┌───────────────────────────────────────────────────────┐
│  Frontend — Vanilla JS + Vite                         │
│  src/js/      (bridge/, ui/, util/)                   │
│  src/css/     (17 hojas de estilo por componente)     │
│  src/public/  (i18n en 4 idiomas)                     │
│                                                       │
│  Acceso a Rust SOLO a través de api.js (bridge/):     │
│    invoke(cmd, args) → resultado                      │
│    listen(evento, fn) → suscripción                   │
└──────────────────────────┬────────────────────────────┘
                           │ IPC (window.__TAURI__)
┌──────────────────────────▼────────────────────────────┐
│  Backend — Rust + Tauri v2                            │
│  src-tauri/src/ (core, model, engine, domain, ipc)    │
│                                                       │
│  AppState {                                           │
│    config:   Arc<Mutex<AppConfig>>      → JSON        │
│    audio:    Mutex<AudioEngine>         → rodio/cpal  │
│    player:   Mutex<PlayerEngine>        → rodio/cpal  │
│    tracks:   Arc<Mutex<TrackStore>>     → SQLite      │
│    waveforms, track_analysis, last_played, history    │
│  }                                                    │
└───────────────────────────────────────────────────────┘
```

**Dos motores de audio independientes.** `audio` reproduce los efectos (mezcla muchas fuentes a la vez). `player` es el reproductor auxiliar del panel lateral (música de fondo): su propio hilo, su propia salida, su propio dispositivo y su propio volumen. Uno no detiene al otro: el Stop general y el Solo de los efectos no cortan la música, y el reproductor tiene su propio Stop.

El frontend **nunca** calcula duración de audio, nivel de señal, si un archivo es válido, cuánto tiempo lleva sonando, ni ningún otro estado crítico. Todo eso lo calcula Rust y lo comunica via IPC o eventos.

---

## 4. Modelo de datos principal

### Jerarquía de la configuración

```
AppConfig
  ├── theme, language, button_text_size, editor_mode
  ├── preload: PreloadConfig
  ├── locutions: LocutionConfig
  ├── fixed_panel: FixedPanelConfig {scope, view: "player"|"buttons", side, columns, rows, width, ...}
  ├── player: PlayerConfig          ← reproductor auxiliar, global (uno solo)
  │     ├── tracks: Vec<ButtonData>   (la cola; reutiliza ButtonData, admite todos los tipos)
  │     ├── playback_mode: "normal"|"repeat"|"random"
  │     ├── volume: f32               (0.0..=1.5, independiente del master)
  │     └── output_device: String     ("" = el mismo de los efectos)
  └── profiles: Vec<ProfileData>
        └── ProfileData
              ├── id, name, bg, text
              ├── audio: AudioConfig {out_main, out_pre, playback_mode, master_volume, ...}
              ├── active_paleta_id
              ├── fixed_buttons: Vec<ButtonData>
              └── paletas: Vec<PaletaData>
                    └── PaletaData
                          ├── id, nombre, rows, cols, audio_out, shortcut, tab_bg, tab_text
                          └── botones: Vec<ButtonData>
                                └── ButtonData
                                      ├── id: "{paleta_id}_btn_{index}"
                                      ├── type: "audio"|"time"|"temperature"|"humidity"|"random_folder"
                                      ├── path, folder, name, label
                                      ├── color_bg, color_text
                                      ├── vol: f32   (multiplicador lineal, NO en dB)
                                      ├── duration: f64, duration_str: String
                                      ├── loop_mode, stop_other, overlap, restart: bool
                                      └── shortcut: String
```

### Metadatos de pista (SQLite `tracks.db`)

```
TrackMeta (1 fila por archivo)
  path: String      ← clave PK, lowercase en Windows, literal en Linux
  mtime: i64        ← epoch de modificación del archivo (invalida la fila si cambia)
  size: i64         ← tamaño en bytes (verificación secundaria)
  duration_s, sample_rate, channels
  cue_start_s: f64  ← punto de inicio manual (usuario, editor de pistas)
  cue_end_s: f64?   ← punto de fin (None = hasta el final)
  gain_db: f64      ← ajuste manual en dB (usuario)
  norm_enabled: bool, norm_gain_db: f64  ← normalización automática
  measured_peak_db, measured_lufs: f64?
  last_played: i64? ← epoch de última reproducción (precarga OnPlay)
```

---

## 5. Pipeline de audio

```
invoke('play_button', {id}) ─► cmd_button_playback::play_button_id()
     │
     ├── Lee AppConfig (RAM, sin I/O)
     ├── Resuelve tipo de botón (audio / time / temperature / random_folder)
     ├── Consulta tracks.db: cue, dB, mtime/size
     └── audio::AudioEngine::play_file() ─► AudioCommand::Play ─► canal mpsc
                                                                         │
                                                              engine/audio/thread.rs (hilo)
                                                                         │
                                                   engine/cache/preload.rs::build_play_source()
                                                         ├── Hit → CachedSource::new_at() O(1)
                                                         └── Miss → decode + CuedSource O(n)
                                                                         │
                                          routing::bus_for(to_pre, group) → BusId
                                                                         │
                                                    attach_button() → bus de la consola
                                                                         │
                                                   ButtonSource: s × file_gain × vol_btn × fade
                                                                         │
                                    engine/console: DynamicMixer → FaderSource → LevelSource
                                                                         │
                                     Efectos, Panel, Reproductor ─► Programa (fader = MASTER)
                                              Cue ─────────────────────┐  │
                                                                       ▼  ▼
                                                          OutputEndpoint (play_raw)
                                                                                              │
                                                                                         Altavoces
```

**Modelo de ganancia — cada factor en su etapa:**
```
ButtonSource (canal):     muestra × file_gain(dB→lineal) × vol_botón(lineal 0-1) × fade
Bus del grupo (fader):    × fader_del_bus (Efectos / Panel / Cue; hoy a 1.0)
Bus Programa (master):    × master(lineal 0-1.5) — solo lo que suma en él
```
El `master` es el **fader del bus `Programa`**: una etapa real sobre la suma, no un número que
cada fuente se aplica a sí misma. Se le pide a la consola (`console.fader(BusId::Programa)`), no
al motor de efectos. **El medidor va después del fader**, así que el vúmetro enseña lo que de
verdad sale.

**La consola (`engine/console/`)** es dueña de las salidas físicas y de los buses; el motor
de efectos le pide un bus y le entrega fuentes. Reproducir NO pasa por su hilo: el controller del
bus es `Arc<...>` y `add()` toma `&self`, así que cada motor añade desde el suyo. El hilo guardián
solo atiende ruteo, porque `OutputStream` no es `Send` y alguien tiene que ser su dueño.

**Pre-escucha:** `to_pre=true` va **siempre** al bus `BusId::Cue`, sin fallback. Si no tiene
tarjeta propia usa `Routing::ProgramDevice`: sale por la tarjeta del programa pero con su propio
enchufe, su fader y su medidor — se suma con el programa **en el conector, no en el bus**. Así no
le pega el master ni cuenta en el vúmetro aunque solo haya una tarjeta. `sanitize` impide rutear
el CUE a `Program` aunque se pida.

---

## 6. Eventos Rust → Frontend

| Evento Tauri | Payload | Quién lo consume |
|---|---|---|
| `"audio-tick"` | `{buttons[{group, progress_percent, ...}], display_remaining, display_duration, master_level_l, master_level_r}` | gridPlayback.js, fixedPanel.js, clockWidget.js, vuMeter.js, tabs.js |
| `"player-tick"` | `PlayerSnapshot {playing, path, position_s, duration_s, current_index, next_index, mode, stop_after, queue_len}` | runtimeEvents.js → playerView.js (verde = `current_index`, naranja = `next_index`) |
| `"clock-tick"` | `{time_str, date_str}` | clockWidget.js |
| `"weather-updated"` | datos de clima | settingsLocutions.js |
| `"global-shortcut-refresh"` | — | startup.js → recarga la UI |
| `"track-editor-dock"` | `{path, name, zoom}` | startup.js → abre editor en modal |
| `"track-analysis-progress"` | `{path, stage}` | trackEditor.js → actualiza progreso del análisis |
| `"theme-changed"` | `{theme}` | ventana pop-out del editor |

`startup.js` también dispara el `CustomEvent('lf-audio-tick')` en el DOM cada vez que recibe `"audio-tick"` de Rust. Los módulos del editor usan `window.addEventListener('lf-audio-tick', ...)` porque es un `CustomEvent`, no un evento Tauri.

**Por qué el reproductor tiene su propio tick:** `"audio-tick"` no se emite en reposo (si no hay efectos sonando, calla). La música de fondo suele sonar sin ningún efecto disparado, así que colgar la lista de ese tick la dejaba sin pintar. `"player-tick"` es independiente, como lo son los dos motores.

---

## 7. Comandos IPC (resumen)

Para la lista completa ver [`CLAUDE.md §9`](CLAUDE.md).

**Los más frecuentes:**
- `get_config` → `AppConfig` completo
- `get_grid_state` → rejilla de la paleta activa
- `play_button(id)` → disparo principal de un botón
- `play_audio(id, path, volume, ...)` → reproducción directa (pre-escucha, editor)
- `stop_audio(id)`, `stop_all_audio`
- `analyze_track(path)` → análisis del editor con caché/progreso + envolvente de onda
- `set_track_cue(path, start, end?)` / `set_track_gain(path, dB)` / `set_track_normalization(path, enabled)`
- `update_button_data(paleta_id, index, data)` → guarda edición de botón
- `export_tab_by_id(paleta_id)` / `import_tab()` → formatos .bdelf
- `set_editor_mode(mode)` → "modal" | "window"; persiste en AppConfig

**Reproductor auxiliar** (motor propio; los índices son POSICIONES 0-based en la cola):
- `get_player` → cola + ajustes + estado; `get_player_snapshot` → solo el estado en vivo
- `player_play_index(index)` / `player_activate_index(index)` → el motor decide si reproduce o marca
- `player_next` / `player_prev` / `player_stop` / `player_pause` / `player_resume`
- `player_mark_next(index?)` → marcar siguiente. **Es ley:** se respeta en todos los modos
- `player_set_mode(mode)` → "normal" | "repeat" | "random". El modo dice QUÉ pista viene; que se pare al acabar lo decide `player_set_stop_after`, que se combina con los tres
- `player_set_stop_after(enabled)` → al acabar la actual, no arranca sola
- `player_set_volume(volume)` / `player_set_device(device)` → salida propia ("" = la de los efectos)
- `player_add_track(path, index?)` / `player_add_button(buttonId, index?)` → sin `index`, al final
- `player_remove_track(index)` / `player_reorder_tracks(from, to)` / `player_clear_queue`
- `player_save_playlist` / `player_open_playlist` → formato `.LFPlay` (compatible con LFA)

---

## 8. Mapa de archivos clave

Para el mapa completo, ver [`Documentación/LIBRO_PROYECTO.md §3 y §4`](Documentación/LIBRO_PROYECTO.md).

**Los más importantes para entender el sistema:**

| Archivo | Por qué es central |
|---|---|
| `src-tauri/src/core/state.rs` | Define `AppState` y mantiene todo el estado de la app |
| `src-tauri/src/core/setup.rs` | Inicializa la app: dispositivo, hilos, precarga |
| `src-tauri/src/model/` | Esquema de datos completo serializable |
| `src-tauri/src/engine/persist/config_io.rs` | Persistencia JSON + migración automática |
| `src-tauri/src/engine/audio/thread.rs` | Hilo de los efectos: comandos, estados y fades |
| `src-tauri/src/engine/console/thread.rs` | Hilo guardián: único dueño de las tarjetas abiertas |
| `src-tauri/src/engine/console/bus.rs` | Un bus: mezcla de fuentes + medición de nivel |
| `src-tauri/src/engine/console/endpoint.rs` | Una tarjeta física abierta exactamente una vez |
| `src-tauri/src/engine/player/thread.rs` | Hilo del reproductor auxiliar: único dueño de su salida y sus dos decks |
| `src-tauri/src/domain/player/advance.rs` | Regla pura de los tres modos: qué pista suena después |
| `src-tauri/src/ipc/cmd_button_playback.rs` | Lógica completa de disparo de un botón |
| `src-tauri/src/engine/persist/tracks.rs` | CRUD de metadatos de pista en SQLite |
| `src/js/bridge/api.js` | Única puerta de acceso al IPC desde el frontend |
| `src/js/ui/startup.js` | Bootstrap completo de la UI |
| `src/js/ui/trackEditor.js` | Orquestador del editor de pistas |

---

## 9. Cómo añadir una nueva función

### Nuevo comando IPC
1. Crear o elegir el `src-tauri/src/ipc/cmd_*.rs` correspondiente.
2. Añadir `#[tauri::command]` a la función.
3. Registrar en `src-tauri/src/ipc/register.rs` dentro de `lf_invoke_handlers!`.
4. Llamar desde el frontend a través de `src/js/bridge/api.js`.
5. Si modifica `AppConfig`, llamar `engine::persist::config_io::save_config(&cfg)` al final.

### Nuevo tipo de botón
1. Añadir el caso en `src-tauri/src/domain/button/types.rs`.
2. Añadir la rama en `src-tauri/src/ipc/cmd_button_playback.rs`.
3. Añadir los campos de UI en `src/js/ui/editTypes.js`.
4. Añadir las claves i18n en los 4 idiomas.

### Nueva clave i18n
1. Añadir en `src/public/i18n/es.json` (fuente de verdad).
2. Añadir la misma clave en `en.json`, `pt-BR.json` y `pt-PT.json`.

### Nuevo campo en el modelo de datos
1. Añadir con `#[serde(default)]` al struct correspondiente (obligatorio).
2. Implementar `Default` o usar `#[serde(default = "fn")]`.
3. Añadir getter/setter IPC si la UI necesita leerlo o modificarlo.

---

## 10. Trampas frecuentes

### `window.__TAURI__` es `undefined` al parsear módulos
En producción (WebView2), el objeto global se inyecta después de que los módulos JS se cargan. Si se captura al nivel del módulo (`const tauri = window.__TAURI__`), queda `undefined` permanentemente. **Solución:** resolverlo dentro del cuerpo de cada función, como hace `api.js`.

### `lf-audio-tick` no es un evento Tauri
Es un `CustomEvent` del DOM que dispara `startup.js`. Usar `window.addEventListener('lf-audio-tick', fn)`, nunca `api.listen('lf-audio-tick', fn)`.

### IDs de botón: formato actual vs. legado
El formato actual es `{paleta_id}_btn_{index}` (ejemplo: `paleta_1_btn_3`). El formato antiguo era `btn_{index}` y colisionaba entre paletas. `config_io.rs::normalize_button_ids()` migra automáticamente al cargar; no producir IDs en el formato viejo.

### El hilo de audio no hace I/O
El hilo de audio (`engine/audio/thread.rs`) **no** accede al disco. La decodificación de reproducción ocurre en `engine/cache/preload.rs::build_play_source()` que llama a `engine/audio/decode.rs`. El análisis del editor se lanza desde IPC en un worker bloqueante (`engine/dsp/editor_analysis.rs`), nunca en el hilo de audio.

### `tracks.db` vs `botonera_config.json`
Los datos de los **botones** (qué archivo, qué volumen, qué loop) viven en `botonera_config.json`. Los **metadatos del archivo** (cue, dB, LUFS, mtime) viven en `tracks.db`. Son dos persistencias independientes. El cue y el dB NO viajan en `botonera_config.json`.

### `capabilities/default.json` sin BOM
El parser de Tauri rechaza este archivo si tiene marca BOM (Byte Order Mark). Guardarlo siempre como UTF-8 sin BOM.

### Sincronización de versión
Al publicar una nueva versión, los tres archivos siguientes deben coincidir:
- `package.json` → `"version"`
- `src-tauri/Cargo.toml` → `version` en `[package]`
- `src-tauri/tauri.conf.json` → `"version"`

---

## 11. Cómo verificar un cambio

```bash
# Tests unitarios Rust (suite actual: 209 passed, 4 ignored)
cd src-tauri
cargo test --lib

# Build del frontend (debe completar sin errores)
cd ..
npm run build

# Contar líneas de un archivo Rust (límite: 200)
wc -l src-tauri/src/<modulo>.rs
```

La prueba funcional la hace el usuario en su equipo. No hay harness de integración para el webview.

**Al publicar una nueva versión:**
1. Actualizar `CHANGELOG.md`: añadir las entradas de `[Sin publicar]`, renombrar esa sección a `[X.Y.Z] — YYYY-MM-DD`, crear una nueva `[Sin publicar]` vacía encima, y añadir el enlace comparativo al pie.
2. Ejecutar `SET-VERSION.bat X.Y.Z` — sincroniza `package.json`, `Cargo.toml` y `tauri.conf.json`.
3. `cd src-tauri && cargo check` — regenera `Cargo.lock`.
4. `git commit -am "Release X.Y.Z"` → `git tag vX.Y.Z` → `git push && git push --tags`.

---

## 12. Estado del proyecto y pendientes

**Versión 1.2.1 — funcionalidades completas:**
- Perfiles, paletas, botones (tipos: audio, time, temperature, humidity, random_folder)
- Motor de audio: loop, overlap, restart, stop_other, pre-escucha independiente
- Modos de reproducción global: normal, loop, overlap, restart + solo mode
- Atajos de teclado locales y globales del SO
- Editor de pistas: cue, normalización dB/LUFS, onda, transporte, pop-out a ventana
- Precarga de audio RAM (LRU, estrategias FullProfile/VisibleTabs/OnPlay, TTL)
- Locuciones dinámicas de hora y clima (open-meteo)
- Export/import `.bdelf`/`.bdeplf` con portabilidad de cue/dB
- Panel lateral fijo con botones globales o por perfil
- Reproductor auxiliar con cola y listas `.LFPlay` compatibles con LF Automatizador
- i18n en 4 idiomas (es, en, pt-BR, pt-PT)
- CI/CD con GitHub Actions

**En desarrollo (rama `codex/distribucion-tiendas`):**
- Microsoft Store completada con la versión 1.2.1.
- Prueba física prioritaria en Linux y preparación posterior de Flathub.
- Canales actuales centralizados: `direct` para GitHub Releases y `store` para
  Microsoft Store; los canales administrados de Linux todavía no están implementados.
- Plan y evidencia en
  [`Documentación/PLAN_DISTRIBUCION_TIENDAS.md`](Documentación/PLAN_DISTRIBUCION_TIENDAS.md).

**Pendientes conocidos:**

**A — Prueba física en Linux**
El código es multiplataforma (rutas via `config::get_data_dir()`, SQLite bundled, rodio/ALSA). Falta probar el build (`.deb`, `.AppImage`) en una máquina Linux real.

**B — Deuda menor: `master_volume` es `f32`**
Su representación en JSON crece sola al guardar (`0.45` → `0.4499999…`). Inocuo, pero ensucia el fichero. Afecta a `AudioConfig` y al `vol` de `ButtonData`.

**Política de colores de los botones nuevos: DESCARTADA** (2026-07-16). El autor la vio complicada de explicar y de usar. En su lugar existe la **selección múltiple** (Ctrl+clic y clic derecho → pintar: `buttonSelection.js` + `set_buttons_color`). **No volver a proponerla.** [`Documentación/PLAN_POLITICA_COLORES.md`](Documentación/PLAN_POLITICA_COLORES.md) se conserva solo como registro de lo que se decidió.
