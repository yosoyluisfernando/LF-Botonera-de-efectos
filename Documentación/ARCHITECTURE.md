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
│   ├── js/                  ~50 módulos ES (cada uno <200 líneas)
│   ├── css/                 Hojas de estilo por componente
│   └── public/
│       └── i18n/            Traducciones: es.json (fuente), en, pt-BR, pt-PT
│
├── src-tauri/               Backend Rust + configuración Tauri
│   ├── Cargo.toml           Dependencias Rust
│   ├── tauri.conf.json      Config de la app (nombre, versión, ventanas, bundle)
│   ├── capabilities/        Permisos del webview (default.json sin BOM)
│   ├── icons/               Iconos del instalador
│   └── src/                 ~65 módulos Rust (cada uno <200 líneas)
│
├── Documentación/           Documentación interna del proyecto
│   ├── REGLAS_PROYECTO.md   Las 10 reglas inmutables (lectura obligatoria)
│   └── COMPILACION_Y_VERSIONES.md  Proceso de release y notas de antivirus
│
├── .github/workflows/       CI: build.yml (dev), release-builds.yml (tags v*)
├── CLAUDE.md                Guía completa para IAs colaboradoras
├── COMPILAR.md              Instrucciones detalladas de compilación
├── MANUAL.md                Manual del usuario final
└── README.md                Presentación pública del proyecto (GitHub)
```

---

## Separación de responsabilidades

### Lo que hace el frontend

- Renderizar la rejilla de botones, pestañas y perfiles con datos que vienen de Rust.
- Capturar clics, drag & drop y teclado; llamar al IPC Rust correspondiente.
- Suscribirse a eventos Rust (`audio-tick`, `clock-tick`, `weather-updated`) y actualizar la pantalla.
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
4. audio_thread recibe el comando:
   a. Decide bus de destino (main o pre)
   b. build_play_source: cache hit → O(1) seek; cache miss → decode + skip O(n)
   c. MasterBus::add_source → ButtonSource en el DynamicMixer
5. audio_monitor (hilo) detecta el nuevo ButtonState → emite "audio-tick" cada 100ms
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
    ├── Comprueba track_analysis_cache (evita re-analizar si mtime/size no cambió)
    ├── audio_analysis::analyze(path):
    │     ├── Decodifica PCM completo (symphonia)
    │     ├── Mide LUFS integrado (ebur128)
    │     ├── Mide pico dBFS
    │     ├── Calcula ganancia sugerida (objetivo −14 LUFS, techo −1 dBFS)
    │     └── Construye WaveEnvelope (min/max por bucket, hasta 120k puntos)
    ├── Upsert en tracks.db (preserva cue/dB del usuario si ya había fila)
    ├── Guarda PCM en PreloadCache (para seek O(1) instantáneo en la previa)
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
     ├── analyze_track() → siempre cachea el PCM del archivo editado
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

El LFA usa nombres de campo distintos en JSON (`file`, `bg`, `text`, `loop`, `stopOther`). La conversión vive en `src-tauri/src/lfa_format/`:

```
Botonera ──► to_lfa_paleta() ──► LfaPaleta (JSON .bdelf compatible con LFA)
LFA .bdelf ──► from_lfa_paleta() ──► PaletaData (campo desconocido ignorado por serde)
```

**Regla de compatibilidad:** todo campo nuevo en el formato Botonera debe tener `#[serde(default)]` para que el LFA pueda deserializar la estructura ignorando el campo. Nunca añadir campos obligatorios a los tipos LFA.

El campo `bdelf_tracks` (cue/dB por archivo) es opcional y el LFA lo ignora. Al importar, `export_tracks.rs::restore()` reescribe los metadatos en `tracks.db` adaptados al sistema de archivos local.

---

## Cómo añadir una nueva función

### Nuevo comando IPC

1. Crear o elegir el `cmd_*.rs` correspondiente.
2. Añadir la función con `#[tauri::command]`.
3. Registrarla en `lib.rs` dentro de `invoke_handler!(tauri::generate_handler![...])`.
4. Añadir el wrapper en `api.js` (o simplemente llamar `invoke('nombre_comando', args)`).
5. Si la función lee o modifica `AppConfig`, usar `state.config.lock().unwrap()` y llamar `config::save_config(&cfg)`.

### Nuevo tipo de botón

1. Añadir la variante en `button_types.rs`.
2. Añadir el caso en `cmd_button_playback::play_button_id()`.
3. Añadir el renderizado en `editTypes.js`.
4. Añadir la clave i18n en `es.json` y los 3 idiomas restantes.

### Nueva clave de configuración

1. Añadir el campo con `#[serde(default)]` en el struct de `types.rs` (u otro `types_*.rs`).
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
# Tests unitarios de Rust (39 tests en v1.1.2)
cd src-tauri
cargo test --lib

# Build completo del frontend
cd ..
npm run build

# Límite de 200 líneas por archivo Rust
wc -l src-tauri/src/*.rs src-tauri/src/**/*.rs
```

Los tests cubren:
- `db.rs`: migración, idempotencia, `normalize_key`
- `types_track.rs`: `sanitized_cue`, `effective_duration_s`, casos extremos
- `audio_monitor.rs`: `compute_display_time` con múltiples instancias
- `cached_source.rs`, `cue_source.rs`: seek y bucle
- `preload_cache.rs`: LRU, presupuesto RAM
- `cmd_preload.rs`: validación de parámetros de precarga
- `track_store.rs`: upsert preservando ediciones, `recent_paths`
- `last_played.rs`: debounce

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
