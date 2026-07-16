# CLAUDE.md — LF Botonera de Efectos

Guía de contexto completa para una IA que colabora en este proyecto.
Lee todo antes de tocar código.

> **¿Retomas el trabajo o acabas de perder el hilo de la conversación?**
> Lee primero [`Documentación/CONTINUIDAD_SESION.md`](Documentación/CONTINUIDAD_SESION.md):
> estado actual, decisiones ya cerradas y trampas verificadas. Evita rehacer lo hecho.

---

## 1. Identidad del proyecto

**LF Botonera de Efectos** es una botonera de sonidos (soundboard) para radio y streaming en directo. El operador asigna archivos de audio a botones organizados en paletas (pestañas) dentro de perfiles, y los dispara en tiempo real.

- **Versión actual:** 1.1.2
- **Repositorio:** `C:\OVERLAY\BOTONERA`
- **GitHub:** https://github.com/yosoyluisfernando/LF-Botonera-de-efectos
- **Autor:** Luis Fernando Velásquez
- **Licencia:** GPL-3.0-or-later
- **App hermana:** LF Automatizador v1.0 (`C:\LF Automatizador v1.0`) — comparten formatos `.bdelf` / `.bdeplf`
- **Datos de usuario:** `%APPDATA%\LF Botonera\` (Windows) / `~/.config/LF Botonera/` (Linux)

---

## 2. Stack técnico

| Capa | Tecnología |
|---|---|
| Backend | Rust 2021 + Tauri v2 |
| Audio | rodio 0.19 + symphonia (codec) + opus-decoder + ebur128 (LUFS) |
| Persistencia config | serde_json → `botonera_config.json` en `%APPDATA%` |
| Persistencia pistas | rusqlite (feature `bundled`, SQLite estático) → `tracks.db` |
| HTTP | ureq (clima open-meteo) |
| Fecha/hora | chrono |
| Frontend | Vanilla JS (módulos ES) + Vite (bundler) |
| i18n | JSON en `src/public/i18n/{es,en,pt-BR,pt-PT}.json` |
| Temas | CSS custom properties (sin JS en tiempo de render) |
| Build | `npm run tauri build` → `.msi`/`.exe`/`.deb`/`.rpm`/`.AppImage` |
| CI | GitHub Actions (`.github/workflows/build.yml`, `release-builds.yml`) |

**Tauri config clave:** `withGlobalTauri: true` — el objeto `window.__TAURI__` se inyecta en el webview. `api.js` lo resuelve en el *cuerpo* de cada función, nunca al nivel del módulo (si se capturara al parsear, quedaría `undefined` permanentemente en producción).

---

## 3. Principios de desarrollo (aplicar siempre)

1. **Adaptar, no transcribir.** Las ideas del LFA se reimplementan ajustadas al contexto de esta app; no se copia/pega código sin entenderlo y adaptarlo.
2. **Soluciones desde la raíz.** Cuando aparece un bug, se busca y corrige la causa real. Añadir condiciones defensivas, silenciar errores o rodear lógica rota con código adicional desplaza el problema sin resolverlo.
3. **Límite 200 líneas por archivo.** Medir con `wc -l` (POSIX). PowerShell `Measure-Object -Line` puede descontar la última línea sin salto y dar un resultado menor al real.
4. **La UI es un "humilde control remoto".** Todo audio, DSP, timers, validaciones y lógica de negocio va en Rust. El frontend dibuja y envía comandos IPC; nada más.
5. **JavaScript requiere justificación.** Antes de escribir lógica en JS, la pregunta es: *¿puede esto vivir en Rust?* El camino fácil no es criterio válido. JS solo está justificado para interacciones visuales inmediatas o para APIs del navegador sin equivalente en el backend.
6. **Compatibilidad bidireccional** con LF Automatizador en `.bdelf`/`.bdeplf`. Cualquier campo nuevo debe ser OPCIONAL y con `#[serde(default)]` para que la otra app lo ignore sin romperse.
7. **i18n siempre.** Cero strings visibles hardcodeados en la UI; todo pasa por `t()` y archivos JSON.
8. **Tema claro/oscuro dinámico.** Solo CSS custom properties; sin cambios de clase que causen parpadeo blanco.
9. **Las IAs no son colaboradoras del proyecto.** Commits, PRs y cualquier contribución registrada en el repositorio van únicamente a nombre de usuarios humanos reales con cuenta de GitHub (ejemplo: "Yo Soy Luis Fernando"). Sin trailers `Co-Authored-By: Claude`, sin firmas de IA, sin menciones a asistentes en mensajes de commit, descripciones de PR ni comentarios de código. El historial de git debe reflejar solo a personas reales. El reconocimiento al uso de herramientas de IA durante el desarrollo está documentado en la sección "Créditos de desarrollo" de `README.md`.
10. **No usar computer-use.** Controlar la pantalla del usuario cuesta tokens. Verificar con `cargo test --lib`, `cargo build --lib` (sin tests: `cargo test` los compila y un `use super::*` puede tapar un import muerto) y `npm run build`; el usuario prueba en su PC.
11. **Conversar antes de tocar código.** Ante dudas de arquitectura, preguntar.

---

## 4. Modelo de datos

### AppConfig (raíz del fichero de configuración)
```
AppConfig {
  is_first_boot: bool,           // false tras completar el wizard
  weather_module_enabled: bool,  // activa el módulo de locuciones de clima
  lf_automatizador_link: bool,   // (futuro) enlace con LFA
  theme: String,                 // "dark" | "light" | "auto"
  language: String,              // "es" | "en" | "pt-BR" | "pt-PT"
  button_text_size: String,      // "small" | "normal" | "large"
  editor_mode: String,           // "modal" | "window" — persiste cómo abrió el editor
  active_profile_id: String,
  clock_24h: bool,
  last_update_check: i64,        // epoch de la última comprobación de actualizaciones
  locutions: LocutionConfig,
  preload: PreloadConfig,
  norm: NormConfig,              // normalizador automático (engranaje del editor de pistas)
  cue_detect: CueDetectConfig,   // detección automática de puntos de inicio/fin
  norm_prompted: bool,
  fade: FadeConfig,              // fundidos globales
  playback_progress: PlaybackProgressConfig, // paso del salto (1|2|5|10|20|30 s)
  waveform_cache: WaveformCacheConfig,       // caché persistente de onda del editor
  startup: StartupPromptState,
  fixed_panel: FixedPanelConfig, // alcance, vista, lado, dimensiones, capacidad y botones
  player: PlayerConfig,          // reproductor auxiliar: uno solo y GLOBAL (no por perfil)
  profiles: Vec<ProfileData>,
}
```

### ProfileData
```
ProfileData {
  id: String,                  // timestamp + random, único
  name: String,
  bg: String,                  // color HEX del perfil
  text: String,
  audio: AudioConfig,
  active_paleta_id: String,
  paletas: Vec<PaletaData>,
  fixed_buttons: Vec<ButtonData>, // botones del panel cuando el alcance es por perfil
}
```

### AudioConfig (por perfil)
```
AudioConfig {
  out_main: String,             // nombre de dispositivo CPAL o "default"
  out_pre: String,              // dispositivo de pre-escucha (vacío = fallback a out_main)
  global_keys: bool,
  key_stop: String,             // atajo global "Detener todo" (ej. "Ctrl+Space")
  key_next: String,             // siguiente pestaña
  key_prev: String,             // pestaña anterior
  playback_mode: String,        // "normal" | "loop" | "overlap" | "restart"
  solo_mode: bool,              // true = Stop Others al reproducir
  master_volume: f32,           // 0.0–1.5 (1.5 solo en modo boost)
  master_volume_remember: bool,
  master_volume_boost: bool,
}
```

### PaletaData
```
PaletaData {
  id: String,                  // "paleta_1", "paleta_2", …
  nombre: String,
  rows: u32,                   // default 5
  cols: u32,                   // default 5
  audio_out: String,           // "" = usa out_main del perfil
  shortcut: String,
  tab_bg: String,
  tab_text: String,
  botones: Vec<ButtonData>,
}
```

### ButtonData
```
ButtonData {
  id: String,        // "{paleta_id}_btn_{index}" — único por paleta
  index: u32,        // posición en la rejilla (1-based)
  label: String,
  type: String,      // "audio" | "time" | "temperature" | "humidity" | "random_folder"
                     // (serde: rename = "type", campo Rust = type_field)
  path: String,      // ruta al archivo de audio
  folder: String,    // carpeta (para time/temperature/humidity/random_folder)
  name: String,      // nombre para mostrar
  color_bg: String,
  color_text: String,
  vol: f32,          // multiplicador lineal 0–1 (trim por botón; Regla 5: no cambia)
  duration: f64,     // duración del archivo en segundos
  duration_str: String,
  loop_mode: bool,
  stop_other: bool,
  overlap: bool,
  restart: bool,
  shortcut: String,
}
```

**Notas de IDs de botón:** formato `{paleta_id}_btn_{index}`. `config_io.rs` normaliza al cargar para migrar el formato antiguo `btn_{index}` que colisionaba entre paletas.

### TrackMeta (SQLite, 1 fila por archivo)
```
TrackMeta {
  path: String,            // clave PK, normalizada por db::normalize_key()
  mtime: i64,              // epoch de modificación (invalida la fila si cambia)
  size: i64,               // tamaño en bytes (verificación secundaria)
  duration_s: f64,
  sample_rate: u32,
  channels: u16,
  cue_start_s: f64,        // punto de inicio manual (usuario)
  cue_end_s: Option<f64>,  // punto de fin (None = hasta el final)
  gain_db: f64,            // trim manual en dB (usuario)
  norm_enabled: bool,
  norm_gain_db: f64,       // ganancia calculada por el normalizador automático
  measured_peak_db: Option<f64>,
  measured_lufs: Option<f64>,
  analyzed_at: Option<i64>,
  last_played: Option<i64>,
}
```

**Normalización de clave en disco:**
- Windows: `path.to_lowercase()` (sistema de archivos case-insensitive)
- Linux: `path.to_string()` (case-sensitive)

### PlayerConfig (dentro de AppConfig) — reproductor auxiliar
```
PlayerConfig {
  tracks: Vec<ButtonData>,     // cola ordenada; reutiliza ButtonData → soporta los 5 tipos
  playback_mode: String,       // "normal" | "repeat" | "random" (NO existe "manual": se quitó)
  volume: f32,                 // propio, independiente del master (0.0–1.5; la UI expone 0–100 %)
  output_device: String,       // "" = el mismo de los efectos; "default" = del sistema
  time_display: String,        // "elapsed" | "remaining"
  large_folder_action: String, // "ask" | "always" | "never" (carpeta > 250 canciones)
}
```
**No se persisten** (son de transporte, arrancan apagados): el botón Loop y "detener al finalizar".

### PreloadConfig (dentro de AppConfig)
```
PreloadConfig {
  enabled: bool,              // default true
  ram_budget_mb: u32,         // 32 | 64 | 128 | 256 (default 128)
  max_duration_s: u32,        // umbral: solo precargar si duración < este valor
  strategy: PreloadStrategy,  // FullProfile | VisibleTabs | OnPlay
  evict_after_hours: u32,     // TTL para expulsar de la caché (default 72)
  prompted: bool,
}
```

### LocutionConfig (dentro de AppConfig)
```
LocutionConfig {
  time_enabled: bool,
  time_folder: String,
  weather_enabled: bool,
  temp_folder: String,
  hum_folder: String,
  weather_city: String,
  weather_lat: f64,
  weather_lon: f64,
  weather_unit: String,  // "metric" | "imperial"
}
```

---

## 5. Esquema SQLite (`tracks.db`)

```sql
CREATE TABLE IF NOT EXISTS track (
  path             TEXT PRIMARY KEY,
  mtime            INTEGER NOT NULL,
  size             INTEGER NOT NULL,
  duration_s       REAL    NOT NULL,
  sample_rate      INTEGER NOT NULL,
  channels         INTEGER NOT NULL,
  cue_start_s      REAL    NOT NULL DEFAULT 0,
  cue_end_s        REAL,
  gain_db          REAL    NOT NULL DEFAULT 0,
  norm_enabled     INTEGER NOT NULL DEFAULT 0,
  norm_gain_db     REAL    NOT NULL DEFAULT 0,
  measured_peak_db REAL,
  measured_lufs    REAL,
  analyzed_at      INTEGER,
  last_played      INTEGER
);
```

- WAL habilitado (`PRAGMA journal_mode=WAL`) para escrituras frecuentes baratas.
- Versión del esquema en `PRAGMA user_version` (actualmente 1).
- `last_played` se vuelca desde memoria a disco cada 30 s (debounce) y al cerrar.

---

## 6. Motor de audio — pipeline

```
IPC invoke("play_button", id)
    │
    ▼
cmd_button_playback::play_button_id()
    ├── Lee AppConfig (perfil activo, modo de reproducción, flags del botón)
    ├── Consulta tracks.db → ResolvedEdit {cue_start, cue_end, file_gain, duration}
    └── audio::AudioEngine::play_file(...) → Sender<AudioCommand>
                                                    │
                                  ┌─────────────────┘
                                  ▼
                           engine/audio/thread.rs (hilo dedicado)
                                  │
                          AudioCommand::Play{to_pre}
                                  │
                    │
                    ▼
            routing::bus_for(to_pre, group) → BusId
              ├── to_pre       → BusId::Cue
              ├── group=Main   → BusId::Efectos
              └── group=Fixed  → BusId::Panel
                    │
                    ▼
            console.bus(bus)   ← sin fallback: cada bus existe por su cuenta
                    │
                    ▼
            engine/cache/preload.rs::build_play_source(path, loop, cue)
                    ├── Cache HIT → CachedSource::new_at(pcm, offset) — O(1) seek
                    └── Cache MISS → audio_decode::source_from_path() + CuedSource (O(n))
                                          │
                                          ▼
                    engine/audio/attach.rs::attach_button(bus, source, args)
                                          │
                                          ▼
                               ButtonSource { inner, stop_flag, done_flag, file_gain, volume, master_volume }
                               (implementa Iterator<Item=f32>; aplica: s × file_gain × vol_btn × master)
                                          │
                                          ▼
                    ══ engine/console/ ═══════════════════════════════
                               Bus: DynamicMixer → FaderSource → LevelSource
                                          │
                    ┌─────────────────────┴──────────────────┐
                    │ Efectos, Panel: Routing::Program        │ Cue: ProgramDevice
                    ▼                                         ▼
        Programa: mixer → fader(MASTER) → level      play_raw (su propio
                    │                                 enchufe en la tarjeta)
                    ▼                                         │
        play_raw → OutputEndpoint (una tarjeta, abierta 1 vez) ◄──┘
                    │
                    ▼
        mixer interno de OutputStream → dispositivo CPAL
```

**La topología:**
```
  Efectos ──────┐
  Panel ────────┼─► Programa ─► fader (MASTER) ─► medidor (vúmetro) ─► tarjeta
  Reproductor ──┘   (su fader = volumen del reproductor)

  Cue ──────────────────────► fader ──────────► medidor ──────────► tarjeta
```

**La consola (`engine/console/`)** es dueña de las salidas y de los buses. El motor de efectos
le pide un bus y le entrega fuentes. **Reproducir NO pasa por su hilo:** el controller del bus
es `Arc<DynamicMixerController<f32>>` (Send + Sync) y `add()` toma `&self`. El hilo guardián
solo atiende ruteo, porque `OutputStream` no es `Send` y alguien debe ser su dueño.

**El `Routing` de cada bus** (`domain/console/routing.rs`) dice a dónde entrega:
- `Program` — suma en el programa: le pega el master y lo cuenta su vúmetro.
- `ProgramDevice` — sale por la **misma tarjeta** que el programa, pero **sin sumar en él**.
  Esta variante es la idea entera de la consola: que dos cosas salgan por el mismo altavoz no
  las convierte en la misma señal. Se suman en el *conector*, no en el *bus*.
- `Device(x)` — su propia tarjeta, ajeno al programa y al master.

**El grafo se reconstruye entero ante cualquier cambio de ruteo** (`engine/console/graph.rs`), y
no es pereza: rodio no sabe sacar una fuente de un mixer. Remendar dejaría el mixer de un bus
viejo colgado dentro del programa para siempre, sonando a silencio y sin que nadie pueda quitarlo.

**Trampa: soltar un `Bus` NO lo retira de su destino.** A un mixer de rodio no se le quita una
fuente; la única forma de sacar algo es que **se agote**. Por eso `Bus` tiene un `close()` que
cierra su grifo (`BusOutlet` devuelve `None` y el padre lo deja caer), y `rebuild` cierra los
viejos **antes** de soltarlos. Sin eso, un bus soltado sigue vivo dentro de la tarjeta: como los
atómicos del medidor son del `BusSlot` y sobreviven a la reconstrucción, **el bus viejo y el nuevo
escriben el mismo nivel a la vez** — y el viejo, ya sin fuentes, escribe cero. El vúmetro
parpadea. Fue un bug real (2026-07-16); lo cubre `bus_tests.rs`.

**Pero cambiar de tarjeta NO calla nada.** Las fuentes mueren con sus buses, así que **se vuelven
a crear en el segundo por el que iban**. Los motores se enteran por la **generación** de la
consola (`console.generation()`), que sube en cada `rebuild`:
- **Efectos y panel:** el hilo de audio lo hace en el acto — `set_bus_routing_sync` (esperar es
  obligatorio: entregar antes sería dárselo al bus muerto) y `reattach::reattach_all`.
- **Reproductor:** lo detecta en su tick comparando la generación, y **antes** de mirar si el deck
  terminó — si no, lo tomaría por fin de pista y saltaría de canción.

La ficha para rehacer (`ReplayInfo`) va en el **`ButtonState`**, no en un mapa por id: con
`overlap` hay varias instancias del mismo botón sonando cada una por su sitio, y una ficha
compartida las pondría a todas en la misma posición. Las locuciones llevan `replay: None` — son
varios archivos encadenados y no se reposicionan.

**Modelo de ganancia — cada factor en su etapa:**
```
ButtonSource (el canal):     muestra × file_gain(dB→lineal) × vol_botón(lineal) × fade
Bus del grupo (su fader):    × fader_del_bus  (Efectos / Panel / Cue; hoy a 1.0)
Bus Programa (el master):    × master(lineal) — solo para los que suman en él
```
- `file_gain`: de `TrackMeta.effective_gain_linear()` = norm_gain_db + gain_db → lineal
- `vol_botón`: `ButtonData.vol` (lineal 0–1; se preserva para compat `.bdelf`)
- `master`: `AudioConfig.master_volume` (0–1.5 en modo boost). Es el **fader del bus `Programa`**,
  y se pide a la consola (`console.fader(BusId::Programa)`), no al motor de efectos: el programa
  es la suma de varios buses, no de uno. **La pre-escucha no lo cruza nunca**

Desde la Fase 2 el master **es una etapa real**. Antes era un atómico que cada `ButtonSource`
leía y se aplicaba a sí mismo: no una etapa, sino un acuerdo entre fuentes. El resultado audible
es el mismo (`Σ(sᵢ × m)` = `Σ(sᵢ) × m`), pero ahora sale una multiplicación y una lectura atómica
por muestra en vez de una por cada fuente sonando.

**El medidor va DESPUÉS del fader** (`mixer → FaderSource → LevelSource → play_raw`): el vúmetro
enseña lo que de verdad sale, igual que el medidor de programa de una consola. Al revés no se
enteraría de los movimientos del fader.

**El reproductor no obedece al master** porque es un motor aparte con su propio `OutputStream`.
La Fase 4 lo une a la consola y pasa a obedecerlo (decisión del autor, 2026-07-16) — en volumen,
no en transporte: el Stop general y el Solo siguen sin tocarlo.

**Pre-escucha:**
- ID especial `__prelisten__` para la barra de pre-escucha
- ID especial `__track_preview__` para la previa dentro del editor de pistas
- Ambos van con `to_pre=true` → **siempre** al bus `BusId::Cue`. Sin fallback.

**El fallback que había (arreglado en la Fase 3):** `out_pre` viene vacío por defecto, y entonces
el bus de pre-escucha no existía y la pre-escucha caía al principal, donde le pegaba el master y
sumaba al vúmetro de programa. Con tarjeta dedicada no pasaba: el mismo botón se comportaba
distinto según el equipo.

Ahora el bus `Cue` **existe siempre**. Sin tarjeta propia usa `Routing::ProgramDevice`: sale por
la tarjeta del programa, pero con su propio `play_raw`, su propio fader y su propio medidor. Se
suma con el programa **en el conector**, no en el bus. `domain/console/routing.rs::sanitize`
garantiza que el CUE no pueda rutearse a `Program` ni pidiéndolo — no es programación defensiva,
es la regla: una escucha privada que se cuela en el aire no es una escucha privada.

---

## 7. Hilos en segundo plano

| Hilo | Módulo | Descripción |
|---|---|---|
| Consola | `engine/console/thread.rs` | Hilo guardián: **único dueño de las tarjetas abiertas** (`OutputStream` no es Send). Solo atiende ruteo; reproducir no pasa por aquí |
| Audio | `engine/audio/thread.rs` | Motor de efectos: comandos, estados de botón y fades |
| Monitor | `engine/audio/monitor.rs` | Emite `"audio-tick"` cada 100 ms con progreso + VU. **En reposo calla**; reposo = ni efectos ni reproductor, porque los dos suman en el bus que mide el vúmetro |
| Monitor reproductor | `engine/player/monitor.rs` | Emite `"player-tick"` cada 100 ms. Propio, porque el reproductor tiene su cola y su transporte y suena sin efectos |
| Reloj | `cmd_meta` | Emite `"clock-tick"` cada 1 s con hora y fecha localizadas |
| Historial | `last_played` | Vuelca buffer en memoria a tracks.db cada 30 s (debounce) |
| Preloader | `preloader` | Decodifica archivos cortos en segundo plano para la caché RAM |
| Clima | `weather` | Refresca datos open-meteo cada 15 min; emite `"weather-updated"` |

---

## 8. Eventos Rust → Frontend

| Evento | Payload | Quién escucha |
|---|---|---|
| `"audio-tick"` | `AudioTickPayload {buttons[{group, progress_percent, ...}], display_remaining, display_duration, master_level_l, master_level_r, idle}` | gridPlayback.js, fixedPanel.js, clockWidget.js, vuMeter.js, tabs.js; también dispara `CustomEvent("lf-audio-tick")` en el DOM |
| `"player-tick"` | `PlayerSnapshot {playing, path, position_s, duration_s, current_index, next_index, mode, stop_after, loop_current, can_seek, volume, queue_len}` | runtimeEvents.js → playerView.js (verde = `current_index`, naranja = `next_index`) |
| `"player-drop-progress"` | progreso al añadir una carpeta grande a la cola (lotes de 20) | playerDrop.js |
| `"clock-tick"` | `{time_str, date_str}` | clockWidget.js |
| `"weather-updated"` | datos de clima | settingsLocutions.js |
| `"global-shortcut-refresh"` | (vacío) | startup.js → `_refresh()` |
| `"track-editor-dock"` | `{path, name, zoom}` | startup.js → abre editor en modo modal |
| `"track-analysis-progress"` | `{path, stage}` | trackEditor.js → actualiza progreso del análisis |
| `"theme-changed"` | `{theme}` | ventana pop-out del editor |

---

## 9. Comandos IPC (registro completo)

### Config y perfiles
- `get_config` → `AppConfig`
- `set_first_boot_complete`
- `set_theme(theme)`
- `set_language(language)`
- `set_button_text_size(size)`
- `set_active_profile(profile_id)`
- `create_profile(name)`
- `delete_profile(profile_id)`
- `update_profile_meta(id, name, bg, text)`

### Paletas (pestañas)
- `set_active_paleta(paleta_id)`
- `create_paleta(nombre, rows, cols)`
- `delete_paleta(paleta_id)`
- `update_paleta_meta(paleta_id, ...)`
- `reorder_paletas(from_id, to_id)`

### Audio
- `get_audio_devices` → `Vec<String>`
- `get_audio_device_status`
- `apply_configured_audio_devices`
- `set_audio_device(device_name)`
- `set_pre_device(device_name)` — vacío = fallback a principal
- `play_audio(id, path, volume, duration?, loop_mode?, stop_other?, overlap?, restart?, cue_start_s?, cue_end_s?, gain_db?)`
- `stop_audio(id)`
- `stop_all_audio`
- `play_button(id)` — disparo principal desde la UI
- `set_audio_volume(id, volume)`
- `get_master_volume_state` → `{volume, remember, boost, max}`
- `set_master_volume(volume)`
- `set_master_volume_options(remember, boost)`

### Grid / botones
- `get_grid_state` → grid de la paleta activa
- `suggest_button_style(path)` → colores sugeridos basados en el archivo
- `get_color_palette` → paleta de 24 colores (`SAFE_COLORS`, `domain/palette.rs`)
- `set_buttons_color(indexes, colorBg, group)` — pinta de una vez los seleccionados con Ctrl+clic. `group` = "grid" (paleta activa) o "fixed" (panel). El color del TEXTO no se pide: lo calcula Rust (regla 8)
- `assign_file_to_button(paleta_id, index, path)`
- `clear_button(paleta_id, index)`
- `undo_config`
- `redo_config`
- `toggle_button_flag(paleta_id, index, flag)` — flag ∈ {loop_mode, stop_other, overlap, restart}
- `get_edit_button_types`
- `update_button_data(paleta_id, index, data)`
- `move_button_to_paleta(from_paleta_id, from_index, to_paleta_id, to_index)`
- `reorder_buttons(paleta_id, from_id, to_id)`

### Atajos de teclado
- `set_global_keys(stop?, next?, prev?)`
- `cycle_paleta(direction)` — "next" | "prev"
- `handle_local_shortcut(combo)` → dispara el botón o acción asignada
- `clear_button_shortcut(paleta_id, index)`

### Locuciones (Fase 6)
- `set_locution_config(config)`
- `pick_named_folder(name)` → abre diálogo de carpeta
- `search_city(query)` → geocoding
- `preview_weather`
- `get_weather_now`
- `play_time_locution(id?, vol?, folder?)`
- `play_climate_locution(id?, climate_type, vol?, folder?)`

### Export / Import
- `export_tab(paleta_id, path?)` — abre diálogo si no se pasa path
- `export_tab_by_id(paleta_id)` → JSON string
- `import_tab(path?)` — abre diálogo si no se pasa path
- `export_profile(profile_id, path?)`
- `export_profile_by_id(profile_id)` → JSON string
- `import_profile(path?)`

### Metadatos / Sistema
- `get_app_version` → String
- `toggle_clock_format`
- `check_for_updates` → `{checked, updateAvailable, currentVersion, latestVersion, releaseUrl, notes}`

### Modo de reproducción global
- `get_playback_mode` → String
- `get_playback_state` → `{mode, solo}`
- `set_playback_mode(mode)` — "normal" | "loop" | "overlap" | "restart"
- `set_solo_mode(enabled)`

### Reproductor auxiliar (modo reproductor del panel fijo)
Motor **independiente** en lo que importa: su hilo, su cola, su avance y su transporte. El Stop
general y el Solo de los efectos **no** lo detienen. Lo que ya **no** tiene es tarjeta propia:
entrega sus fuentes al bus `Reproductor` de la consola, que suma en el programa — por eso
**obedece al máster** (en volumen, no en transporte) y aparece en el vúmetro.

**Su volumen es el fader del bus `Reproductor`.** Bajar la música para hablar encima es mover ese
fader, y no toca los efectos. El máster es otra cosa: baja los tres buses a la vez.

Su `output_device` vacío = `Routing::Program` (suma en el programa); un nombre =
`Routing::Device(x)`, sale por esa tarjeta y **deja de obedecer al máster**. Ojo al traducir "" a
un nombre de tarjeta en el camino: lo sacaría del programa sin querer.

Los índices son POSICIONES 0-based en la cola. Ver `Documentación/PLAN_MODO_REPRODUCTOR.md`.

- `get_player` → `PlayerView {tracks, mode, volume, output_device, total_s, snapshot}`
- `get_player_snapshot` → `PlayerSnapshot` (ligero)
- `player_play_index(index)` — reproduce esa posición
- `player_activate_index(index)` — doble clic: **el motor decide** (detenido reproduce; sonando marca siguiente)
- `player_next` / `player_prev` / `player_stop` / `player_pause` / `player_resume`
- `player_mark_next(index?)` — marcar siguiente (naranja). **Es ley**: se respeta en todos los modos
- `player_set_mode(mode)` — "normal" | "repeat" | "random". Dice QUÉ pista viene; que el reproductor se pare al acabar lo decide `player_set_stop_after`, que se combina con los tres (hubo un modo `manual` que duplicaba eso y además forzaba el orden normal: se quitó y se migra a `normal`)
- `player_set_stop_after(enabled)` — al acabar la actual no arranca sola
- `player_set_loop(enabled)` — **Loop**: repite la canción ACTUAL (≠ modo `repeat`, que repite la lista). Manda sobre stop-after; cede ante Siguiente; no toca lo marcado
- `player_seek(positionS)` — salta dentro de la pista; el motor lo ignora si `can_seek` es falso (una locución son varios archivos encadenados)
- `player_toggle_time_display` → "elapsed" | "remaining"; se persiste
- `player_set_volume(volume, persist?)` — 0.0–1.5 (la UI expone 0–100 %). `persist: false` mientras se arrastra: aplicar es un atómico, guardar en cada píxel sería una tormenta de escrituras
- `player_set_device(device)` — "" = el mismo de los efectos. **Reaplicarlo reabre la salida y corta la música**
- `player_add_track(path, index?)` / `player_add_button(buttonId, index?)`
- `player_remove_track(index)` / `player_reorder_tracks(fromIndex, toIndex)` / `player_clear_queue`
- `player_save_playlist` / `player_open_playlist` — formato `.LFPlay` (compatible con LFA)
- `player_scan_drop(paths)` → `DropScan` — cuenta lo soltado (carpetas incluidas, recursivo) y **Rust decide** si hay que preguntar (umbral `LARGE_FOLDER_THRESHOLD` = 250)
- `player_add_drop(paths)` — añade en `spawn_blocking` por lotes de 20 emitiendo `player-drop-progress`; una sola escritura a disco al final
- `player_set_large_folder_action(action)` — `ask` | `always` | `never`: qué hacer ante una carpeta grande. Se puede rectificar en Ajustes → Panel fijo si se marcó "recordar siempre" sin querer

### Editor de pistas
- `analyze_track(path)` → análisis del editor con `track-analysis-progress`, caché en memoria/disco y `AnalysisResult {waveform, peak_db, lufs, suggested_gain_db, duration_s, cue_start_s, cue_end_s, gain_db, norm_enabled, norm_gain_db}`
- `waveform_view(path, start_s, end_s, buckets)` → `Vec<[f32;2]>` (min/max por columna)
- `get_track_meta(path)` → `TrackMeta`
- `set_track_cue(path, cue_start_s, cue_end_s?)`
- `set_track_gain(path, gain_db)`
- `set_track_normalization(path, enabled)`
- `set_editor_mode(mode)` — "modal" | "window"; persiste en AppConfig

### Precarga
- `get_preload_config` → `PreloadView {enabled, ram_budget_mb, max_duration_s, strategy, ttl{value, unit}}`
- `should_prompt_preload` → bool
- `mark_preload_prompted`
- `set_preload_config(config)`
- `get_preload_stats` → `PreloadStats {used_mb, count, budget_mb, enabled}`

---

## 10. Mapa de módulos Rust (`src-tauri/src/`)

A partir de la v1.1.3, el backend sigue una arquitectura de "Núcleo + Motores" en 5 capas:

| Capa | Responsabilidad |
|---|---|
| `core/` | `AppState`, setup inicial, configuración global y mapeo de errores |
| `model/` | Tipos puros (AppConfig, TrackMeta, etc.) y serialización serde. Sin lógica de I/O. |
| `engine/` | Motores autónomos: `console/` (salidas físicas y buses), `audio/` (efectos), `player/` (reproductor auxiliar), `dsp/` (ebur128, symphonia, waveform), `cache/` (LRU), `persist/` (SQLite, JSON), `weather/` (open-meteo), `input/` (atajos). |
| `domain/` | Reglas de negocio puras (relativas a grids, botones, y modos de reproducción). |
| `ipc/` | Endpoints Tauri. Funciones finas que extraen parámetros y delegan en el dominio/motores. |
| `domain/export/lfa_format/` | Adaptadores de compatibilidad bidireccional con LF Automatizador (.bdelf/.LFPlay). |

**`engine/console/` (la consola de audio), por responsabilidad:**
`mod.rs` (handle `ConsoleEngine` + `BusSlot` + `ConsoleState`), `thread.rs` (hilo guardián: único
dueño de las tarjetas), `graph.rs` (monta el grafo de buses según el ruteo), `endpoint.rs`
(`OutputEndpoint`: una tarjeta abierta **una vez** + registro por nombre), `bus.rs` (`Bus`: mixer
+ fader + medidor; **sin `Sink`**), `fader.rs` (`FaderSource`), `level.rs` (`LevelSource`),
`device.rs` (`find_device` / `available_devices`).

**`domain/console/`** — las reglas: `BusId`, `Routing`, `sanitize`, `device_of`,
`devices_in_use`. Puras y probadas sin tarjeta de sonido. Aquí se decide qué va dónde; el motor
solo obedece.

Es un motor **debajo** de `audio/` y `player/`, no al lado: no produce audio, lo encamina, y los
dos son sus clientes. `MasterBus` y `AudioDeviceRuntime` desaparecieron absorbidos por él.
Fases y decisiones: `Documentación/PLAN_CONSOLA_VIRTUAL.md`.

**`engine/player/` (reproductor auxiliar), por responsabilidad:**
`mod.rs` (handle `PlayerEngine`), `thread.rs` (hilo; dueño de los 2 decks),
`source.rs` (`DeckSource` + `DeckHandle`: la fuente en el bus y su mando — sustituye al `Sink`
sosteniendo a mano posición, fin de pista y pausa),
`deck.rs` (estados de un deck), `queue.rs` (datos de la cola), `queue_ops.rs` (transporte),
`queue_select.rs` (elegir siguiente + pre-carga ping-pong), `resolve.rs` (resuelve los tipos
especiales **al sonar**), `exec.rs` (traduce acciones a rodio), `monitor.rs` (emite `"player-tick"`).
La regla pura de avance vive en `domain/player/advance.rs`; el cue/ganancia en `domain/playback/edit.rs`.

## 11. Mapa de módulos Frontend (`src/`)

A partir de la v1.1.3, el frontend sigue una arquitectura de 3 capas en `src/js/`:

| Capa | Responsabilidad |
|---|---|
| `bridge/` | Capa IPC (`api.js`). Aísla `window.__TAURI__` del resto del código. |
| `ui/` | Componentes de la interfaz de usuario: modals, editor de pistas, rejilla, botones. |
| `util/` | Helpers y utilidades: i18n, adaptadores de color, formato de tiempo. |

## 12. Formatos de exportación `.bdelf` / `.bdeplf`

El LFA usa nombres de campo distintos (`file`, `bg`, `text`, `loop`, `stopOther`). La conversión vive en `domain/export/lfa_format/`.

`.bdelf` = exportación de una paleta:
```json
{
  "id": "paleta_1",
  "nombre": "BOTONERA 1",
  "rows": 5,
  "cols": 5,
  "botones": [ /* array con TODAS las celdas, vacías incluidas, indexadas 1-based */ ],
  "bdelf_tracks": {
    "C:/ruta/audio.mp3": { "cue_start_s": 1.5, "gain_db": -3.0, ... }
  }
}
```

`bdelf_tracks` es **OPCIONAL** (solo lo añade la Botonera). El LFA lo ignora. Al importar, `domain/export/tracks.rs::restore()` escribe los datos en `tracks.db` re-sellando con `mtime`/`size` del archivo local.

---

## 13. Secuencia de arranque

1. `main.rs` → `lib::run()`
2. `lib::run()` inicializa `AppState` (carga config, abre tracks.db, crea AudioEngine)
3. Tauri llama `core::setup::on_setup()`:
   - Aplica dispositivo de audio (out_main del perfil activo)
   - Aplica dispositivo de pre-escucha (out_pre, si difiere del principal)
   - Fija presupuesto de RAM de la caché
   - Arranca 4 hilos: monitor, reloj, historial, clima
   - Precarga caliente según estrategia
   - Registra `on_window_event` CloseRequested → flush de historial
4. Frontend (`main.js`) espera `DOMContentLoaded` → `startup::startApp()`
5. `startApp()` espera `window.__TAURI__` → invoca `get_config` → aplica tema/idioma → inicia módulos → suscribe eventos Rust

**Detección de pop-out:** si la URL contiene `?editor=<path>`, `startup.js` arranca en modo editor exclusivo (sin rejilla, sin barra inferior), carga solo el módulo `trackEditor.js`.

---

## 14. Cómo verificar sin tocar la pantalla

```bash
# Backend Rust (suite actual: 168 passed, 1 ignored)
cd C:\OVERLAY\BOTONERA\src-tauri
cargo test --lib

# Consola contra las TARJETAS REALES del equipo (no entran en la suite normal:
# necesitan hardware, y una prueba que falla por el entorno no dice nada).
# Comprueban lo que los mixers de mentira no pueden: que el grafo se monta sobre
# tarjetas de verdad, que el medidor mide lo que sale, y que cambiar de salida no
# deja el vúmetro mudo ni pisado por buses fantasma.
# CIERRA LA APLICACIÓN antes: abren las mismas tarjetas.
cargo test --test consola_tarjetas_reales -- --ignored --nocapture --test-threads=1

# OJO: `cargo test` compila CON los tests, y un `use super::*` de un fichero de
# pruebas puede tapar un import que ya nadie usa en el módulo. El usuario compila
# SIN tests (`tauri dev`) y ahí sí sale el warning. Comprobar siempre las dos:
cargo build --lib

# Frontend + build completo
cd C:\OVERLAY\BOTONERA
npm run build

# Límite de líneas (POSIX)
wc -l src-tauri/src/<archivo>.rs
```

No lanzar la app por computer-use. El usuario prueba en su PC.

**Al publicar una nueva versión:**
1. Actualizar `CHANGELOG.md`: añadir las entradas acumuladas en `[Sin publicar]`, renombrar esa sección a `[X.Y.Z] — YYYY-MM-DD`, crear una nueva sección `[Sin publicar]` vacía encima, y añadir el enlace comparativo al pie del archivo.
2. Ejecutar `SET-VERSION.bat X.Y.Z` — sincroniza la versión en `package.json`, `Cargo.toml` y `tauri.conf.json`.
3. Regenerar `Cargo.lock`: `cd src-tauri && cargo check`.
4. Commit, tag y push: `git commit -am "Release X.Y.Z"` → `git tag vX.Y.Z` → `git push && git push --tags`.

---

## 15. Pendientes reales (en orden de prioridad)

### A) Prueba física en Linux
- El código es agnóstico del SO (rutas vía `config::get_data_dir()`, SQLite bundled, rodio/ALSA).
- Falta compilar y probar en una máquina Linux real (`.deb`, `.AppImage`).

### B) Deuda menor: `master_volume` es `f32`
- Su representación en JSON crece sola al guardar (`0.45` → `0.4499999…`). Inocuo, pero ensucia el
  fichero. Afecta a `AudioConfig` y al `vol` de `ButtonData`.

> **Política de colores de los botones nuevos: DESCARTADA** (2026-07-16). El autor la vio
> complicada de explicar y de usar. En su lugar existe la **selección múltiple** (Ctrl+clic y clic
> derecho → pintar: `buttonSelection.js` + `set_buttons_color`). **No volver a proponerla.**
> `Documentación/PLAN_POLITICA_COLORES.md` se conserva solo como registro de lo que se decidió.

---

## 16. Trampas conocidas y notas de entorno

- **`window.__TAURI__`** no está disponible al parsear módulos en producción (WebView2). Capturarlo al nivel del módulo lo congela como `undefined`. `api.js` lo resuelve dentro de cada función.
- **`lf-audio-tick`** es un `CustomEvent` del DOM (dispara `startup.js`), NO un evento de Tauri. Escuchar con `window.addEventListener('lf-audio-tick', ...)`, nunca con `api.listen()`.
- **`buttons` vacío NO significa silencio.** El bus `Programa` suma efectos, panel **y reproductor**, y la música no aparece en esa lista. Para saber si hay silencio está el campo `idle` del tick, que lo decide Rust. Deducirlo de `buttons.length` fue un bug real: el vúmetro daba cada tick por final, le aplicaba el decaimiento de 0.8 s y la aguja nunca alcanzaba el nivel con música de fondo.
- **IDs de botón** cambiaron de `btn_{index}` a `{paleta_id}_btn_{index}` para evitar colisiones entre pestañas. `config_io.rs::normalize_button_ids()` migra automáticamente al cargar.
- **Instalador MSI** tiene `upgradeCode` fijo (`43888972-…`). No cambiar entre versiones.
- **`capabilities/default.json`** NO debe tener BOM: el parser de Tauri lo rechaza silenciosamente.
- **Instancias `tauri dev`** acumuladas: matar TODAS las instancias de `tauri-app` y relanzar `npm run tauri dev`; matar solo una puede dejar Vite caído.
- **Sincronización de versión:** al publicar, actualizar la misma versión en `package.json`, `src-tauri/Cargo.toml` y `src-tauri/tauri.conf.json`.
