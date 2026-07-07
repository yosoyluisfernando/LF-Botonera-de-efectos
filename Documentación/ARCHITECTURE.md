# Arquitectura de LF Botonera de Efectos

Documentación técnica para programadores que colaboran o estudian el proyecto.

---

## Visión general

LF Botonera de Efectos sigue una arquitectura **frontend ligero / backend pesado**:

```
┌─────────────────────────────────────────────────────────────┐
│  Frontend (Vanilla JS + Vite + WebView2)                    │
│  Dibuja la UI, captura gestos del usuario, envía comandos   │
│  IPC. No procesa audio. No calcula estado crítico.          │
└────────────────────────┬────────────────────────────────────┘
                         │  IPC (window.__TAURI__.core.invoke)
                         │  Eventos (window.__TAURI__.event.listen)
┌────────────────────────▼────────────────────────────────────┐
│  Backend Rust (Tauri v2)                                    │
│  Audio (rodio/cpal), DSP (ebur128, symphonia), config       │
│  (serde_json), base de datos (rusqlite/SQLite), HTTP        │
│  (ureq), atajos globales del SO.                            │
└─────────────────────────────────────────────────────────────┘
```

La UI nunca contiene lógica de negocio: valida lo que Rust le da, lo pinta y reenvía las acciones del usuario hacia Rust.

---

## Estructura de directorios

```
BOTONERA/
├── src/                     Frontend HTML/CSS/JS
│   ├── index.html           Punto de entrada del webview
│   ├── js/                  Arquitectura en 3 capas
│   │   ├── bridge/          Capa IPC (api.js)
│   │   ├── ui/              Componentes visuales y renderizado
│   │   └── util/            Helpers y utilidades
│   ├── css/                 Hojas de estilo por componente
│   └── public/
│       └── i18n/            Traducciones: es.json (fuente), en, pt-BR, pt-PT
│
├── src-tauri/               Backend Rust + configuración Tauri
│   ├── Cargo.toml           Dependencias Rust
│   ├── tauri.conf.json      Config de la app (nombre, versión, ventanas, bundle)
│   ├── capabilities/        Permisos del webview (default.json sin BOM)
│   ├── icons/               Iconos del instalador
│   └── src/                 Arquitectura Núcleo + Motores
│       ├── core/            AppState, configuración global y setup
│       ├── model/           Estructuras de datos (AppConfig, etc.)
│       ├── engine/          Motores (audio, dsp, caché, input, weather)
│       ├── domain/          Reglas de negocio puras
│       └── ipc/             Comandos Tauri (endpoints)
│
├── Documentación/           Documentación interna del proyecto
│   ├── REGLAS_PROYECTO.md   Las 14 reglas inmutables (lectura obligatoria)
│   └── COMPILACION_Y_VERSIONES.md  Proceso de release y notas de antivirus
│
├── .github/workflows/       Automatización de builds y releases
├── CLAUDE.md                Guía completa para IAs colaboradoras
├── COMPILAR.md              Instrucciones detalladas de compilación
├── MANUAL.md                Manual del usuario final
└── README.md                Presentación pública del proyecto (GitHub)
```

---

## Separación de responsabilidades

### Backend actual en 5 capas

El backend está organizado para que cada cambio tenga un lugar natural:

| Capa | Qué contiene | Regla práctica |
|---|---|---|
| `core/` | `AppState`, setup inicial, errores comunes y arranque de hilos | Coordina la aplicación, no implementa lógica de producto |
| `model/` | Tipos serializables: configuración, botones, pistas, precarga, normalización | Datos puros con `serde`; todo campo nuevo compatible debe usar `#[serde(default)]` |
| `engine/` | Motores técnicos: audio, DSP, caché, persistencia, clima, entrada | Trabajo pesado, I/O, hilos, base de datos, audio y procesamiento |
| `domain/` | Reglas de negocio: botones, rejilla, reproducción, exportación LFA | Decisiones del producto sin depender de Tauri ni de la UI |
| `ipc/` | Comandos Tauri expuestos al frontend | Puerta fina: recibe datos, llama a `domain/` o `engine/`, devuelve respuesta |

Los motores actuales son:

| Motor | Responsabilidad |
|---|---|
| `engine/audio/` | Reproducción, mezcla, buses main/pre, dispositivos, VU, hilo de audio |
| `engine/dsp/` | Análisis de audio, LUFS, cue, fade, waveform y análisis del editor |
| `engine/cache/` | Precarga RAM, caché de análisis, caché persistente de waveforms |
| `engine/persist/` | `botonera_config.json`, `tracks.db`, historial y últimos reproducidos |
| `engine/input/` | Atajos globales/locales, reglas de conflicto y acciones de teclado |
| `engine/weather/` | Geocoding, clima, locuciones dinámicas y reproducción asociada |

El frontend está organizado en 3 capas:

| Capa | Responsabilidad |
|---|---|
| `src/js/bridge/` | Acceso único a Tauri IPC y eventos |
| `src/js/ui/` | Componentes visuales, modales, rejilla, editor, ajustes |
| `src/js/util/` | Helpers sin estado crítico: i18n, colores, formato, inputs |

### Lo que hace el frontend

- Renderizar la rejilla de botones, pestañas y perfiles con datos que vienen de Rust.
- Capturar clics, drag & drop y teclado; llamar al IPC Rust correspondiente.
- Suscribirse a eventos Rust (`audio-tick`, `clock-tick`, `weather-updated`, `track-analysis-progress`) y actualizar la pantalla.
- Mostrar modales de edición, configuración y el editor de pistas.

### Lo que hace Rust

- Todo el audio (reproducción, mezcla, enrutamiento, VU).
- DSP: análisis de loudness LUFS (ebur128), envolvente de onda (symphonia).
- Precarga RAM (caché LRU).
- Persistencia de configuración (JSON) y metadatos de pistas (SQLite).
- Atajos de teclado globales del SO.
- Locuciones dinámicas de hora y clima (open-meteo).
- Export/import de formatos `.bdelf` / `.bdeplf`.
- Verificación de actualizaciones (GitHub Releases API).

---

## Flujo de reproducción de un botón

```
1. Usuario pulsa botón en la rejilla
2. grid.js → invoke('play_button', { id })
3. cmd_button_playback::play_button_id()
   a. Lee el botón de AppConfig (en memoria, sin I/O)
   b. Según type: audio → play_file | time → locución | random_folder → carpeta aleatoria
   c. Consulta tracks.db por el archivo: cue, dB, normalización (mtime/size para validar)
   d. Combina modo global (AudioConfig.playback_mode) con flags del botón
   e. Envía AudioCommand::Play al canal del hilo de audio
4. engine/audio/thread.rs recibe el comando:
   a. Decide bus de destino (main o pre)
   b. build_play_source: cache hit → O(1) seek; cache miss → decode + skip O(n)
   c. MasterBus::add_source → ButtonSource en el DynamicMixer
5. engine/audio/monitor.rs detecta el nuevo ButtonState → emite "audio-tick" cada 100ms
6. gridPlayback.js pinta el botón en verde + barra de progreso roja
```

---

## Pipeline del editor de pistas

```
Usuario abre el editor de pista
    │
    ▼
trackEditor.js → invoke('analyze_track', { path })
    │
    ▼
cmd_tracks::analyze_track (Rust)
    ├── spawn_blocking → engine::dsp::editor_analysis::analyze_track()
    ├── Emite "track-analysis-progress" por etapas: cache, decode, analyze, save, cleanup
    ├── Comprueba TrackAnalysisCache en memoria (mtime/size)
    ├── Si tracks.db sigue válido + waveform_disk hit:
    │     └── Devuelve resultado sin decodificar el audio completo
    ├── Si tracks.db sigue válido + falta waveform:
    │     └── Reconstruye solo WaveEnvelope, guarda caché persistente y devuelve
    ├── Si no hay caché válida:
    │     ├── Decodifica PCM completo (symphonia)
    │     ├── Mide LUFS integrado (ebur128)
    │     ├── Mide pico dBFS
    │     ├── Calcula ganancia sugerida según configuración global
    │     └── Construye WaveEnvelope (min/max por bucket, hasta 120k puntos)
    ├── Upsert en tracks.db (preserva cue/dB del usuario si ya había fila)
    ├── Guarda WaveEnvelope en caché persistente de disco
    ├── Inserta PCM en PreloadCache solo si la precarga está activa y el archivo cabe
    └── Devuelve AnalysisResult al frontend
    │
    ▼
waveformCanvas.js: dibuja envolvente en <canvas>
    ├── Amplitud proporcional al gain_db actual (refleja el volumen real)
    └── Pinta en rojo las muestras que saturen (clip)

trackTransport.js: cursor de reproducción con requestAnimationFrame
    ├── Al pulsar Play: registra startClock = performance.now() - playOrigin
    └── Loop rAF: t = performance.now() - startClock; actualiza posición del cursor
```

---

## Caché de precarga RAM

```
PreloadCache (HashMap<String, Arc<CachedPcm>> + VecDeque LRU + bytes_used)
     │
     │ Llenado por 3 fuentes:
     ├── analyze_track() → cachea PCM solo si preload.enabled y duration <= max_duration_s
     ├── Preloader hilo → recibe rutas por canal; llama decode_pcm(); inserta
     └── warm_* en arranque → warm_for_strategy (FullProfile/VisibleTabs)
                              warm_onplay_recent (OnPlay + TTL desde last_played)

Al reproducir:
build_play_source(cache, path, loop, cue_start, cue_end)
    ├── cache.get(path) == Some → CachedSource::new_at(pcm, offset_samples) — O(1)
    └── cache.get(path) == None → audio_decode::source_from_path + CuedSource — O(n)
```

---

## Sistema de i18n

1. `es.json` es la fuente de verdad. Todos los demás idiomas deben tener exactamente las mismas claves.
2. `i18n.js::loadLanguage(lang)` carga el JSON desde `public/i18n/`.
3. `t(key)` devuelve la cadena; `key` usa notación de punto (`"button.play"`, `"errors.fatal_ipc"`).
4. El HTML usa `data-i18n="key"` para actualización automática al cambiar idioma.
5. El reloj (`cmd_meta.rs`) formatea hora y fecha según el idioma activo en Rust (sin reiniciar el hilo).

---

## Sistema de temas

- Las custom properties CSS en `theme.css` definen todas las variables de color.
- La clase `html.theme-dark` / `html.theme-light` selecciona el conjunto de variables.
- `theme.js::applyTheme()` añade la clase antes de que el navegador pinte; sin parpadeo blanco.
- `colorAdapter.js` ajusta colores de usuario (fondo/texto) para mantener contraste en cualquier tema.

---

## Compatibilidad con LF Automatizador

El LFA usa nombres de campo distintos en JSON (`file`, `bg`, `text`, `loop`, `stopOther`). La conversión vive en `src-tauri/src/domain/export/lfa_format/`:

```
Botonera ──► to_lfa_paleta() ──► LfaPaleta (JSON .bdelf compatible con LFA)
LFA .bdelf ──► from_lfa_paleta() ──► PaletaData (campos desconocidos ignorados por serde)
```

**Regla de compatibilidad:** todo campo nuevo en el formato Botonera debe tener `#[serde(default)]` para que el LFA pueda deserializar la estructura ignorando el campo. Nunca añadir campos obligatorios a los tipos LFA.

El campo `bdelf_tracks` (cue/dB por archivo) es opcional y el LFA lo ignora. Al importar, `domain/export/tracks.rs::restore()` reescribe los metadatos en `tracks.db` adaptados al sistema de archivos local.

---

## Cómo añadir una nueva función

### Nuevo comando IPC

1. Crear o elegir el `ipc/cmd_*.rs` correspondiente.
2. Añadir la función con `#[tauri::command]`.
3. Registrar el comando en `ipc/register.rs` dentro de `lf_invoke_handlers!`.
4. Llamar desde el frontend mediante `src/js/bridge/api.js`.
5. Si la función lee o modifica `AppConfig`, usar `state.config.lock().unwrap()` y llamar `engine::persist::config_io::save_config(&cfg)`.
6. Si aparece lógica de negocio, moverla a `domain/` o `engine/`; el archivo IPC debe quedar como una puerta fina.

### Nuevo tipo de botón

1. Añadir la variante en `domain/button/types.rs`.
2. Añadir el caso en `ipc/cmd_button_playback.rs`.
3. Añadir el renderizado en `src/js/ui/editTypes.js`.
4. Añadir la clave i18n en `es.json` y los 3 idiomas restantes.

### Nueva clave de configuración

1. Añadir el campo con `#[serde(default)]` en el struct correspondiente dentro de `model/`.
2. Implementar el `Default` o usar `#[serde(default = "fn")]`.
3. Añadir el comando IPC de getter/setter si la UI necesita leerlo/escribirlo.
4. Añadir la clave i18n si tiene texto visible.

### Nueva clave i18n

1. Añadir la clave en `src/public/i18n/es.json` (fuente de verdad).
2. Añadir la misma clave en `en.json`, `pt-BR.json` y `pt-PT.json`.
3. Verificar con: las 4 claves de JSON deben ser idénticas entre sí.

---

## Testing

```bash
# Tests unitarios de Rust (suite actual: 61 passed, 1 ignored)
cd src-tauri
cargo test --lib

# Build completo del frontend
cd ..
npm run build

# Límite de 200 líneas por archivo Rust/JS
wc -l src-tauri/src/**/*.rs src/js/**/*.js
```

Los tests cubren:
- `engine/persist/db.rs`: migración, idempotencia, `normalize_key`
- `model/track.rs`: `sanitized_cue`, `effective_duration_s`, casos extremos
- `engine/audio/monitor.rs`: `compute_display_time` con múltiples instancias
- `engine/cache/cached_source.rs`, `engine/dsp/cue_source.rs`: seek y bucle
- `engine/cache/preload.rs`: LRU, presupuesto RAM
- `engine/cache/waveform_disk.rs`, `engine/cache/waveform_binary.rs`: caché persistente de waveforms
- `ipc/cmd_preload.rs`: validación de parámetros de precarga
- `ipc/cmd_startup_prompts.rs`: recordatorios de novedades y donación
- `engine/persist/tracks.rs`: upsert preservando ediciones, `recent_paths`
- `engine/persist/last_played.rs`: debounce

No existen tests de UI (Tauri no expone un harness de integración para el webview). La verificación visual la hace el usuario en su máquina.

---

## Dependencias externas clave

| Crate | Versión | Uso |
|---|---|---|
| `tauri` | 2 | Framework desktop/webview |
| `rodio` | 0.19 | Audio output (cpal + buffer pipeline) |
| `symphonia` | 0.5.5 | Decodificación de audio (MP3, FLAC, OGG, M4A…) |
| `opus-decoder` | 0.1.1 | Soporte adicional para Opus/OGG |
| `ebur128` | 0.1 | Medición LUFS integrado (EBU R128) |
| `rusqlite` | 0.32 (bundled) | SQLite compilado estático; sin DLL de sistema |
| `serde` + `serde_json` | 1 | Serialización JSON (config, IPC) |
| `ureq` | 2 | HTTP síncrono (clima, updates, geocoding) |
| `chrono` | 0.4 | Fecha/hora localizada |
| `tauri-plugin-global-shortcut` | 2.3.2 | Atajos de teclado del SO |
| `tauri-plugin-window-state` | 2 | Recuerda tamaño/posición de ventana |
| `tauri-plugin-dialog` | 2.7.1 | Diálogos de abrir/guardar archivo |

---

## Notas de release y distribución

Ver `Documentación/COMPILACION_Y_VERSIONES.md` para el procedimiento completo, incluyendo:
- Antivirus false-positives del NSIS (historial del problema y solución).
- Cómo sincronizar la versión en los 3 archivos (package.json, Cargo.toml, tauri.conf.json).
- El `upgradeCode` MSI no debe cambiar entre versiones.

Al publicar un tag `v*`, el CI (`release-builds.yml`) compila automáticamente para Windows y Linux y sube los artefactos al release de GitHub.
