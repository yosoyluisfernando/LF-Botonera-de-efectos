# CLAUDE.md — LF Botonera de Efectos

Guía de contexto completa para una IA que colabora en este proyecto.
Lee todo antes de tocar código.

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
10. **No usar computer-use.** Controlar la pantalla del usuario cuesta tokens. Verificar solo con `cargo test --lib` y `npm run build`; el usuario prueba en su PC.
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

**Notas de IDs de botón:** formato `{paleta_id}_btn_{index}`. `config.rs` normaliza al cargar para migrar el formato antiguo `btn_{index}` que colisionaba entre paletas.

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
                           audio_thread (hilo dedicado)
                                  │
                          AudioCommand::Play{to_pre}
                                  │
                    ┌─────────────┴────────────────┐
                    │ to_pre=false                  │ to_pre=true
                    ▼                               ▼
              device.bus()               device_pre.bus() ──fallback──► device.bus()
                    │
                    ▼
            preload_cache::build_play_source(path, loop, cue)
                    ├── Cache HIT → CachedSource::new_at(pcm, offset) — O(1) seek
                    └── Cache MISS → audio_decode::source_from_path() + CuedSource (O(n))
                                          │
                                          ▼
                               MasterBus::add_source(source, volume, duration, loop_mode, file_gain)
                                          │
                                          ▼
                               ButtonSource { inner, stop_flag, done_flag, file_gain, volume, master_volume }
                               (implementa Iterator<Item=f32>; aplica: s × file_gain × vol_btn × master)
                                          │
                                          ▼
                               DynamicMixer<f32>
                                          │
                                          ▼
                               LevelSource (mide PICO del PCM sumado → atomic)
                                          │
                                          ▼
                               Sink → OutputStreamHandle → dispositivo CPAL
```

**Modelo de ganancia (3 capas):**
```
señal_salida = muestra × file_gain(dB→lineal) × vol_botón(lineal) × master(lineal)
```
- `file_gain`: de `TrackMeta.effective_gain_linear()` = norm_gain_db + gain_db → lineal
- `vol_botón`: `ButtonData.vol` (lineal 0–1; se preserva para compat `.bdelf`)
- `master`: `AudioConfig.master_volume` (0–1.5 en modo boost)

**Pre-escucha:**
- ID especial `__prelisten__` para la barra de pre-escucha
- ID especial `__track_preview__` para la previa dentro del editor de pistas
- Ambos van con `to_pre=true` → enrutados a `device_pre` si existe; si no, a `device`

---

## 7. Hilos en segundo plano

| Hilo | Módulo | Descripción |
|---|---|---|
| Audio | `audio_thread` | Ejecuta el motor; único hilo que toca rodio/cpal |
| Monitor | `audio_monitor` | Emite `"audio-tick"` cada 100 ms con progreso + VU |
| Reloj | `cmd_meta` | Emite `"clock-tick"` cada 1 s con hora y fecha localizadas |
| Historial | `last_played` | Vuelca buffer en memoria a tracks.db cada 30 s (debounce) |
| Preloader | `preloader` | Decodifica archivos cortos en segundo plano para la caché RAM |
| Clima | `weather` | Refresca datos open-meteo cada 15 min; emite `"weather-updated"` |

---

## 8. Eventos Rust → Frontend

| Evento | Payload | Quién escucha |
|---|---|---|
| `"audio-tick"` | `AudioTickPayload {buttons, display_remaining, display_duration, master_level_l, master_level_r}` | gridPlayback.js, clockWidget.js, vuMeter.js, tabs.js; también dispara `CustomEvent("lf-audio-tick")` en el DOM |
| `"clock-tick"` | `{time_str, date_str}` | clockWidget.js |
| `"weather-updated"` | datos de clima | settingsLocutions.js |
| `"global-shortcut-refresh"` | (vacío) | startup.js → `_refresh()` |
| `"track-editor-dock"` | `{path, name, zoom}` | startup.js → abre editor en modo modal |
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
- `get_color_palette` → paleta de 32 colores
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

### Editor de pistas
- `analyze_track(path)` → `AnalysisResult {waveform, peak_db, lufs, suggested_gain_db, duration_s, cue_start_s, cue_end_s, gain_db, norm_enabled, norm_gain_db}`
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
| `engine/` | Motores autónomos: `audio/` (rodio, hilos), `dsp/` (ebur128, symphonia, waveform), `cache/` (LRU), `persist/` (SQLite, JSON), `weather/` (open-meteo), `input/` (atajos). |
| `domain/` | Reglas de negocio puras (relativas a grids, botones, y modos de reproducción). |
| `ipc/` | Endpoints Tauri. Funciones finas que extraen parámetros y delegan en el dominio/motores. |
| `lfa_format/` | Adaptadores de compatibilidad bidireccional con LF Automatizador (.bdelf). |

## 11. Mapa de módulos Frontend (`src/`)

A partir de la v1.1.3, el frontend sigue una arquitectura de 3 capas en `src/js/`:

| Capa | Responsabilidad |
|---|---|
| `bridge/` | Capa IPC (`api.js`). Aísla `window.__TAURI__` del resto del código. |
| `ui/` | Componentes de la interfaz de usuario: modals, editor de pistas, rejilla, botones. |
| `util/` | Helpers y utilidades: i18n, adaptadores de color, formato de tiempo. |

## 12. Formatos de exportación `.bdelf` / `.bdeplf`

El LFA usa nombres de campo distintos (`file`, `bg`, `text`, `loop`, `stopOther`). La conversión vive en `lfa_format/`.

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

`bdelf_tracks` es **OPCIONAL** (solo lo añade la Botonera). El LFA lo ignora. Al importar, `export_tracks.rs::restore()` escribe los datos en `tracks.db` re-sellando con `mtime`/`size` del archivo local.

---

## 13. Secuencia de arranque

1. `main.rs` → `lib::run()`
2. `lib::run()` inicializa `AppState` (carga config, abre tracks.db, crea AudioEngine)
3. Tauri llama `app_setup::on_setup()`:
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
# Backend Rust (39 tests)
cd C:\OVERLAY\BOTONERA\src-tauri
cargo test --lib

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

---

## 16. Trampas conocidas y notas de entorno

- **`window.__TAURI__`** no está disponible al parsear módulos en producción (WebView2). Capturarlo al nivel del módulo lo congela como `undefined`. `api.js` lo resuelve dentro de cada función.
- **`lf-audio-tick`** es un `CustomEvent` del DOM (dispara `startup.js`), NO un evento de Tauri. Escuchar con `window.addEventListener('lf-audio-tick', ...)`, nunca con `api.listen()`.
- **IDs de botón** cambiaron de `btn_{index}` a `{paleta_id}_btn_{index}` para evitar colisiones entre pestañas. `config.rs::normalize_button_ids()` migra automáticamente al cargar.
- **Instalador MSI** tiene `upgradeCode` fijo (`43888972-…`). No cambiar entre versiones.
- **`capabilities/default.json`** NO debe tener BOM: el parser de Tauri lo rechaza silenciosamente.
- **Instancias `tauri dev`** acumuladas: matar TODAS las instancias de `tauri-app` y relanzar `npm run tauri dev`; matar solo una puede dejar Vite caído.
- **Sincronización de versión:** al publicar, actualizar la misma versión en `package.json`, `src-tauri/Cargo.toml` y `src-tauri/tauri.conf.json`.
