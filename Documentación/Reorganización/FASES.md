# Fases de la Reorganización

## Resumen

| Fase | Nombre | Archivos afectados | Tipo de trabajo | Sesión estimada |
|------|--------|--------------------|-----------------|-----------------|
| 1 | Modelo de datos | ~10 archivos | Mover | Sesión 1 |
| 2 | Motores | 35 archivos | Mover | Sesión 1 |
| 3 | Dominio | 18 archivos | Mover | Sesión 2 |
| 4 | Puerta IPC | ~22 archivos | Mover + split | Sesión 2 |
| 5 | Núcleo | ~3 archivos | Mover + crear | Sesión 3 |
| 6 | Helpers y deduplicación | ~20 archivos tocados | Crear + refactorizar | Sesión 3 |
| 7 | Frontend | ~54 archivos | Mover | Sesión 4 (separada) |
| 8 | Verificación final | 0 archivos | Pruebas | Sesión 5 |

---

## Fase 1 — Modelo de datos (`model/`)

**Objetivo:** Extraer todos los tipos de datos puros a `src-tauri/src/model/`.

**Archivos a mover:**
- `types.rs` → `model/config.rs`
- `types_audio.rs` → `model/audio.rs`
- `types_track.rs` → `model/track.rs`
- `types_preload.rs` → `model/preload.rs`
- `types_fade.rs` → `model/fade.rs`
- `types_grid.rs` → `model/grid.rs`
- `types_locutions.rs` → `model/locutions.rs`
- `types_norm.rs` → `model/norm.rs`
- `types_playback_progress.rs` → `model/playback.rs`
- `types_startup.rs` → `model/startup.rs`

**Archivos a crear:**
- `model/mod.rs` — declara todos los sub-módulos y re-exporta tipos públicos

**Verificación:**
```bash
cd src-tauri && cargo test --lib && cd .. && npm run build
```

**Commit:** `refactor: move data types to model/ directory`

---

## Fase 2 — Motores (`engine/`)

**Objetivo:** Agrupar cada subsistema en su propia subcarpeta dentro de `engine/`.

### 2a. Motor de Audio (`engine/audio/`)

**Archivos a mover (13):**
- `audio.rs` → `engine/audio/engine.rs`
- `audio_command.rs` → `engine/audio/command.rs`
- `audio_thread.rs` → `engine/audio/thread.rs`
- `audio_thread_play.rs` → `engine/audio/thread_play.rs`
- `audio_device.rs` → `engine/audio/device.rs`
- `audio_device_list.rs` → `engine/audio/device_list.rs`
- `audio_decode.rs` → `engine/audio/decode.rs`
- `audio_formats.rs` → `engine/audio/formats.rs`
- `audio_monitor.rs` → `engine/audio/monitor.rs`
- `audio_ops.rs` → `engine/audio/ops.rs`
- `master_bus.rs` → `engine/audio/bus.rs`
- `master_button.rs` → `engine/audio/button.rs`
- `vu_meter.rs` → `engine/audio/vu.rs`

### 2b. Motor DSP (`engine/dsp/`)

**Archivos a mover (5):**
- `audio_analysis.rs` → `engine/dsp/analysis.rs`
- `cue_source.rs` → `engine/dsp/cue_source.rs`
- `cue_detect.rs` → `engine/dsp/cue_detect.rs`
- `fade_ramp.rs` → `engine/dsp/fade.rs`
- `waveform.rs` → `engine/dsp/waveform.rs`

### 2c. Motor de Caché (`engine/cache/`)

**Archivos a mover (5):**
- `preload_cache.rs` → `engine/cache/preload.rs`
- `preloader.rs` → `engine/cache/preloader.rs`
- `preload_warm.rs` → `engine/cache/warm.rs`
- `cached_source.rs` → `engine/cache/cached_source.rs`
- `track_analysis_cache.rs` → `engine/cache/track_analysis.rs`

### 2d. Motor de Persistencia (`engine/persist/`)

**Archivos a mover (5):**
- `config.rs` → `engine/persist/config_io.rs`
- `config_history.rs` → `engine/persist/history.rs`
- `db.rs` → `engine/persist/db.rs`
- `track_store.rs` → `engine/persist/tracks.rs`
- `last_played.rs` → `engine/persist/last_played.rs`

### 2e. Motor de Clima (`engine/weather/`)

**Archivos a mover (4):**
- `weather.rs` → `engine/weather/client.rs`
- `geocode.rs` → `engine/weather/geocode.rs`
- `locutions.rs` → `engine/weather/resolver.rs`
- `locution_playback.rs` → `engine/weather/playback.rs`

### 2f. Motor de Entrada (`engine/input/`)

**Archivos a mover (3):**
- `global_shortcuts.rs` → `engine/input/keyboard.rs`
- `shortcut_rules.rs` → `engine/input/rules.rs`
- `tab_reorder.rs` → `engine/input/tab_reorder.rs`

**Archivos a crear:**
- `engine/mod.rs`
- `engine/audio/mod.rs`
- `engine/dsp/mod.rs`
- `engine/cache/mod.rs`
- `engine/persist/mod.rs`
- `engine/weather/mod.rs`
- `engine/input/mod.rs`

**Verificación:**
```bash
cd src-tauri && cargo test --lib && cd .. && npm run build
```

**Commit:** `refactor: move subsystems to engine/ directory`

---

## Fase 3 — Dominio (`domain/`)

**Objetivo:** Agrupar la lógica de negocio pura en `domain/`.

### 3a. Reproducción (`domain/playback/`)

- `playback_mode.rs` → `domain/playback/mode.rs`
- `playback_state.rs` → `domain/playback/state.rs`
- `playback_source.rs` → `domain/playback/source.rs`
- `playback_seek.rs` → `domain/playback/seek.rs`

### 3b. Botones (`domain/button/`)

- `button_defaults.rs` → `domain/button/defaults.rs`
- `button_types.rs` → `domain/button/types.rs`
- `random_folder.rs` → `domain/button/random_folder.rs`

### 3c. Rejilla (`domain/grid/`)

- `grid_view.rs` → `domain/grid/view.rs`
- `grid_resize.rs` → `domain/grid/resize.rs`
- `grid_reorder.rs` → `domain/grid/reorder.rs`
- `grid_move.rs` → `domain/grid/move_btn.rs`

### 3d. Exportación (`domain/export/`)

- `export_tracks.rs` → `domain/export/tracks.rs`
- `lfa_format.rs` → `domain/export/lfa_format.rs`
- `lfa_format/types.rs` → `domain/export/lfa_format/types.rs`
- `lfa_format/paleta.rs` → `domain/export/lfa_format/paleta.rs`
- `lfa_format/profile.rs` → `domain/export/lfa_format/profile.rs`

### 3e. Otros

- `colors.rs` → `domain/colors.rs`
- `cmd_track_response.rs` → `domain/track_response.rs`

**Archivos a crear:**
- `domain/mod.rs`
- `domain/playback/mod.rs`
- `domain/button/mod.rs`
- `domain/grid/mod.rs`
- `domain/export/mod.rs`

**Verificación:**
```bash
cd src-tauri && cargo test --lib && cd .. && npm run build
```

**Commit:** `refactor: move business logic to domain/ directory`

---

## Fase 4 — Puerta IPC (`ipc/`)

**Objetivo:** Mover todos los comandos IPC a `ipc/` y hacer los splits necesarios.

**Archivos a mover (20):**
- Todos los `cmd_*.rs` actuales → `ipc/cmd_*.rs`
- `register_handlers.rs` → `ipc/register.rs`

**Splits a realizar:**
- Extraer de `cmd_profiles.rs`:
  - `set_theme`, `set_language`, `set_button_text_size`, `set_first_boot_complete`,
    `set_editor_mode` → **nuevo** `ipc/cmd_config.rs`
  - `set_norm_config`, `set_cue_detect_config`, `mark_norm_prompted`,
    `set_fade_config` → **nuevo** `ipc/cmd_norm.rs`
  - Lo que queda en `cmd_profiles.rs`: solo CRUD de perfiles

**Archivos a crear:**
- `ipc/mod.rs`
- `ipc/cmd_config.rs`
- `ipc/cmd_norm.rs`

**Verificación:**
```bash
cd src-tauri && cargo test --lib && cd .. && npm run build
```

**Commit:** `refactor: move IPC commands to ipc/ directory`

---

## Fase 5 — Núcleo (`core/`)

**Objetivo:** Crear el núcleo central de la aplicación.

**Archivos a mover:**
- Extraer `AppState` de `lib.rs` → `core/state.rs`
- `app_setup.rs` → `core/setup.rs`

**Archivos a crear:**
- `core/mod.rs`
- `core/errors.rs` — tipo `AppError` unificado con `thiserror`

**Actualizar `lib.rs`:**
- Reducir a: declaraciones `mod` de primer nivel + función `run()` que llama a
  `core::setup` y registra los handlers de `ipc/`

**Verificación:**
```bash
cd src-tauri && cargo test --lib && cd .. && npm run build
```

**Commit:** `refactor: create core/ with AppState and unified errors`

---

## Fase 6 — Helpers y deduplicación

**Objetivo:** Crear los archivos nuevos y eliminar duplicaciones.

### 6a. Config helpers
- **Crear** `engine/persist/config_helpers.rs`:
  - `AppConfig::active_profile()` y `active_profile_mut()`
  - `AppConfig::active_paleta()` y `active_paleta_mut()`
  - `AppConfig::active_audio()`
- **Actualizar** ~15 archivos que repiten el patrón de búsqueda

### 6b. Acciones centralizadas de entrada
- **Crear** `engine/input/actions.rs`:
  - `cycle_paleta(cfg, offset, preload)` — una sola implementación
  - `activate_paleta(cfg, paleta_id, preload)` — una sola implementación
  - `play_by_shortcut(cfg, key, audio)` — dispatch centralizado
- **Actualizar** `ipc/cmd_keys.rs`, `engine/input/keyboard.rs`,
  `ipc/cmd_local_shortcuts.rs` para usar las funciones centralizadas

### 6c. Extracción del reloj
- **Crear** `domain/clock.rs`:
  - `start_clock_thread()` y datos estáticos de días/meses
- **Actualizar** `ipc/cmd_meta.rs` para delegar al nuevo módulo

### 6d. Reubicación de probe_duration_secs
- **Mover** `probe_duration_secs` de `ipc/cmd_audio.rs` a `engine/audio/formats.rs`
- **Actualizar** todos los archivos que la llaman

### 6e. Fix track_store.rs (201 → ≤200 líneas)
- Extraer un método helper o eliminar una línea en blanco

**Verificación:**
```bash
cd src-tauri && cargo test --lib && cd .. && npm run build
```

**Commit:** `refactor: add helpers, centralize actions, deduplicate`

---

## Fase 7 — Frontend (fase separada)

**Objetivo:** Reorganizar `src/js/` en `bridge/`, `ui/`, `util/`.

Esta fase es independiente del backend y solo afecta:
- Las rutas de `import` entre módulos JS
- El `index.html` (ruta del script de entrada si cambia)
- La configuración de Vite (si necesita alias)

**Detalle completo en el plan de implementación principal.**

**Verificación:**
```bash
npm run build
```

**Commit:** `refactor: reorganize frontend into bridge/ui/util directories`

---

## Fase 8 — Verificación final

**Objetivo:** Confirmar que todo funciona.

**Checklist:**
- [ ] `cargo test --lib` — todos los tests pasan
- [ ] `npm run build` — frontend compila
- [ ] `wc -l` — ningún archivo supera 200 líneas
- [ ] App arranca correctamente
- [ ] Botones reproducen audio
- [ ] Pre-escucha funciona
- [ ] Editor de pistas abre y muestra onda
- [ ] Cue markers se arrastran
- [ ] Atajos locales y globales responden
- [ ] Export/import .bdelf funciona
- [ ] Tema claro/oscuro sin parpadeo
- [ ] Reloj y VU meter actualizan
- [ ] Perfiles y pestañas CRUD funcional

**Commit final:** `refactor: merge refactor/architecture into main`
