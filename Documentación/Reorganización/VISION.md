# Visión Arquitectónica

## El problema

Hoy LF Botonera tiene **88 archivos `.rs` en un solo directorio plano** y
**54 archivos `.js` en otro directorio plano**. El código es de alta calidad
(200 líneas máximo por archivo, sin dependencias circulares, principio
Rust-first respetado), pero la organización de carpetas no refleja la
arquitectura real del sistema.

Un desarrollador nuevo (o una IA) que abre `src-tauri/src/` ve 88 archivos
sin jerarquía y no puede distinguir qué pertenece al motor de audio, qué al
sistema de caché, qué es un comando IPC, y qué es un tipo de datos.

## La solución

Reorganizar el código en **5 capas** con responsabilidades claras:

```
┌────────────────────────────────────────────┐
│            PUERTA IPC (ipc/)               │
│  Capa fina: recibe, delega, responde       │
├────────────────────────────────────────────┤
│          DOMINIO (domain/)                 │
│  Reglas de negocio puras                   │
│  No accede a disco ni a red               │
├────────────────────────────────────────────┤
│          MOTORES (engine/)                 │
│  Audio, DSP, Caché, Persistencia,          │
│  Clima, Entrada                            │
│  Cada motor es un subsistema autónomo      │
├────────────────────────────────────────────┤
│          MODELO (model/)                   │
│  Structs + serde. Cero lógica, cero I/O   │
├────────────────────────────────────────────┤
│          NÚCLEO (core/)                    │
│  AppState, setup, errores                  │
│  El punto de gravedad de la app            │
└────────────────────────────────────────────┘
```

## Los 6 motores

| Motor | Responsabilidad | Archivos actuales | Visión futura |
|-------|----------------|-------------------|---------------|
| **Audio** | Reproducción, mezcla, dispositivos, monitoreo, decodificación | 13 archivos `audio*.rs` + `master_*.rs` + `vu_meter.rs` | — |
| **DSP** | Análisis (LUFS, peak), cue, fade, forma de onda | 5 archivos: `audio_analysis`, `cue_*`, `fade_ramp`, `waveform` | — |
| **Caché** | Precarga LRU, preloader, calentamiento, análisis | 5 archivos: `preload_*`, `cached_source`, `track_analysis_cache` | — |
| **Persistencia** | Config JSON, SQLite, undo/redo, last_played | 6 archivos: `config*`, `db`, `track_store`, `last_played` | — |
| **Clima** | API Open-Meteo, geocode, locuciones, secuencias | 4 archivos: `weather`, `geocode`, `locutions`, `locution_playback` | — |
| **Entrada** | Atajos de teclado, dispatch de acciones | 4 archivos: `global_shortcuts`, `shortcut_rules`, `tab_reorder` + nuevo `actions.rs` | Stream Deck, botoneras físicas, teclados macro, MIDI controllers |

### Motor de Entrada — Visión a futuro

El motor de entrada (`engine/input/`) se diseña desde el inicio como un sistema
extensible de **fuentes de entrada**. Hoy la única fuente es el teclado (atajos
locales y globales del SO). Mañana podrán añadirse:

```
engine/input/
├── mod.rs              ← Fachada del motor
├── actions.rs          ← Acciones centralizadas (cycle_paleta, play_button, etc.)
├── rules.rs            ← Detección de colisiones
├── keyboard.rs         ← Fuente: atajos de teclado del SO (tauri-plugin-global-shortcut)
├── tab_reorder.rs      ← Reorden de pestañas por atajo
│
│   — Futuro —
├── streamdeck.rs       ← Fuente: Elgato Stream Deck
├── midi.rs             ← Fuente: controladores MIDI
└── macro_keyboard.rs   ← Fuente: teclados macro / botoneras USB
```

Cada fuente de entrada traduce su evento nativo al mismo `InputAction` que
`actions.rs` ya sabe ejecutar. Así, un botón de Stream Deck y una tecla F5
llegan al mismo punto de dispatch.

## Qué NO cambia

- La **lógica interna** de cada archivo (el código que hace el trabajo real)
- El **esquema de datos** (AppConfig, TrackMeta, ButtonData, etc.)
- Los **comandos IPC** (mismos nombres, mismas firmas, mismo comportamiento)
- Los **eventos Tauri** (audio-tick, clock-tick, etc.)
- El **frontend** en su primera fase (se reorganiza aparte)
- Los **tests** (deben seguir pasando sin modificación)

## Qué SÍ cambia

- La **ubicación** de cada archivo (de plano a jerárquico)
- Las **rutas de import** (`use crate::audio_thread` → `use crate::engine::audio::thread`)
- Las **declaraciones de módulo** (`mod audio_thread;` → `mod engine;` con sub-mods)
- Se **crean ~6 archivos nuevos** pequeños (helpers, deduplicación, errores, actions)
- Se **eliminan ~3 duplicaciones** de lógica (cycle_paleta ×3, búsqueda de perfil ×15+)

## La metáfora

Es como reorganizar una oficina: los mismos papeles, la misma información,
el mismo trabajo que hacen. Pero en vez de tener 88 carpetas apiladas en una
mesa, cada departamento tiene su archivador propio con etiquetas claras.
Y el departamento de "Entrada" se diseña con espacio vacío en el archivador
para cuando lleguen los nuevos dispositivos.
