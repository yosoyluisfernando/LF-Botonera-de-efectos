# AGENTS.md — LF Botonera de Efectos

Guía para cualquier IA colaboradora (independientemente del modelo o plataforma).
Lee este archivo completo antes de proponer o escribir código.

> Si usas Claude Code, lee también [`CLAUDE.md`](CLAUDE.md) para instrucciones específicas de esa herramienta.
> Para una narrativa completa del proyecto, consulta [`Documentación/LIBRO_PROYECTO.md`](Documentación/LIBRO_PROYECTO.md).
> Para el glosario de términos, consulta [`Documentación/GLOSARIO.md`](Documentación/GLOSARIO.md).

---

## 1. Qué es este proyecto

**LF Botonera de Efectos** es una botonera de sonidos (*soundboard*) para radio y *streaming* en directo. Los operadores de radio asignan archivos de audio a botones en una rejilla, organizados en pestañas (paletas) dentro de perfiles, y los disparan en tiempo real durante transmisiones.

- **Versión:** 1.1.2
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

10. **Verificar sin lanzar la aplicación.** Verificar con `cargo test --lib` (backend) y `npm run build` (frontend). La prueba visual la hace el usuario en su equipo.

11. **Las IAs no son colaboradoras del proyecto.** Commits, PRs y cualquier contribución registrada van únicamente a nombre de usuarios humanos reales con cuenta de GitHub (ejemplo: "Juan Pérez", "Yo Soy Luis Fernando"). Sin trailers `Co-Authored-By: <nombre de IA>`, sin firmas de asistente, sin atribuciones de IA en mensajes de commit, descripciones de PR ni comentarios de código. El historial de git debe reflejar exclusivamente a personas reales. El reconocimiento al uso de herramientas de IA en el desarrollo está documentado en la sección "Créditos de desarrollo" de `README.md`.

---

## 3. Arquitectura en una página

```
┌───────────────────────────────────────────────────────┐
│  Frontend — Vanilla JS + Vite                         │
│  src/js/      (49 módulos, cada uno <200 líneas)      │
│  src/css/     (17 hojas de estilo por componente)     │
│  src/public/  (i18n en 4 idiomas)                     │
│                                                       │
│  Acceso a Rust SOLO a través de api.js:               │
│    invoke(cmd, args) → resultado                      │
│    listen(evento, fn) → suscripción                   │
└──────────────────────────┬────────────────────────────┘
                           │ IPC (window.__TAURI__)
┌──────────────────────────▼────────────────────────────┐
│  Backend — Rust + Tauri v2                            │
│  src-tauri/src/   (~65 módulos, cada uno <200 líneas) │
│                                                       │
│  AppState {                                           │
│    config:   Arc<Mutex<AppConfig>>      → JSON        │
│    audio:    Mutex<AudioEngine>         → rodio/cpal  │
│    tracks:   Arc<Mutex<TrackStore>>     → SQLite      │
│    waveforms, track_analysis, last_played, history    │
│  }                                                    │
└───────────────────────────────────────────────────────┘
```

El frontend **nunca** calcula duración de audio, nivel de señal, si un archivo es válido, cuánto tiempo lleva sonando, ni ningún otro estado crítico. Todo eso lo calcula Rust y lo comunica via IPC o eventos.

---

## 4. Modelo de datos principal

### Jerarquía de la configuración

```
AppConfig
  ├── theme, language, button_text_size, editor_mode
  ├── preload: PreloadConfig
  ├── locutions: LocutionConfig
  └── profiles: Vec<ProfileData>
        └── ProfileData
              ├── id, name, bg, text
              ├── audio: AudioConfig {out_main, out_pre, playback_mode, master_volume, ...}
              ├── active_paleta_id
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
                                                              audio_thread (hilo)
                                                                         │
                                                   preload_cache::build_play_source()
                                                         ├── Hit → CachedSource::new_at() O(1)
                                                         └── Miss → decode + CuedSource O(n)
                                                                         │
                                                              MasterBus::add_source()
                                                                         │
                                                   ButtonSource: s × file_gain × vol_btn × master
                                                                         │
                                                              DynamicMixer → LevelSource → Sink
                                                                                              │
                                                                                         Altavoces
```

**Modelo de ganancia (3 capas):**
```
señal = muestra × file_gain(dB→lineal) × vol_botón(lineal 0-1) × master(lineal 0-1.5)
```

**Pre-escucha:** los comandos con `to_pre=true` van al bus `device_pre` (segundo OutputStream). Si `device_pre` no está configurado, cae al bus principal.

---

## 6. Eventos Rust → Frontend

| Evento Tauri | Payload | Quién lo consume |
|---|---|---|
| `"audio-tick"` | `{buttons[], display_remaining, display_duration, master_level_l, master_level_r}` | gridPlayback.js, clockWidget.js, vuMeter.js, tabs.js |
| `"clock-tick"` | `{time_str, date_str}` | clockWidget.js |
| `"weather-updated"` | datos de clima | settingsLocutions.js |
| `"global-shortcut-refresh"` | — | startup.js → recarga la UI |
| `"track-editor-dock"` | `{path, name, zoom}` | startup.js → abre editor en modal |
| `"theme-changed"` | `{theme}` | ventana pop-out del editor |

`startup.js` también dispara el `CustomEvent('lf-audio-tick')` en el DOM cada vez que recibe `"audio-tick"` de Rust. Los módulos del editor usan `window.addEventListener('lf-audio-tick', ...)` porque es un `CustomEvent`, no un evento Tauri.

---

## 7. Comandos IPC (resumen)

Para la lista completa ver [`CLAUDE.md §9`](CLAUDE.md).

**Los más frecuentes:**
- `get_config` → `AppConfig` completo
- `get_grid_state` → rejilla de la paleta activa
- `play_button(id)` → disparo principal de un botón
- `play_audio(id, path, volume, ...)` → reproducción directa (pre-escucha, editor)
- `stop_audio(id)`, `stop_all_audio`
- `analyze_track(path)` → análisis DSP completo + envolvente de onda
- `set_track_cue(path, start, end?)` / `set_track_gain(path, dB)` / `set_track_normalization(path, enabled)`
- `update_button_data(paleta_id, index, data)` → guarda edición de botón
- `export_tab_by_id(paleta_id)` / `import_tab()` → formatos .bdelf
- `set_editor_mode(mode)` → "modal" | "window"; persiste en AppConfig

---

## 8. Mapa de archivos clave

Para el mapa completo, ver [`Documentación/LIBRO_PROYECTO.md §3 y §4`](Documentación/LIBRO_PROYECTO.md).

**Los más importantes para entender el sistema:**

| Archivo | Por qué es central |
|---|---|
| `src-tauri/src/lib.rs` | Define `AppState` y registra todos los comandos IPC |
| `src-tauri/src/app_setup.rs` | Inicializa la app: dispositivo, hilos, precarga |
| `src-tauri/src/types.rs` | Esquema de datos completo serializable |
| `src-tauri/src/config.rs` | Persistencia JSON + migración automática |
| `src-tauri/src/audio_thread.rs` | El único hilo que toca rodio/cpal |
| `src-tauri/src/master_bus.rs` | Mezcla de fuentes + medición de nivel |
| `src-tauri/src/cmd_button_playback.rs` | Lógica completa de disparo de un botón |
| `src-tauri/src/track_store.rs` | CRUD de metadatos de pista en SQLite |
| `src/js/api.js` | Única puerta de acceso al IPC desde el frontend |
| `src/js/startup.js` | Bootstrap completo de la UI |
| `src/js/trackEditor.js` | Orquestador del editor de pistas |

---

## 9. Cómo añadir una nueva función

### Nuevo comando IPC
1. Crear o elegir el `cmd_*.rs` correspondiente.
2. Añadir `#[tauri::command]` a la función.
3. Registrar en `lib.rs` dentro de `invoke_handler!(tauri::generate_handler![...])`.
4. Llamar desde el frontend con `invoke('nombre_comando', args)`.
5. Si modifica `AppConfig`, llamar `config::save_config(&cfg)` al final.

### Nuevo tipo de botón
1. Añadir el caso en `button_types.rs`.
2. Añadir la rama en `cmd_button_playback::play_button_id()`.
3. Añadir los campos de UI en `editTypes.js`.
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
El formato actual es `{paleta_id}_btn_{index}` (ejemplo: `paleta_1_btn_3`). El formato antiguo era `btn_{index}` y colisionaba entre paletas. `config.rs::normalize_button_ids()` migra automáticamente al cargar; no producir IDs en el formato viejo.

### El hilo de audio no hace I/O
El hilo de audio (`audio_thread.rs`) **no** accede al disco. La decodificación ocurre en `preload_cache::build_play_source()` que llama a `audio_decode::source_from_path()`. El análisis DSP ocurre en el hilo de un comando IPC, nunca en el hilo de audio.

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
# Tests unitarios Rust (39 tests en v1.1.2)
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

**Versión 1.1.2 — funcionalidades completas:**
- Perfiles, paletas, botones (tipos: audio, time, temperature, humidity, random_folder)
- Motor de audio: loop, overlap, restart, stop_other, pre-escucha independiente
- Modos de reproducción global: normal, loop, overlap, restart + solo mode
- Atajos de teclado locales y globales del SO
- Editor de pistas: cue, normalización dB/LUFS, onda, transporte, pop-out a ventana
- Precarga de audio RAM (LRU, estrategias FullProfile/VisibleTabs/OnPlay, TTL)
- Locuciones dinámicas de hora y clima (open-meteo)
- Export/import `.bdelf`/`.bdeplf` con portabilidad de cue/dB
- i18n en 4 idiomas (es, en, pt-BR, pt-PT)
- CI/CD con GitHub Actions

**Pendientes conocidos:**

**A — Prueba física en Linux**
El código es multiplataforma (rutas via `config::get_data_dir()`, SQLite bundled, rodio/ALSA). Falta probar el build (`.deb`, `.AppImage`) en una máquina Linux real.
