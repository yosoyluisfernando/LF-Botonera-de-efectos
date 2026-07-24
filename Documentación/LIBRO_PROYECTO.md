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
13. [El panel lateral fijo y el reproductor auxiliar](#13-el-panel-lateral-fijo-y-el-reproductor-auxiliar)
14. [La compatibilidad con LF Automatizador](#14-la-compatibilidad-con-lf-automatizador)
15. [El sistema de arranque](#15-el-sistema-de-arranque)
16. [Archivos de configuración y scripts](#16-archivos-de-configuración-y-scripts)

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
├── model/                   ← Estructuras de datos puras (serde, sin lógica)
│   ├── config.rs            ← AppConfig: las PREFERENCIAS (tema, idioma, ajustes…)
│   ├── content.rs           ← El CONTENIDO del usuario: perfil → paleta → botón
├── engine/                  ← Motores autónomos
│   ├── audio/               ← rodio/cpal, hilo principal, mezcla, dispositivos (EFECTOS)
│   ├── player/              ← Reproductor auxiliar: motor propio e independiente
│   ├── dsp/                 ← symphonia, ebur128, cue, fade, waveform
│   ├── cache/               ← LRU RAM, preloader, calentamiento
│   ├── persist/             ← SQLite, config JSON, undo/redo
│   ├── weather/             ← open-meteo, geocoding
│   └── input/               ← Atajos globales de SO, reglas de dispatch
│
├── domain/                  ← Reglas de negocio puras (rutinas, grids, reloj, avance del reproductor)
├── ipc/                     ← Comandos Tauri (puntos de entrada de UI)
└── domain/export/lfa_format/ ← Adaptadores bidireccionales con LF Automatizador
```

**Hay dos motores de audio, no uno.** `engine/audio/` reproduce los efectos y mezcla muchas
fuentes a la vez. `engine/player/` es el reproductor auxiliar del panel lateral (música de
fondo): su propio hilo, su propia salida, su propio dispositivo y su propio volumen. Ninguno
detiene al otro.

**Nota:** Anteriormente, el proyecto mantenía ~88 archivos `.rs` en la raíz. A partir de la versión 1.1.3, el código se ha estructurado jerárquicamente en **5 capas** para garantizar la separación de responsabilidades ("Núcleo + Motores").
## 4. Árbol del frontend JavaScript

Todos los archivos viven bajo `src/js/` en tres capas. Mismo límite de 200 líneas.

```
src/js/
├── bridge/
│   └── api.js               ← Wrapper IPC: invoke(), listen(), emit(), waitForTauri()
├── ui/                      ← Componentes visuales y orquestadores de pantalla
│   ├── main.js              ← Entry point Vite; solo importa startup.js
│   ├── startup.js           ← Orquesta el arranque; detecta modo pop-out del editor
│   ├── grid.js              ← Renderiza y actualiza la rejilla de botones
│   ├── settingsModal.js     ← Panel de configuración
│   ├── trackEditor.js       ← Orquestador del editor de pistas
│   ├── runtimeEvents.js     ← Suscribe "audio-tick" (efectos) y "player-tick" (reproductor)
│   ├── playerView.js        ← Modo reproductor: cola + transporte del panel lateral
│   └── ...                  ← Modales, pestañas, perfiles, VU, pre-escucha, etc.
└── util/                    ← Helpers sin estado crítico
    ├── i18n.js              ← loadLanguage(), t(key), data-i18n
    ├── colorAdapter.js      ← Contraste claro/oscuro
    ├── gainDb.js            ← Conversión visual lineal ↔ dB
    └── ...
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
3. ipc/cmd_button_playback.rs :: play_button_id()
   ├─ Lee AppConfig → perfil activo → paleta → botón (de RAM, sin I/O de disco)
   ├─ Según type del botón:
   │    "audio"          → play_file()
   │    "time"           → engine/weather/playback.rs::play_time()
   │    "temperature"    → engine/weather/playback.rs::play_climate()
   │    "random_folder"  → domain/button/random_folder.rs
   │
   └─ play_file():
       ├─ Lee tracks.db: ¿hay cue/dB para este archivo? ¿sigue vigente (mtime/size)?
       ├─ Combina playback_mode global con flags del botón (loop, stop_other, overlap, restart)
   └─ engine/audio/engine.rs::AudioEngine::play_file() → envía AudioCommand::Play al canal
              │
              ▼
4. engine/audio/thread.rs (hilo dedicado)
   ├─ Pide su bus a la consola: routing::bus_for(to_pre, group) → Efectos | Panel | Cue
   ├─ engine/cache/preload.rs::build_play_source()
   │    ├─ Cache HIT  → CachedSource::new_at(pcm, offset) — seek O(1) instantáneo
   │    └─ Cache MISS → engine/audio/decode.rs + CuedSource (skip O(n))
   │
   └─ engine/audio/attach.rs::attach_button(bus, source, args)
        │  (envuelve en ButtonSource, el CANAL: muestra × file_gain × vol_btn × fade)
        ▼
5. engine/console/ — Bus: DynamicMixer<f32>
   └─► FaderSource (el master: UNA etapa sobre la suma, no una por fuente)
        └─► LevelSource (mide PICO en tiempo real, DESPUÉS del fader)
             └─► play_raw → OutputEndpoint → dispositivo CPAL → altavoces
                 (la tarjeta se abre UNA vez; varios buses en ella se suman en el conector)

6. MIENTRAS SUENA — engine/audio/monitor.rs (hilo 100 ms)
   └─► emite "audio-tick" → Frontend:
        ├─ gridPlayback.js: botón en verde + barra roja de progreso
        ├─ tabs.js: pestaña con indicador de audio
        ├─ clockWidget.js: cuenta regresiva en la barra inferior
        └─ vuMeter.js: vúmetro L/R con balística
```

> Ver glosario: [AudioEngine](#), [AudioCommand](#), [consola](#), [Bus](#), [OutputEndpoint](#), [ButtonSource](#), [file_gain](#), [ButtonState](#), [CachedSource](#)

---

## 7. El modelo de datos en disco

El sistema guarda la información del usuario en **dos archivos**, ambos en `%APPDATA%\LF Botonera\` (Windows) o `~/.config/LF Botonera/` (Linux):

### botonera_config.json — la configuración

Contiene todo el estado de la app: perfiles, paletas, botones, ajustes de audio, i18n, tema, precarga, locuciones.

```
AppConfig
  ├── fixed_panel: alcance, vista ("player"|"buttons"), lado, visibilidad y botones globales
  │     └── playback_mode + solo_mode: reproducción independiente del panel
  │     └── columns, row_mode, rows, width: distribución y capacidad persistentes
  ├── player: PlayerConfig — el reproductor auxiliar (uno solo, global)
  │     ├── tracks[]: la cola (reutiliza ButtonData: admite todos los tipos)
  │     ├── playback_mode: "normal" | "repeat" | "random"
  │     ├── volume: propio, independiente del master (0.0–1.5)
  │     └── output_device: "" = el mismo de los efectos
  └── profiles[]
        └── ProfileData
              ├── fixed_buttons: botones laterales específicos del perfil
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
│  IPC (ipc/cmd_*.rs)                          │  ← El frontend llega hasta aquí
│  Reciben argumentos del frontend,            │
│  coordinan el estado y llaman a las capas    │
│  inferiores. Sin lógica de negocio directa.  │
└────────────────┬─────────────────────────────┘
                 │ usa
┌────────────────▼─────────────────────────────┐
│  LÓGICA DE NEGOCIO                           │
│  domain/playback, domain/button,             │
│  domain/export, domain/colors…               │
└────────────────┬─────────────────────────────┘
                 │ usa
┌────────────────▼─────────────────────────────┐
│  MOTOR DE AUDIO                              │
│  engine/audio, engine/dsp, engine/cache,     │
│  engine/input, engine/weather                │
└────────────────┬─────────────────────────────┘
                 │ usa
┌────────────────▼─────────────────────────────┐
│  PERSISTENCIA                                │
│  engine/persist/config_io.rs, db.rs, tracks.rs│
│  (SQLite), last_played.rs (debounce)         │
└────────────────┬─────────────────────────────┘
                 │ usa
┌────────────────▼─────────────────────────────┐
│  model/*.rs                                  │
│  Solo structs + serde. Sin lógica pesada.    │
└──────────────────────────────────────────────┘
```

**AppState** es el objeto que Tauri inyecta en cada comando IPC. Contiene todo el estado compartido:

```
AppState {
  config:         Arc<Mutex<AppConfig>>      ← configuración completa
  audio:          Mutex<AudioEngine>         ← fachada del motor de audio (efectos)
  player:         Mutex<PlayerEngine>        ← fachada del reproductor auxiliar (motor propio)
  history:        Mutex<ConfigHistory>       ← pila undo/redo
  random_folders: Arc<Mutex<RandomFolderState>>  ← Arc: lo comparte el resolvedor del reproductor
  tracks:         Arc<Mutex<TrackStore>>     ← acceso a tracks.db (Arc para compartir con flusher)
  waveforms:      Mutex<WaveformCache>       ← envolventes calientes en memoria
  track_analysis: Mutex<TrackAnalysisCache>  ← resultados de análisis en memoria
  last_played:    LastPlayed                 ← buffer de última reproducción (debounce)
}
```

El orden de construcción importa: `config`, `random_folders` y `tracks` se crean **antes** que el
`PlayerEngine`, porque su resolvedor necesita esos `Arc` ya montados. Nunca se le pasa el
`AppState` entero: contiene el propio motor y se formaría un ciclo.

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
| `engine/persist/tracks.rs` | Persiste cue, dB y normalización en SQLite |
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
        └─► resolver::resolve_time_files(carpeta)   ← lee la carpeta y mira el reloj
              └─► domain::locution::time_sequence(nombres, hh, mm)   ← DECIDE (puro)
                    └─► audio::play_sequence(id, paths, vol, dur)
                          └─► SequenceSource: reproduce los archivos en orden como uno solo

Botón type="temperature" / "humidity"
  └─► locution_playback::play_climate()
        └─► engine/weather/client.rs proporciona el valor actual (caché 10 min; open-meteo API)
              └─► resolver::resolve_climate_file(carpeta, tipo, valor)
                    └─► domain::locution::climate(nombres, tipo, valor)
```

### El formato de los nombres

El formato es el de **ZaraRadio**, que es el que usa medio gremio:

- **Hora:** `HRS00`–`HRS23` más `MIN00`–`MIN59`. Dos dígitos siempre.
- **En punto:** `HRS14_O` — con la **letra O**, no un cero. Se acepta también `HRS14_0`
  con cero, porque confundirlos es el error más repetido y aceptarlo sale más barato
  que explicarlo.
- **Temperatura:** `TMP025`. Bajo cero, `TMPN003`.
- **Humedad:** `HUM082` (0–100).

**Y también las variantes de RadioBOSS**, que dice lo mismo con otra convención: su manual
da `TMP29.mp3`, `TMP-10.mp3` y `HUM3.mp3` — **sin ceros a la izquierda**, y las negativas
con signo en vez de la `N`. Se aceptan las dos escrituras para que un pack traído de
cualquiera de los dos suene sin renombrar nada. **Salamandra** usa la hora de ZaraRadio tal
cual (y un `TIME_JINGLE` que aquí se ignora, sin estorbar).

**Dinesat y Audicom NO entran, y no es pereza:** no localizan estos audios por nombre de
archivo. Dinesat usa una **categoría de su base de datos** (`HTH ESPAÑOL`, elegida en
Preferencias de la terminal → Clima) más una plantilla en `HTH.ini` donde la temperatura es
un número, no un archivo; Audicom trae su módulo *Meteor* con voces pregrabadas que la
emisora regraba con sus propios locutores. No hay convención con la que ser compatible: no
existe un `TMP025` de Dinesat esperando a que lo leamos. Los packs que se venden como
"compatibles con los cuatro" lo son porque el comprador los coloca a mano, no porque los
cuatro programas lean igual. (Investigado contra las fuentes primarias el 2026-07-17.)

### Dos trampas que costaron sangre

**El número tiene que acabar donde acaba el prefijo.** Aceptar `TMP25` sin ceros abre un
agujero: a 0 grados el nombre corto es `TMP0`, y `TMP025` **empieza por** `TMP0`. Buscando
por prefijo a secas, la radio diría "veinticinco grados" con la ciudad a cero. Por eso
`starts()` exige que lo que siga al alias no sea otra cifra. Lo cubren
`a_cero_grados_no_se_cuela_la_de_veinticinco` y `bajo_cero_no_confunde_tres_con_treinta`.

**`read_dir` no promete ningún orden.** Con `HRS14.mp3` y `HRS14 - las dos.mp3` en la misma
carpeta se elegía uno u otro según el sistema de archivos, y la misma carpeta podía sonar
distinta en dos equipos. Ahora se ordena alfabéticamente y el nombre exacto le gana al
rotulado.

> Ver glosario: [locución](#), [SequenceSource](#), [open-meteo](#)

---

## 13. El panel lateral fijo y el reproductor auxiliar

El panel lateral es un panel **persistente**: no depende de la pestaña activa. Tiene dos
presentaciones (`fixed_panel.view`): **`buttons`**, botones fijos siempre a mano, y **`player`**,
el reproductor auxiliar. La primera mitad de este capítulo va del panel; la segunda, del
reproductor.

### El panel: alcance, identidad y colocación

**El alcance es de toda la barra, no de cada botón.** O una colección global compartida por todos
los perfiles (`fixed_panel.global_buttons`), o una propia de cada perfil
(`ProfileData.fixed_buttons`). Permitir que cada botón eligiera su alcance mezclaría colecciones
y sería más difícil de entender.

Al cambiar de alcance, la colección anterior **se conserva guardada y oculta**: si vuelves atrás,
reaparece intacta. Solo se borra si el usuario lo confirma (`clear_fixed_scope`). Y las dos
colecciones **nunca se fusionan automáticamente**, porque pueden traer nombres, atajos e
identificadores repetidos.

**La identidad depende del alcance vigente.** `button_prefix()` decide el prefijo:

```
alcance global   → fixed_global_btn_{index}
alcance perfil   → fixed_{profile_id}_btn_{index}
```

Así los globales y los de cada perfil nunca colisionan. Al mover un botón entre la rejilla y el
panel (`cmd_fixed_move.rs`) se reasigna el id y se remapean sus referencias.

**Cambiar de lado no reconstruye nada.** `right` invierte las columnas con
`flex-direction:row-reverse` sobre `.workspace-shell[data-fixed-side]`: es CSS puro, sin rehacer
la interfaz y sin parpadeo (regla 8).

**Los audios del panel se precargan** (`engine/cache/warm.rs`): los globales siempre; en alcance
por perfil, solo los del perfil activo — no tiene sentido gastar RAM en perfiles que no se están
usando.

El panel **no viaja en `.bdelf`**, porque ese formato representa *una pestaña* y los fijos no
pertenecen a ninguna. Los de perfil sí van en `.bdeplf`; los globales no, porque no pertenecen a
un perfil.

### El reproductor: por qué existe

El reproductor es una lista de canciones pensada para quien hace radio o streaming y quiere
**música de fondo** sonando mientras dispara sus efectos.

Esa frase esconde la decisión más importante: si la música dependiera del motor de efectos,
cualquier "Detener todo" la cortaría. Por eso el reproductor **no es un grupo dentro del motor
de efectos: es un motor propio**, con su hilo, su `OutputStream`, su dispositivo y su volumen.
El Stop general y el Solo de los efectos no lo tocan; el reproductor tiene su propio Stop.

La idea viene de los reproductores auxiliares del **LF Automatizador v1** (en JavaScript), pero
la arquitectura está tomada del motor de reproducción ya escrito en Rust del **2.0**. No se
copió código: se tradujo y se adaptó (regla 1).

### Los dos decks y el ping-pong

El reproductor mantiene **dos decks** (`engine/player/deck.rs`), cada uno una envoltura de un
`Sink` de rodio que reproduce una pista a la vez. Se alternan:

```
   Deck A (sonando)  ──fin──▶  play(B)   ← instantáneo: ya estaba listo
   Deck B (pre-cargado, en pausa)
        ▲
        └── mientras A sonaba, el motor eligió la siguiente y la dejó preparada aquí
```

Un deck en estado `Loaded` ya tiene la pista **decodificada y en pausa**. Cuando la actual
termina, arrancar la siguiente es solo un `play()`: el motor nunca "se queda pensando". Al
reproducir B, se pre-carga la siguiente en A, y así sucesivamente.

### Quién decide y quién obedece

La separación es deliberada, y sigue la regla 4 (la interfaz no decide nada):

| Módulo | Papel |
|---|---|
| `domain/player/advance.rs` | **La regla pura.** Dado el modo y los índices, dice qué pista sigue. Sin audio, sin I/O, sin estado |
| `engine/player/queue_select.rs` | **Elige** y pre-carga: envuelve la regla pura con lo que necesita el runtime |
| `engine/player/queue_ops.rs` | **El transporte**: arrancar, detener, avanzar, marcar |
| `engine/player/exec.rs` | **Obedece**: única pieza que traduce una `DeckAction` a rodio |
| `engine/player/thread.rs` | El hilo: único dueño de la salida y los decks (rodio no es `Send`) |

La cola **decide y devuelve acciones**; el hilo las ejecuta. Por eso la lógica se puede probar
sin tarjeta de sonido.

### Los cuatro modos, y las dos reglas que mandan sobre ellos

Los modos son **de lista**, no de botón: `normal` (recorre y se detiene al final), `repeat` (da
la vuelta), `random` (al azar sin repetir la actual) y `manual` (no avanza solo).

Por encima del modo hay dos reglas:

- **Lo marcado como siguiente es ley.** Si hay una pista marcada (naranja), se respeta siempre,
  incluso en `manual`. Y la marca sigue a **su canción**, no a la posición: al reordenar la lista
  se conserva por `id`; si esa canción se borra, la marca se cae en vez de saltar a otra.
- **Detener al finalizar.** Al acabar la pista actual, la siguiente no arranca sola hasta que se
  pulsa reproducir; y la que tocaba **se conserva marcada**.

### El naranja es una guía, no un adorno

Con el reproductor detenido, el operador necesita saber **qué va a sonar** si pulsa reproducir.
Por eso el naranja no desaparece al pulsar Stop: lo que estaba pre-cargado pasa a marcado, y si
no había nada marcado se calcula lo que arrancaría (`ensure_upcoming_marked`). El invariante es
**"detenido y con cola ⇒ hay naranja"**, y de él sale también que al añadir canciones a una lista
vacía la primera quede marcada sola. No son dos reglas: es la misma.

### Editar la lista nunca corta la música

Limpiar la lista o abrir otra **no detiene lo que está sonando**. La canción termina, pero queda
*huérfana*: sin verde, porque ya no está en la lista (`current = None`); al acabar, entra la
lista nueva desde el principio. Es el criterio del LF Automatizador v1, cuyo `clearList` vacía
las filas sin tocar la reproducción. Lo garantiza `QueueState::set_entries`, que **no** emite
`StopAll` cuando la pista actual desaparece de la cola nueva.

### Clic y doble clic

Un **clic simple no hace nada**: marcar sin querer al rozar una fila era problemático en directo.
El **doble clic activa** la fila, y quien decide qué significa eso es el motor, no la interfaz
(regla 4): si está detenido la reproduce; si algo suena, la marca como siguiente sin cortar la
música. El IPC es `player_activate_index`; la lógica, `QueueState::activate(index, is_playing)`.
Ese `is_playing` lo aporta el hilo, que es quien conoce los decks: una pista huérfana puede estar
sonando sin figurar ya en la cola, y la cola por sí sola no lo sabría.

### Por qué la hora se resuelve tarde

La cola admite los mismos tipos que los botones: audio, carpeta aleatoria, locución horaria,
temperatura y humedad. Pero hay un detalle que obliga a un tratamiento especial.

La pre-carga ocurre **mientras suena la pista anterior**. Si precargáramos una locución horaria,
diría la hora de hace varios minutos. Lo mismo con el clima. Por eso `engine/player/resolve.rs`
resuelve la hora y el clima **en el relevo**, no antes (`needs_late_resolve`). La carpeta
aleatoria sí se precarga: elegir la canción por adelantado no la estropea.

`resolve.rs` no inventa nada: reutiliza `RandomFolderState`, `resolve_time_folder` /
`resolve_climate_folder` (las MISMAS que usan los botones para ubicar una locución, no una copia)
y `resolve_edit` (cue y ganancia del editor de pistas). Recibe solo los `Arc` que necesita, nunca
el `AppState`, porque el `AppState` contiene el propio motor y se formaría un ciclo.

**Las locuciones no guardan carpeta, y es a propósito.** El LF Automatizador escribe un
*marcador* en la `ruta` de esas filas (`time_locution`, `temperature_locution`,
`humidity_locution`) en vez de un directorio, porque **cada aplicación resuelve la locución con
sus propias carpetas**. Eso es justo lo que hace que una lista creada en el LFA suene aquí y al
revés. Por eso el adaptador convierte el marcador en **carpeta vacía** al importar (y la escribe
de vuelta al exportar): carpeta vacía significa "la que diga Ajustes". Una carpeta propia en la
fila sí se respeta y manda sobre la global.

Una locución son **varios archivos** ("son", "las", "tres"), pero suenan como **una sola pista**
gracias a `SequenceSource`, que ya existía para las locuciones de los botones (hoy en
`engine/audio/sequence.rs`). Por eso los tipos especiales caben en un deck sin tocar el deck ni
el ping-pong.

**Si la resolución falla** —carpeta vacía, sin internet para el clima, falta el archivo de esa
hora— el deck queda en `Failed`, que `poll_finished` trata como terminado: el motor releva y **la
música sigue**, en vez de callarse esperando una locución que no existe.

### Duraciones que no se conocen

Un audio normal tiene duración; una carpeta aleatoria, la hora o el clima **no la tienen hasta
resolverse**. La lista los muestra como `--:--` y **no cuentan para el total**, igual que hace el
LF Automatizador. El total lo suma **Rust** (`PlayerView.total_s`), no la interfaz: qué cuenta y
qué no es una regla de negocio (regla 4).

### Por qué tiene su propio pulso

El `"audio-tick"` de los efectos **no se emite en reposo**: si no hay efectos sonando, calla. Y
la música de fondo suele sonar sin ningún efecto disparado. Colgar la lista de ese tick la
dejaba sin pintar: ni verde, ni naranja, ni tiempo. Por eso `engine/player/monitor.rs` emite su
propio `"player-tick"` cada 100 ms, con el mismo patrón que el monitor de efectos pero
independiente, como lo son los dos motores.

```
engine/player/monitor.rs ──"player-tick"──▶ runtimeEvents.js ──▶ playerView.js
                                                                  verde  = current_index
                                                                  naranja = next_index
```

### El viaje de una canción

```
1. Usuario pulsa ▶  (o la pista actual termina sola)
2. playerView.js → invoke('player_resume')   |  el hilo detecta el fin del deck activo
3. queue_ops.rs::advance() pregunta a domain/player/advance.rs qué sigue
   a. ¿Hay pista marcada? → esa (es ley)
   b. Si no → según el modo
   c. ¿"Detener al finalizar" y fin natural? → parar y conservar la marcada
4. exec.rs carga la pista en el deck ocioso (resolve.rs si es hora/clima/aleatorio)
   reutilizando build_play_source → cue del editor + caché RAM
5. Relevo: suena el deck ya pre-cargado y se detiene el saliente (sin solapar)
6. queue_select.rs pre-carga la siguiente en el deck que quedó libre
7. monitor.rs emite "player-tick" → la lista se pinta sola
```

### En el frontend

| Archivo | Papel |
|---|---|
| `src/js/ui/playerView.js` | Dibuja la cola y el transporte; Limpiar / Abrir / Guardar |
| `src/js/ui/playerDnd.js` | Reordenar arrastrando y recibir lo que se suelta |
| `src/js/ui/settingsPlayer.js` | Ajustes: modo, volumen propio y dispositivo propio |
| `src/js/ui/runtimeEvents.js` | Suscribe `"audio-tick"` y `"player-tick"` |
| `src/css/player.css` | Verde `--player-playing-bg` (sonando) y naranja `--player-next-bg` (siguiente) |

Las listas se guardan y abren en **`.LFPlay`**, el formato del LF Automatizador (ver capítulo
14). Limpiar y Abrir borran la lista, así que antes preguntan si se desea guardarla.

> Ver glosario: [reproductor auxiliar](#), [deck](#), [ping-pong](#), [marcar siguiente](#),
> [detener al finalizar](#), [resolución tardía](#), [player-tick](#), [.LFPlay](#)

---

## 14. La compatibilidad con LF Automatizador

Los archivos `.bdelf` (una paleta) y `.bdeplf` (un perfil completo) son el puente entre la Botonera y el LF Automatizador. El LFA usa nombres de campo distintos (`file`, `bg`, `text`, `loop`, `stopOther`).

```
Exportar desde la Botonera:
  ipc/cmd_export.rs
    ├─ lfa_format::to_lfa_paleta(paleta) → JSON compatible con LFA
    └─ domain/export/tracks.rs → añade bdelf_tracks {ruta → cue+dB} como campo OPCIONAL
         (el LFA ignora este campo; la Botonera lo usa al importar para restaurar ediciones)

Importar en la Botonera:
  ipc/cmd_export.rs
    ├─ lfa_format::from_lfa_paleta(json) → PaletaData
    └─ domain/export/tracks.rs::restore() → escribe cue+dB en tracks.db
         (re-sella mtime/size del archivo local para que el cue aplique en esta máquina)
```

**Regla de oro:** cualquier campo nuevo en el JSON de la Botonera debe tener `#[serde(default)]` para que el LFA pueda leer el archivo ignorando ese campo.

### El tercer formato: `.LFPlay` (listas del reproductor)

Las listas del reproductor auxiliar (capítulo 13) usan el formato de listas del LFA: un array
JSON de filas `{ruta, titulo, duracion, type, target}`. La conversión vive en
`domain/export/lfa_format/playlist.rs`. Una lista guardada aquí se abre en el Automatizador, y
al revés.

```
Botonera ──► to_lfa_row()   ──► LfaPlaylistRow  (JSON .LFPlay)
LFA      ──► from_lfa_row() ──► ButtonData
```

Tres detalles aprendidos leyendo el LFA real, y por qué importan:

- **Los tipos se traducen:** `normal` ↔ `audio` y `random` ↔ `random_folder`; `time`,
  `temperature` y `humidity` coinciden tal cual.
- **Lo que no aplica se ignora, no falla.** El LFA guarda filas de automatización (notas,
  saltos entre listas, ejecutar evento) que en una botonera no tienen sentido. Al importar se
  descartan en silencio en lugar de rechazar el archivo.
- **La duración llega de varias formas.** Puede venir como `duracion` o como `duration`, y unas
  veces como número (`172`) y otras como cadena (`"31"`), porque el LFA la lee con `parseInt`.
  Se aceptan todas las variantes, y una duración ilegible vale 0 en vez de tumbar el archivo:
  vale más una canción sin duración que perder la lista entera.

> Ver glosario: [bdelf](#), [bdeplf](#), [.LFPlay](#), [serde_default](#), [LFA](#)

---

## 15. El sistema de arranque

Al lanzar la aplicación, ocurre la siguiente secuencia:

**Backend (Rust):**
1. `main.rs` → `lib::run()`
2. `lib::run()` crea `AppState`: carga `botonera_config.json`, abre `tracks.db`, crea `AudioEngine`
3. Tauri llama `core::setup::on_setup()`:
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

## 16. Archivos de configuración y scripts

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
| `src-tauri/src/domain/distribution.rs` | Fuente única del canal, plataforma y administrador de actualizaciones incorporados al build |
| `scripts/build-store-msix.ps1` | Compila el canal `store` y genera el MSIX con la identidad oficial |
| `DEV.bat` | Arranca la app en modo desarrollo (`npm run tauri dev`); doble clic para usar |
| `SET-VERSION.bat` | Sincroniza la versión en `package.json`, `Cargo.toml` y `tauri.conf.json` de una sola vez |
| `SET-VERSION.ps1` | Lógica PowerShell del script anterior |
| `CHANGELOG.md` | Historial de cambios por versión (formato Keep a Changelog) |

**Sincronización de versión:** usar `SET-VERSION.bat X.Y.Z` antes de cada release. El script actualiza los tres archivos en un paso y recuerda regenerar `Cargo.lock` con `cargo check`.

> Ver [`COMPILACION_Y_VERSIONES.md`](COMPILACION_Y_VERSIONES.md) para el proceso completo de release.

La plataforma, el formato del paquete y el canal de actualización son decisiones
independientes. La política multiplataforma y la forma de aislar código específico
están en [`ARCHITECTURE.md`](ARCHITECTURE.md#política-para-cambios-específicos-de-windows-o-linux).
