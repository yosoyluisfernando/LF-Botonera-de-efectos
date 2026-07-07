# Mapa de Movimientos

Tabla completa de cada archivo `.rs` y su destino final.

**Leyenda:**
- 🚚 = Mover (solo cambia ubicación + imports)
- ✨ = Crear nuevo
- ✂️ = Split (se extrae parte del archivo a uno nuevo)
- 🔧 = Modificar (cambio lógico menor)
- ➖ = Se queda donde está

**Decisión aplicada:** Los archivos se renombran al moverlos para eliminar
redundancia de prefijos (ej: `audio_thread.rs` → `thread.rs` dentro de `audio/`).

---

## Fase 1 — model/

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 1 | `types.rs` | `model/config.rs` | 🚚 Mover |
| 2 | `types_audio.rs` | `model/audio.rs` | 🚚 Mover |
| 3 | `types_track.rs` | `model/track.rs` | 🚚 Mover |
| 4 | `types_preload.rs` | `model/preload.rs` | 🚚 Mover |
| 5 | `types_fade.rs` | `model/fade.rs` | 🚚 Mover |
| 6 | `types_grid.rs` | `model/grid.rs` | 🚚 Mover |
| 7 | `types_locutions.rs` | `model/locutions.rs` | 🚚 Mover |
| 8 | `types_norm.rs` | `model/norm.rs` | 🚚 Mover |
| 9 | `types_playback_progress.rs` | `model/playback.rs` | 🚚 Mover |
| 10 | `types_startup.rs` | `model/startup.rs` | 🚚 Mover |
| 11 | — | `model/mod.rs` | ✨ Crear |

---

## Fase 2 — engine/

### engine/audio/

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 12 | `audio.rs` | `engine/audio/engine.rs` | 🚚 Mover |
| 13 | `audio_command.rs` | `engine/audio/command.rs` | 🚚 Mover |
| 14 | `audio_thread.rs` | `engine/audio/thread.rs` | 🚚 Mover |
| 15 | `audio_thread_play.rs` | `engine/audio/thread_play.rs` | 🚚 Mover |
| 16 | `audio_device.rs` | `engine/audio/device.rs` | 🚚 Mover |
| 17 | `audio_device_list.rs` | `engine/audio/device_list.rs` | 🚚 Mover |
| 18 | `audio_decode.rs` | `engine/audio/decode.rs` | 🚚 Mover |
| 19 | `audio_formats.rs` | `engine/audio/formats.rs` | 🚚 Mover |
| 20 | `audio_monitor.rs` | `engine/audio/monitor.rs` | 🚚 Mover |
| 21 | `audio_ops.rs` | `engine/audio/ops.rs` | 🚚 Mover |
| 22 | `master_bus.rs` | `engine/audio/bus.rs` | 🚚 Mover |
| 23 | `master_button.rs` | `engine/audio/button.rs` | 🚚 Mover |
| 24 | `vu_meter.rs` | `engine/audio/vu.rs` | 🚚 Mover |
| 25 | — | `engine/audio/mod.rs` | ✨ Crear |

### engine/dsp/

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 26 | `audio_analysis.rs` | `engine/dsp/analysis.rs` | 🚚 Mover |
| 27 | `cue_source.rs` | `engine/dsp/cue_source.rs` | 🚚 Mover |
| 28 | `cue_detect.rs` | `engine/dsp/cue_detect.rs` | 🚚 Mover |
| 29 | `fade_ramp.rs` | `engine/dsp/fade.rs` | 🚚 Mover |
| 30 | `waveform.rs` | `engine/dsp/waveform.rs` | 🚚 Mover |
| 31 | — | `engine/dsp/mod.rs` | ✨ Crear |

### engine/cache/

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 32 | `preload_cache.rs` | `engine/cache/preload.rs` | 🚚 Mover |
| 33 | `preloader.rs` | `engine/cache/preloader.rs` | 🚚 Mover |
| 34 | `preload_warm.rs` | `engine/cache/warm.rs` | 🚚 Mover |
| 35 | `cached_source.rs` | `engine/cache/cached_source.rs` | 🚚 Mover |
| 36 | `track_analysis_cache.rs` | `engine/cache/track_analysis.rs` | 🚚 Mover |
| 37 | — | `engine/cache/mod.rs` | ✨ Crear |

### engine/persist/

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 38 | `config.rs` | `engine/persist/config_io.rs` | 🚚 Mover |
| 39 | `config_history.rs` | `engine/persist/history.rs` | 🚚 Mover |
| 40 | `db.rs` | `engine/persist/db.rs` | 🚚 Mover |
| 41 | `track_store.rs` | `engine/persist/tracks.rs` | 🚚 Mover + 🔧 fix 201→200 líneas |
| 42 | `last_played.rs` | `engine/persist/last_played.rs` | 🚚 Mover |
| 43 | — | `engine/persist/mod.rs` | ✨ Crear |

### engine/weather/

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 44 | `weather.rs` | `engine/weather/client.rs` | 🚚 Mover |
| 45 | `geocode.rs` | `engine/weather/geocode.rs` | 🚚 Mover |
| 46 | `locutions.rs` | `engine/weather/resolver.rs` | 🚚 Mover |
| 47 | `locution_playback.rs` | `engine/weather/playback.rs` | 🚚 Mover |
| 48 | — | `engine/weather/mod.rs` | ✨ Crear |

### engine/input/

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 49 | `global_shortcuts.rs` | `engine/input/keyboard.rs` | 🚚 Mover |
| 50 | `shortcut_rules.rs` | `engine/input/rules.rs` | 🚚 Mover |
| 51 | `tab_reorder.rs` | `engine/input/tab_reorder.rs` | 🚚 Mover |
| 52 | — | `engine/input/mod.rs` | ✨ Crear |

| 53 | — | `engine/mod.rs` | ✨ Crear |

---

## Fase 3 — domain/

### domain/playback/

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 54 | `playback_mode.rs` | `domain/playback/mode.rs` | 🚚 Mover |
| 55 | `playback_state.rs` | `domain/playback/state.rs` | 🚚 Mover |
| 56 | `playback_source.rs` | `domain/playback/source.rs` | 🚚 Mover |
| 57 | `playback_seek.rs` | `domain/playback/seek.rs` | 🚚 Mover |
| 58 | — | `domain/playback/mod.rs` | ✨ Crear |

### domain/button/

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 59 | `button_defaults.rs` | `domain/button/defaults.rs` | 🚚 Mover |
| 60 | `button_types.rs` | `domain/button/types.rs` | 🚚 Mover |
| 61 | `random_folder.rs` | `domain/button/random_folder.rs` | 🚚 Mover |
| 62 | — | `domain/button/mod.rs` | ✨ Crear |

### domain/grid/

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 63 | `grid_view.rs` | `domain/grid/view.rs` | 🚚 Mover |
| 64 | `grid_resize.rs` | `domain/grid/resize.rs` | 🚚 Mover |
| 65 | `grid_reorder.rs` | `domain/grid/reorder.rs` | 🚚 Mover |
| 66 | `grid_move.rs` | `domain/grid/move_btn.rs` | 🚚 Mover |
| 67 | — | `domain/grid/mod.rs` | ✨ Crear |

### domain/export/

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 68 | `export_tracks.rs` | `domain/export/tracks.rs` | 🚚 Mover |
| 69 | `lfa_format.rs` | `domain/export/lfa_format.rs` | 🚚 Mover |
| 70 | `lfa_format/types.rs` | `domain/export/lfa_format/types.rs` | 🚚 Mover |
| 71 | `lfa_format/paleta.rs` | `domain/export/lfa_format/paleta.rs` | 🚚 Mover |
| 72 | `lfa_format/profile.rs` | `domain/export/lfa_format/profile.rs` | 🚚 Mover |
| 73 | — | `domain/export/mod.rs` | ✨ Crear |

### domain/ (raíz)

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 74 | `colors.rs` | `domain/colors.rs` | 🚚 Mover |
| 75 | `cmd_track_response.rs` | `domain/track_response.rs` | 🚚 Mover |
| 76 | — | `domain/mod.rs` | ✨ Crear |

---

## Fase 4 — ipc/

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 77 | `register_handlers.rs` | `ipc/register.rs` | 🚚 Mover |
| 78 | `cmd_profiles.rs` | `ipc/cmd_profiles.rs` | 🚚 + ✂️ Split (sacar config+norm) |
| 79 | `cmd_paletas.rs` | `ipc/cmd_paletas.rs` | 🚚 Mover |
| 80 | `cmd_grid.rs` | `ipc/cmd_grid.rs` | 🚚 Mover |
| 81 | `cmd_audio.rs` | `ipc/cmd_audio.rs` | 🚚 Mover |
| 82 | `cmd_button_playback.rs` | `ipc/cmd_button_playback.rs` | 🚚 Mover |
| 83 | `cmd_button_update.rs` | `ipc/cmd_button_update.rs` | 🚚 Mover |
| 84 | `cmd_button_flags.rs` | `ipc/cmd_button_flags.rs` | 🚚 Mover |
| 85 | `cmd_button_types.rs` | `ipc/cmd_button_types.rs` | 🚚 Mover |
| 86 | `cmd_master_volume.rs` | `ipc/cmd_master_volume.rs` | 🚚 Mover |
| 87 | `cmd_keys.rs` | `ipc/cmd_keys.rs` | 🚚 Mover |
| 88 | `cmd_local_shortcuts.rs` | `ipc/cmd_local_shortcuts.rs` | 🚚 Mover |
| 89 | `cmd_playback.rs` | `ipc/cmd_playback.rs` | 🚚 Mover |
| 90 | `cmd_playback_progress.rs` | `ipc/cmd_playback_progress.rs` | 🚚 Mover |
| 91 | `cmd_tracks.rs` | `ipc/cmd_tracks.rs` | 🚚 Mover |
| 92 | `cmd_locutions.rs` | `ipc/cmd_locutions.rs` | 🚚 Mover |
| 93 | `cmd_export.rs` | `ipc/cmd_export.rs` | 🚚 Mover |
| 94 | `cmd_history.rs` | `ipc/cmd_history.rs` | 🚚 Mover |
| 95 | `cmd_preload.rs` | `ipc/cmd_preload.rs` | 🚚 Mover |
| 96 | `cmd_meta.rs` | `ipc/cmd_meta.rs` | 🚚 Mover |
| 97 | `cmd_updates.rs` | `ipc/cmd_updates.rs` | 🚚 Mover |
| 98 | `cmd_startup_prompts.rs` | `ipc/cmd_startup_prompts.rs` | 🚚 Mover |
| 99 | — | `ipc/mod.rs` | ✨ Crear |
| 100 | — | `ipc/cmd_config.rs` | ✨ Crear (extraído de cmd_profiles) |
| 101 | — | `ipc/cmd_norm.rs` | ✨ Crear (extraído de cmd_profiles) |

---

## Fase 5 — core/

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 102 | `app_setup.rs` | `core/setup.rs` | 🚚 Mover |
| 103 | (parte de `lib.rs`) | `core/state.rs` | ✂️ Extraer AppState |
| 104 | — | `core/mod.rs` | ✨ Crear |
| 105 | — | `core/errors.rs` | ✨ Crear |

---

## Fase 6 — Helpers y deduplicación

| # | Origen | Destino | Operación |
|---|--------|---------|-----------|
| 106 | — | `engine/persist/config_helpers.rs` | ✨ Crear |
| 107 | — | `engine/input/actions.rs` | ✨ Crear |
| 108 | (parte de `ipc/cmd_meta.rs`) | `domain/clock.rs` | ✂️ Extraer |
| 109 | ~15 archivos ipc/cmd_*.rs | (mismos archivos) | 🔧 Usar helpers |
| 110 | 3 archivos con cycle_paleta | (mismos archivos) | 🔧 Usar actions.rs |

---

## Totales

| Operación | Cantidad |
|-----------|----------|
| 🚚 Mover | ~83 archivos |
| ✨ Crear | ~22 archivos (mayormente `mod.rs` + 6 nuevos con lógica) |
| ✂️ Split/Extraer | ~4 operaciones |
| 🔧 Modificar lógica | ~18 archivos (imports + usar helpers) |
| ➖ Sin cambios | `main.rs` (7 líneas) |
