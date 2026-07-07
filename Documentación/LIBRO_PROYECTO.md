# LF Botonera de Efectos — Libro del Proyecto

Guía de lectura completa del repositorio. Lee esto primero si acabas de llegar al proyecto.

> **Documentos relacionados:**
> - Reglas inmutables → [`REGLAS_PROYECTO.md`](REGLAS_PROYECTO.md)
> - Detalles técnicos de arquitectura → [`ARCHITECTURE.md`](ARCHITECTURE.md)
> - Glosario de términos → [`GLOSARIO.md`](GLOSARIO.md)
> - Guía completa para IA → [`../AGENTS.md`](../AGENTS.md) · [`../CLAUDE.md`](../CLAUDE.md)
> - Compilar y publicar → [`COMPILACION_Y_VERSIONES.md`](COMPILACION_Y_VERSIONES.md)

---

## Índice

1. [¿Qué es este proyecto?](#1-qué-es-este-proyecto)
2. [Árbol del repositorio](#2-árbol-del-repositorio)
3. [Árbol del backend Rust](#3-árbol-del-backend-rust)
4. [Árbol del frontend JavaScript](#4-árbol-del-frontend-javascript)
5. [La frontera frontend / backend](#5-la-frontera-frontend--backend)
6. [El viaje de un clic: de la pantalla al altavoz](#6-el-viaje-de-un-clic-de-la-pantalla-al-altavoz)
7. [El modelo de datos en disco](#7-el-modelo-de-datos-en-disco)
8. [Cómo se conectan los módulos Rust](#8-cómo-se-conectan-los-módulos-rust)
9. [Cómo se conectan los módulos JavaScript](#9-cómo-se-conectan-los-módulos-javascript)
10. [El editor de pistas](#10-el-editor-de-pistas)
11. [La precarga de audio (RAM)](#11-la-precarga-de-audio-ram)
12. [Las locuciones dinámicas](#12-las-locuciones-dinámicas)
13. [La compatibilidad con LF Automatizador](#13-la-compatibilidad-con-lf-automatizador)
14. [El sistema de arranque](#14-el-sistema-de-arranque)
15. [Archivos de configuración y scripts](#15-archivos-de-configuración-y-scripts)

---

## 1. ¿Qué es este proyecto?

**LF Botonera de Efectos** es una botonera de sonidos (*soundboard*) para radio y *streaming* en directo. Un operador asigna archivos de audio a botones organizados en paletas (pestañas) dentro de perfiles, y los dispara en tiempo real durante una transmisión.

El proyecto está construido con **Tauri v2 + Rust** (backend) y **Vanilla JavaScript + Vite** (frontend). La filosofía central es que la interfaz de usuario es un "humilde control remoto": dibuja botones, muestra lo que Rust le dice, y reenvía acciones del usuario hacia Rust. Todo el audio, la lógica y los datos críticos viven en el backend.

Es software libre (GPL-3.0-or-later). Su versión actual es la **1.1.2**.

Tiene una **aplicación hermana**, el [LF Automatizador v1.0](https://github.com/yosoyluisfernando/lf-automatizador), con la que comparte los formatos de archivo `.bdelf` (pestañas) y `.bdeplf` (perfiles). La compatibilidad entre ambas es obligatoria.

> Ver el glosario: [botón](#), [paleta](#), [perfil](#), [Tauri](#)

---

## 2. Árbol del repositorio

```
BOTONERA/
│
├── src/                     ← Frontend (HTML, CSS, JS)
├── src-tauri/               ← Backend (Rust, config de Tauri)
├── Documentación/           ← Documentación interna del proyecto
│
├── .github/workflows/       ← CI/CD: build.yml, release-builds.yml
├── .claude/                 ← Configuración de Claude Code (local)
│
│   ── Documentos ──
├── CLAUDE.md                ← Guía completa para Claude AI
├── AGENTS.md                ← Guía para cualquier IA colaboradora
├── README.md                ← Presentación pública (GitHub)
├── COMPILAR.md              ← Instrucciones detalladas de compilación
├── MANUAL.md                ← Manual del usuario final
├── LICENSE                  ← Licencia GPL-3.0-or-later
│
│   ── Desarrollo ──
├── DEV.bat                  ← Arranca la app en modo desarrollo (doble clic)
├── SET-VERSION.bat          ← Sincroniza la versión en los 3 archivos de config
├── SET-VERSION.ps1          ← Lógica PowerShell del script de versión
├── CHANGELOG.md             ← Historial de cambios por versión (Keep a Changelog)
│
│   ── Assets ──
├── icono.jpg                ← Icono fuente (JPG, para diseño)
├── icono_circular.png       ← Icono circular usado en README y webview
│
│   ── Build ──
├── package.json             ← Dependencias npm y scripts (npm run tauri dev/build)
├── package-lock.json        ← Lockfile de npm
├── vite.config.js           ← Configuración de Vite (bundler del frontend)
│
└── Compilados/              ← Artefactos de release locales (gitignoreado)
```

---

## 3. Árbol del backend Rust

Todos los archivos en `src-tauri/src/`. El **límite es 200 líneas por archivo**; si un módulo crece, se divide.

```
src-tauri/src/
│
├── core/                    ← AppState, setup y configuración global
├── model/                   ← Estructuras de datos puras (AppConfig, TrackMeta, etc.)
├── engine/                  ← Motores autónomos
│   ├── audio/               ← rodio/cpal, hilo principal, mezcla, dispositivos
│   ├── dsp/                 ← symphonia, ebur128, cue, fade, waveform
│   ├── cache/               ← LRU RAM, preloader, calentamiento
│   ├── persist/             ← SQLite, config JSON, undo/redo
│   ├── weather/             ← open-meteo, geocoding
│   └── input/               ← Atajos globales de SO, reglas de dispatch
│
├── domain/                  ← Reglas de negocio puras (rutinas, grids, reloj)
├── ipc/                     ← Comandos Tauri (puntos de entrada de UI)
└── lfa_format/              ← Adaptadores bidireccionales con LF Automatizador
```

**Nota:** Anteriormente, el proyecto mantenía ~88 archivos `.rs` en la raíz. A partir de la versión 1.1.3, el código se ha estructurado jerárquicamente en **5 capas** para garantizar la separación de responsabilidades ("Núcleo + Motores").
│
└── lfa_format/              ← Conversión Botonera ↔ formato LFA
    ├── types.rs             ← LfaButton, LfaPaleta, LfaProfile, LfaConfig, LfaKeys
    ├── paleta.rs            ← to/from_lfa_paleta
    └── profile.rs           ← to/from_lfa_profile
```

---

## 4. Árbol del frontend JavaScript

Todos los archivos en `src/js/`. Mismo límite de 200 líneas.

```
src/js/
│
│   ── Núcleo ──
├── main.js                  ← Entry point Vite; solo importa startup.js
├── api.js                   ← Wrapper IPC: invoke(), listen(), emit(), waitForTauri()
├── startup.js               ← Orquesta el arranque; detecta modo pop-out del editor
├── i18n.js                  ← loadLanguage(), t(key), data-i18n
├── theme.js                 ← applyTheme(): clase CSS sin parpadeo
│
│   ── Inicio y configuración ──
├── wizard.js                ← Asistente de primer arranque (3 pasos)
├── settingsModal.js         ← Panel de configuración: audio, atajos, locuciones, precarga
├── settingsLocutions.js     ← Sección de locuciones en ajustes
├── settingsLocutionsTemplate.js ← Plantilla HTML del formulario de locuciones
├── settingsPreload.js       ← Sección de precarga + indicador de RAM usada
├── updateNotifier.js        ← Banner de actualización disponible
├── audioDeviceRecovery.js   ← Detecta al arranque si el dispositivo de audio desapareció
├── preloadDialog.js         ← Diálogo de primer arranque para configurar la precarga
│
│   ── Rejilla y botones ──
├── grid.js                  ← Renderiza y actualiza la rejilla de botones
├── gridDnd.js               ← Drag & drop de archivos externos + reordenamiento Alt+arrastre
├── gridPlayback.js          ← paintAudioTick(): colorea botones (verde/barra roja)
├── contextMenu.js           ← Menú contextual de botón (clic derecho)
├── editModal.js             ← Modal completo de edición de botón
├── editTypes.js             ← Campos específicos por tipo de botón
├── editVolumeControl.js     ← Slider de volumen por botón
├── gainDb.js                ← Conversión lineal ↔ dB para la UI
├── deleteConfirm.js         ← Diálogo de confirmación de borrado
│
│   ── Pestañas y perfiles ──
├── tabs.js                  ← Sistema de pestañas: render, crear, cambiar, indicador audio
├── tabDnd.js                ← Reordenamiento de pestañas con arrastre
├── tabModal.js              ← Modal para crear/editar pestaña
├── profiles.js              ← Selector de perfiles: dropdown, crear, editar, eliminar
├── profileModal.js          ← Modal para crear/editar perfil
├── importer.js              ← Importación de .bdelf/.bdeplf
│
│   ── Atajos de teclado ──
├── shortcuts.js             ← Listener global keydown; despacha a handle_local_shortcut
├── shortcutSave.js          ← Captura y guardado de combinaciones
├── keyInputs.js             ← Input especial que captura pulsaciones
├── mapping.js               ← Modo de mapeo: overlay con teclas asignadas sobre la rejilla
│
│   ── Audio y reproducción ──
├── prelisten.js             ← Panel flotante de pre-escucha: progreso, stop, seek
├── masterVolume.js          ← Slider de volumen master + boost/remember
├── playbackModes.js         ← Radio-buttons de modo global; getCurrentMode()
│
│   ── Barra inferior ──
├── bottomBar.js             ← Init de clockWidget + vuMeter + playbackModes
├── clockWidget.js           ← Reloj/fecha/contador regresivo; escucha clock-tick y audio-tick
├── vuMeter.js               ← Vúmetro estéreo L/R con balística
│
│   ── Editor de pistas ──
├── trackEditor.js           ← Orquestador: analiza, transporta, guarda, abre modal/ventana
├── trackTransport.js        ← Controles Play/Stop: reloj rAF, playFrom(), stop cíclico
├── trackEditorWindow.js     ← Pop-out: abre WebviewWindow, gestiona docking/undocking
├── waveformCanvas.js        ← Dibuja onda en canvas: envolvente, marcadores cue, playhead
│
│   ── Utilidades visuales ──
├── colorPalette.js          ← Selector de 32 colores + color personalizado
├── colorAdapter.js          ← Adapta colores para contraste en tema claro/oscuro
├── numberInputs.js          ← Controles numéricos con +/- y validación
├── appDialog.js             ← Wrapper de tauri-plugin-dialog
├── menuPosition.js          ← Posiciona menús sin salirse de la ventana
├── titlebar.js              ← Barra de título personalizada (min, max, cerrar)
└── typeIcons.js             ← Mapea tipo de botón a icono Unicode
```

**CSS** (`src/css/`): un archivo por componente principal. `theme.css` define todas las *custom properties* de color. `main.css` define el layout. El resto son estilos de componentes específicos.

**i18n** (`src/public/i18n/`): `es.json` es la fuente de verdad. Los cuatro idiomas (es, en, pt-BR, pt-PT) deben tener exactamente las mismas claves.

---

## 5. La frontera frontend / backend

La comunicación entre capas ocurre **solo por dos vías**:

```
Frontend (JS)                     Backend (Rust)
────────────────────────────────────────────────
invoke('comando', args) ────────► #[tauri::command] fn()
                        ◄──────── return valor
                        
listen('evento') ◄──────────────── app.emit('evento', payload)
```

`api.js` es el punto único de acceso. Todos los módulos JS importan `invoke` y `listen` de ahí. **Nadie accede directamente a `window.__TAURI__`**.

> Ver glosario: [IPC](#), [invoke](#), [listen](#)

---

## 6. El viaje de un clic: de la pantalla al altavoz

Este es el flujo más importante del sistema. Entenderlo explica por qué existe cada módulo.

```
1. USUARIO
   └─► Clic en un botón de la rejilla
       │
       ▼
2. grid.js
   └─► invoke('play_button', { id: 'paleta_1_btn_3' })
       │  (el id es "{paleta_id}_btn_{index}")
       │
       ▼
3. cmd_button_playback.rs :: play_button_id()
   ├─ Lee AppConfig → perfil activo → paleta → botón (de RAM, sin I/O de disco)
   ├─ Según type del botón:
   │    "audio"          → play_file()
   │    "time"           → locution_playback::play_time()
   │    "temperature"    → locution_playback::play_climate()
   │    "random_folder"  → random_folder::active_or_next_audio()
   │
   └─ play_file():
       ├─ Lee tracks.db: ¿hay cue/dB para este archivo? ¿sigue vigente (mtime/size)?
       ├─ Combina playback_mode global con flags del botón (loop, stop_other, overlap, restart)
       └─ audio::AudioEngine::play_file() → envía AudioCommand::Play al canal
              │
              ▼
4. audio_thread.rs (hilo dedicado)
   ├─ Decide bus de destino: to_pre=false → device.bus() (salida principal)
   ├─ preload_cache::build_play_source()
   │    ├─ Cache HIT  → CachedSource::new_at(pcm, offset) — seek O(1) instantáneo
   │    └─ Cache MISS → audio_decode::source_from_path() + CuedSource (skip O(n))
   │
   └─ MasterBus::add_source(source, vol_btn, duration, loop, file_gain)
        │  (envuelve en ButtonSource que aplica: muestra × file_gain × vol_btn × master)
        ▼
5. DynamicMixer<f32>
   └─► LevelSource (mide PICO en tiempo real)
        └─► Sink → OutputStreamHandle → dispositivo CPAL → altavoces

6. MIENTRAS SUENA — audio_monitor.rs (hilo 100 ms)
   └─► emite "audio-tick" → Frontend:
        ├─ gridPlayback.js: botón en verde + barra roja de progreso
        ├─ tabs.js: pestaña con indicador de audio
        ├─ clockWidget.js: cuenta regresiva en la barra inferior
        └─ vuMeter.js: vúmetro L/R con balística
```

> Ver glosario: [AudioEngine](#), [AudioCommand](#), [MasterBus](#), [ButtonSource](#), [file_gain](#), [ButtonState](#), [CachedSource](#)

---

## 7. El modelo de datos en disco

El sistema guarda la información del usuario en **dos archivos**, ambos en `%APPDATA%\LF Botonera\` (Windows) o `~/.config/LF Botonera/` (Linux):

### botonera_config.json — la configuración

Contiene todo el estado de la app: perfiles, paletas, botones, ajustes de audio, i18n, tema, precarga, locuciones.

```
AppConfig
  └── profiles[]
        └── ProfileData
              ├── audio: AudioConfig  (dispositivos, atajos globales, modo reproducción)
              └── paletas[]
                    └── PaletaData
                          └── botones[]
                                └── ButtonData (id, type, path, vol, loop_mode, shortcut…)
```

La función `config::save_config()` escribe este JSON en cada cambio. La lectura es con `config::load_config()`, que incluye migración automática desde formatos anteriores.

### tracks.db — metadatos por archivo

Base de datos SQLite. **Una fila por archivo de audio.** Guarda:

| Campo | Qué es |
|---|---|
| `path` | Ruta normalizada del archivo (clave primaria) |
| `mtime`, `size` | Marca temporal del archivo; invalida la fila si el archivo cambia |
| `cue_start_s`, `cue_end_s` | Punto de inicio y fin manual (editor de pistas) |
| `gain_db` | Trim manual en dB |
| `norm_enabled`, `norm_gain_db` | Normalización automática y ganancia calculada |
| `measured_lufs`, `measured_peak_db` | Resultados del último análisis |
| `last_played` | Epoch de la última reproducción (para la precarga OnPlay) |

> Ver glosario: [AppConfig](#), [TrackMeta](#), [LUFS](#), [cue](#), [WAL](#)

---

## 8. Cómo se conectan los módulos Rust

Los módulos se organizan en capas. **Las capas inferiores no conocen las superiores.**

```
┌──────────────────────────────────────────────┐
│  COMANDOS IPC (cmd_*.rs)                     │  ← El frontend llega hasta aquí
│  Reciben argumentos del frontend,            │
│  coordinan el estado y llaman a las capas    │
│  inferiores. Sin lógica de negocio directa.  │
└────────────────┬─────────────────────────────┘
                 │ usa
┌────────────────▼─────────────────────────────┐
│  LÓGICA DE NEGOCIO                           │
│  config.rs, playback_mode.rs,                │
│  locution_playback.rs, random_folder.rs,     │
│  export_tracks.rs, lfa_format/, colors.rs…  │
└────────────────┬─────────────────────────────┘
                 │ usa
┌────────────────▼─────────────────────────────┐
│  MOTOR DE AUDIO                              │
│  audio.rs → audio_thread.rs → master_bus.rs  │
│  → master_button.rs → vu_meter.rs            │
│  cached_source.rs / cue_source.rs            │
│  preload_cache.rs / preloader.rs             │
└────────────────┬─────────────────────────────┘
                 │ usa
┌────────────────▼─────────────────────────────┐
│  PERSISTENCIA                                │
│  config.rs (JSON), db.rs + track_store.rs    │
│  (SQLite), last_played.rs (debounce)         │
└────────────────┬─────────────────────────────┘
                 │ usa
┌────────────────▼─────────────────────────────┐
│  TIPOS (types*.rs, button_types.rs)          │
│  Solo structs + serde. Sin lógica pesada.    │
└──────────────────────────────────────────────┘
```

**AppState** es el objeto que Tauri inyecta en cada comando IPC. Contiene todo el estado compartido:

```
AppState {
  config:         Arc<Mutex<AppConfig>>      ← configuración completa
  audio:          Mutex<AudioEngine>         ← fachada del motor de audio
  history:        Mutex<ConfigHistory>       ← pila undo/redo
  random_folders: Mutex<RandomFolderState>
  tracks:         Arc<Mutex<TrackStore>>     ← acceso a tracks.db (Arc para compartir con flusher)
  waveforms:      Mutex<WaveformCache>       ← envolventes calientes en memoria
  track_analysis: Mutex<TrackAnalysisCache>  ← resultados de análisis en memoria
  last_played:    LastPlayed                 ← buffer de última reproducción (debounce)
}
```

> Ver glosario: [AppState](#), [Arc<Mutex>](#), [IPC](#)

---

## 9. Cómo se conectan los módulos JavaScript

El árbol de dependencias del frontend sigue un patrón **hub-and-spoke**: `startup.js` importa y coordina todos los módulos; los módulos solo hablan entre sí cuando hay una relación de composición clara.

```
main.js
  └── startup.js ──────────────────────────────────────────────────────────┐
        ├── api.js               (único acceso a window.__TAURI__)         │
        ├── i18n.js              (t(), loadLanguage())                     │
        ├── theme.js             (applyTheme())                            │
        │                                                                  │
        ├── wizard.js            (si is_first_boot)                        │
        │                                                                  │
        ├── tabs.js              ◄─── tabModal.js                         │
        ├── tabDnd.js                                                      │
        ├── profiles.js          ◄─── profileModal.js                     │
        ├── shortcuts.js         ◄─── keyInputs.js, shortcutSave.js       │
        ├── grid.js              ◄─── contextMenu.js ◄─── editModal.js    │
        ├── gridDnd.js                               ◄─── editTypes.js    │
        ├── gridPlayback.js                          ◄─── editVolumeControl.js
        │                                                                  │
        ├── bottomBar.js                                                   │
        │     ├── clockWidget.js                                           │
        │     ├── vuMeter.js                                               │
        │     └── playbackModes.js                                         │
        │                                                                  │
        ├── settingsModal.js                                               │
        │     ├── settingsLocutions.js                                     │
        │     └── settingsPreload.js                                       │
        │                                                                  │
        ├── prelisten.js                                                   │
        ├── masterVolume.js                                                │
        ├── mapping.js                                                     │
        ├── updateNotifier.js                                              │
        ├── audioDeviceRecovery.js                                        │
        └── preloadDialog.js                                              │
                                                                           │
        Eventos Rust que llegan en runtime:                                │
        ├── 'clock-tick'    → clockWidget.js                               │
        ├── 'audio-tick'    → gridPlayback.js + clockWidget.js + vuMeter.js + tabs.js
        │                  → también dispara CustomEvent('lf-audio-tick') en el DOM
        ├── 'weather-updated' → settingsLocutions.js                      │
        ├── 'global-shortcut-refresh' → _refresh()                        │
        ├── 'track-editor-dock' → trackEditor.js (lazy import)            │
        └── 'track-analysis-progress' → trackEditor.js                    │
```

El editor de pistas se importa de forma **dinámica** (`import('./trackEditor.js')`) para no penalizar el tiempo de arranque.

---

## 10. El editor de pistas

El editor de pistas es la función más compleja del sistema. Permite al usuario ver la forma de onda de un audio, fijar puntos de inicio y fin (cue), ajustar el volumen en dB y aplicar normalización automática.

**Módulos implicados:**

| Módulo | Rol |
|---|---|
| `trackEditor.js` | Orquestador: abre el modal o la ventana pop-out, pide análisis a Rust, conecta todos los sub-módulos |
| `trackTransport.js` | Controles de reproducción: Play, Stop cíclico, reanudar. Usa `requestAnimationFrame` para el cursor |
| `waveformCanvas.js` | Dibuja la onda en un `<canvas>`: envolvente, marcadores de cue, playhead. Gestiona zoom y arrastre |
| `trackEditorWindow.js` | Gestiona el modo ventana flotante (pop-out y docking) |
| `editor_analysis.rs` | Orquesta el análisis en Rust con progreso, caché en memoria, `tracks.db` y caché persistente |
| `audio_analysis.rs` | Decodifica el PCM completo, mide LUFS, calcula ganancia sugerida, construye la envolvente |
| `waveform.rs` | Almacena la envolvente de alta resolución; `view()` agrega para el zoom actual |
| `waveform_disk.rs` | Persiste envolventes del editor en disco con límites de tamaño/antigüedad |
| `waveform_binary.rs` | Serializa y lee la envolvente persistente del editor |
| `track_analysis_cache.rs` | Caché en memoria del análisis completo para no re-analizar si el archivo no cambió (mtime/size) |
| `track_store.rs` | Persiste cue, dB y normalización en SQLite |
| `cmd_tracks.rs` | Comandos IPC del editor; `analyze_track` delega en `editor_analysis.rs` mediante worker bloqueante |

**Modelo de ganancia de 3 capas:**
```
señal final = muestra × file_gain × vol_botón × master
```
- `file_gain` viene de `TrackMeta.effective_gain_linear()` = norm_gain + gain_db → lineal
- `vol_botón` es `ButtonData.vol` (se preserva para compatibilidad `.bdelf`)
- `master` es el volumen global del perfil

> Ver glosario: [cue](#), [LUFS](#), [dBFS](#), [file_gain](#), [envolvente](#)

---

## 11. La precarga de audio (RAM)

Para evitar el "jitter" del disco al disparar un sonido, el sistema puede precargar archivos cortos en RAM.

**Cadena de precarga:**

```
PreloadConfig (estrategia, presupuesto RAM, umbral duración)
      │
      ├─► warm_for_strategy() al arrancar
      │     ├─ FullProfile  → precargar TODOS los audios cortos del perfil
      │     ├─ VisibleTabs  → precargar solo la pestaña activa
      │     └─ OnPlay       → precargar lo que se reprodujo en las últimas N horas
      │
      └─► seed_preload() al reproducir (solo estrategia OnPlay)
            └─ Encola el archivo en Preloader (hilo aparte)
                  └─► decode_pcm(path) → PreloadCache::insert()
                              │
                              ▼
                      PreloadCache (HashMap<path, Arc<CachedPcm>> + LRU + bytes_used)
                              │
                              ▼
                      build_play_source() al reproducir
                        ├─ Hit → CachedSource::new_at(pcm, offset) — O(1)
                        └─ Miss → audio_decode + CuedSource — O(n)
```

> Ver glosario: [PreloadCache](#), [LRU](#), [CachedSource](#), [jitter](#)

---

## 12. Las locuciones dinámicas

Los botones de tipo `time`, `temperature` y `humidity` reproducen archivos de audio que representan la hora o los datos del clima en el momento en que se pulsa el botón.

```
Botón type="time"
  └─► cmd_button_playback → locution_playback::play_time()
        └─► locutions::parse_time(hora_actual, carpeta_configurada)
              └─► Construye lista de archivos: HRS14.mp3, MIN30.mp3…
                    └─► audio::play_sequence(id, paths, vol, dur)
                          └─► SequenceSource: reproduce los archivos en orden como uno solo

Botón type="temperature" / "humidity"
  └─► locution_playback::play_climate()
        └─► weather.rs proporciona el valor actual (caché 10 min; open-meteo API)
              └─► locutions::parse_climate(valor, carpeta)
                    └─► mismo mecanismo de secuencia
```

> Ver glosario: [locución](#), [SequenceSource](#), [open-meteo](#)

---

## 13. La compatibilidad con LF Automatizador

Los archivos `.bdelf` (una paleta) y `.bdeplf` (un perfil completo) son el puente entre la Botonera y el LF Automatizador. El LFA usa nombres de campo distintos (`file`, `bg`, `text`, `loop`, `stopOther`).

```
Exportar desde la Botonera:
  cmd_export.rs
    ├─ lfa_format::to_lfa_paleta(paleta) → JSON compatible con LFA
    └─ export_tracks.rs → añade bdelf_tracks {ruta → cue+dB} como campo OPCIONAL
         (el LFA ignora este campo; la Botonera lo usa al importar para restaurar ediciones)

Importar en la Botonera:
  cmd_export.rs
    ├─ lfa_format::from_lfa_paleta(json) → PaletaData
    └─ export_tracks.rs::restore() → escribe cue+dB en tracks.db
         (re-sella mtime/size del archivo local para que el cue aplique en esta máquina)
```

**Regla de oro:** cualquier campo nuevo en el JSON de la Botonera debe tener `#[serde(default)]` para que el LFA pueda leer el archivo ignorando ese campo.

> Ver glosario: [bdelf](#), [bdeplf](#), [serde_default](#), [LFA](#)

---

## 14. El sistema de arranque

Al lanzar la aplicación, ocurre la siguiente secuencia:

**Backend (Rust):**
1. `main.rs` → `lib::run()`
2. `lib::run()` crea `AppState`: carga `botonera_config.json`, abre `tracks.db`, crea `AudioEngine`
3. Tauri llama `app_setup::on_setup()`:
   - Aplica el dispositivo de audio del perfil activo
   - Fija el presupuesto de RAM de la caché de precarga
   - Arranca 4 hilos: monitor de audio, reloj, flusher de historial, refresco de clima
   - Ejecuta la precarga caliente según la estrategia configurada
   - Registra el hook de cierre para volcar el historial pendiente

**Frontend (JS):**
1. `main.js` espera `DOMContentLoaded` → llama `startApp()`
2. `startup.js::startApp()` espera que `window.__TAURI__` esté disponible
3. Detecta si la URL contiene `?editor=path` (modo ventana pop-out del editor)
4. Invoca `get_config` → aplica tema y carga el idioma
5. Si `is_first_boot` → muestra el wizard; si no → carga todos los módulos y la rejilla
6. Suscribe eventos Rust: `clock-tick`, `audio-tick`, `weather-updated`, etc.

> Ver glosario: [AppState](#), [is_first_boot](#), [wizard](#), [pop-out](#)

---

## 15. Archivos de configuración y scripts

| Archivo | Propósito |
|---|---|
| `package.json` | Define `npm run dev` (inicia Vite) y `npm run tauri build` (compila todo) |
| `vite.config.js` | Configuración del bundler del frontend |
| `src-tauri/Cargo.toml` | Dependencias Rust y metadata del paquete |
| `src-tauri/tauri.conf.json` | Nombre de app, versión, ventanas, bundle (icon, msi upgradeCode, nsis) |
| `src-tauri/capabilities/default.json` | Permisos del webview Tauri (⚠ sin BOM) |
| `src-tauri/build.rs` | Script de build Tauri (no tocar) |
| `.github/workflows/build.yml` | CI de desarrollo: compila en push/PR |
| `.github/workflows/release-builds.yml` | CI de release: al publicar tag `v*`, compila y sube artefactos |
| `DEV.bat` | Arranca la app en modo desarrollo (`npm run tauri dev`); doble clic para usar |
| `SET-VERSION.bat` | Sincroniza la versión en `package.json`, `Cargo.toml` y `tauri.conf.json` de una sola vez |
| `SET-VERSION.ps1` | Lógica PowerShell del script anterior |
| `CHANGELOG.md` | Historial de cambios por versión (formato Keep a Changelog) |

**Sincronización de versión:** usar `SET-VERSION.bat X.Y.Z` antes de cada release. El script actualiza los tres archivos en un paso y recuerda regenerar `Cargo.lock` con `cargo check`.

> Ver [`COMPILACION_Y_VERSIONES.md`](COMPILACION_Y_VERSIONES.md) para el proceso completo de release.
