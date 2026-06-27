# PLAN — Editor de Pistas Avanzado (Manual Cue + Normalizador + Onda)

> Plan de trabajo aprobable ANTES de escribir código (igual que PLAN_FASE7).
> Objetivo: un editor por archivo donde el usuario VE la forma de onda, fija a
> mano el punto de inicio (manual cue) para saltar silencios/colas, y normaliza
> el volumen (automático a un objetivo + ajuste fino en dB). Todo el DSP en Rust
> (Regla 4); la UI solo pinta la onda y arrastra el marcador, y se construye AL
> FINAL. La precarga de audio es una **fase futura** ya diseñada en el §12.

---

## 1. Hallazgos del estudio del código actual

| Hecho | Implicación |
|---|---|
| El motor ya es un **Master Bus** con `DynamicMixer`; cada sonido es un `ButtonSource` que multiplica `s × vol_local × master` ([master_button.rs:70](../src-tauri/src/master_button.rs)) | Punto natural para inyectar la **ganancia del archivo** (dB) y el **cue** sin tocar la arquitectura |
| El archivo se **abre y decodifica EN EL MOMENTO de pulsar**, dentro del hilo de audio ([audio_thread.rs:104](../src-tauri/src/audio_thread.rs)) | El cue (saltar al inicio) se aplica aquí, al construir la fuente. La pre-carga para precisión es la **fase futura** del §12 |
| `vol` es un **multiplicador lineal** por botón en el JSON ([types.rs:21](../src-tauri/src/types.rs)) | No hay dB ni cue. Terreno virgen, no se duplica nada. El `vol` por botón se conserva como **trim en vivo** (compatibilidad `.bdelf`, Regla 5) |
| La duración se sondea con `lofty` sin decodificar ([cmd_audio.rs:70](../src-tauri/src/cmd_audio.rs)) | Para onda + LUFS hay que **decodificar el PCM completo una vez** (operación nueva, en hilo de comando, NUNCA en el hilo de audio) |
| `symphonia` ya es dependencia y decodifica MP3/WAV/OGG/FLAC/M4A/Opus | Sirve para el análisis y para `seek`; no añadimos decodificador nuevo |
| `targets: "all"` ya genera `.msi`/`.exe` y `.deb`/`.appimage`/`.rpm`; la CI instala `libasound2-dev` ([build.yml](../.github/workflows/build.yml)) | Las dependencias nuevas deben respetar este empaquetado y no añadir requisitos de sistema |
| No existe SQLite, ni cue, ni onda, ni análisis de loudness | Funcionalidad 100% nueva |

## 2. Decisiones ya cerradas con el usuario

1. **Almacenamiento: SQLite (`rusqlite`, feature `bundled`)** — compila su propio
   SQLite estático, no instala nada en el equipo. Una fila **por archivo**.
2. **La forma de onda NO se persiste** — se calcula al vuelo al abrir el editor
   y se descarta. Solo viven en la DB los datos pequeños (cue, dB, LUFS, etc.).
3. **Cue y dB son POR ARCHIVO (compartido)** — editas un audio una vez y se
   aplica a todos los botones que lo usan. Clave = ruta normalizada + `mtime`.
4. **Normalizador = automático (objetivo LUFS/pico) + ajuste manual en dB.**
5. **La UI va SIEMPRE al final** (etapa E.e); E.f son solo pruebas.
6. **Precarga / precisión de disparo = FASE FUTURA** (§12). No se implementa en
   este ciclo, pero su diseño y su esquema de datos quedan preparados aquí para
   no rehacer la DB después.

## 3. Compatibilidad multiplataforma (objetivo de soporte)

Destinos a cubrir: **Windows 10 y 11 (Home / Pro / LTSC)** y **Linux (`.deb`,
`.AppImage`, `.rpm`)**.

| Pieza nueva | Windows 10/11 Home/Pro/LTSC | Linux deb/appimage/rpm |
|---|---|---|
| `rusqlite` (`bundled`) | Compila SQLite con el toolchain MSVC que ya usa el build. Sin DLL externa, sin dependencia del sistema | Compila con gcc (ya presente en CI). **No** añade dependencia a los paquetes `.deb`/`.rpm` |
| `ebur128` | Pura Rust, sin FFI | Pura Rust, sin FFI |
| Audio (cue/ganancia) | Sigue por rodio/cpal (WASAPI) ya en uso | rodio/cpal (ALSA) — `libasound2-dev` ya está en la CI |
| DB en disco | `%APPDATA%\LF Botonera\tracks.db` | `~/.config/LF Botonera/tracks.db` (vía API de rutas de Tauri, no hardcodear) |

Reglas de implementación para no romper portabilidad:
- **Rutas siempre vía la API de directorios de Tauri**, nunca separadores ni
  `%APPDATA%` a mano.
- **Normalización de la clave de archivo** sensible al SO: en Windows la ruta se
  compara en minúsculas (FS no sensible a mayúsculas); en Linux se respeta tal
  cual. Encapsular en una sola función para no esparcir la diferencia.
- **LTSC**: sin WebView2 preinstalado en algunas imágenes → ya se cubre con el
  instalador actual; el editor no añade requisitos nuevos de runtime.
- Ningún `cfg!(windows)` disperso: las (pocas) diferencias de SO viven en un
  único módulo auxiliar.

## 4. Arquitectura (decisión)

```
                    ┌────────────────── EDITOR (al abrirlo) ──────────────────┐
   archivo.mp3 ──►  │  audio_analysis.rs  →  pico(dBFS) + LUFS  (ebur128)     │
                    │  waveform.rs        →  N buckets min/max  (al vuelo)     │ ──► JS pinta canvas
                    └─────────────────────────┬───────────────────────────────┘
                                              │ upsert (sin la onda)
                                    ┌─────────▼──────────┐
                                    │  tracks.db (SQLite)│  ← cue_start/end, gain_db,
                                    │  1 fila por archivo│    norm_gain_db, lufs, peak…
                                    └─────────┬──────────┘
                                              │ lee al reproducir
   PULSAR BOTÓN ─► play_button_id ─► play_file ─► ButtonSource:
        s × ganancia_archivo(dB→lin) × vol_botón(trim) × master      (+ cue al construir la fuente)
```

**Modelo de ganancia en 3 capas** (no destructivo, todo en vivo):

```
total_dB   = (norm_enabled ? norm_gain_db : 0) + gain_db_manual
file_gain  = 10^(total_dB / 20)
muestra    = s × file_gain × vol_botón × master_volume
```

- `file_gain` (del archivo, de la DB) = nivel base normalizado.
- `vol_botón` = retoque rápido en vivo que ya existe.
- `master_volume` = global.

**El archivo original NUNCA se reescribe.** Cue y ganancia se aplican en la
cadena de reproducción.

## 5. Esquema SQLite (`<config>/LF Botonera/tracks.db`)

`PRAGMA user_version` controla migraciones (patrón sencillo, sin ORM). Modo
**WAL** para que las escrituras frecuentes (p.ej. `last_played`, §12) sean baratas.

```sql
CREATE TABLE track (
  path            TEXT PRIMARY KEY,   -- ruta absoluta normalizada (minúsculas en Win)
  mtime           INTEGER NOT NULL,   -- fecha modif. del archivo → invalida caché si cambia
  size            INTEGER NOT NULL,   -- verificación secundaria
  duration_s      REAL    NOT NULL,
  sample_rate     INTEGER NOT NULL,
  channels        INTEGER NOT NULL,
  cue_start_s     REAL    NOT NULL DEFAULT 0,
  cue_end_s       REAL,               -- NULL = hasta el final
  gain_db         REAL    NOT NULL DEFAULT 0,   -- trim manual del usuario
  norm_enabled    INTEGER NOT NULL DEFAULT 0,   -- bool
  norm_gain_db    REAL    NOT NULL DEFAULT 0,    -- ganancia calculada por el auto-normalizador
  measured_peak_db REAL,             -- pico real (dBFS) medido
  measured_lufs    REAL,             -- loudness integrado medido (LUFS)
  analyzed_at     INTEGER,           -- epoch del último análisis
  last_played     INTEGER            -- epoch de la última reproducción (lo usa la precarga §12)
);
```

- Si al reproducir `mtime`/`size` no coinciden con el archivo en disco → la fila
  se considera **obsoleta** (se ignora cue/dB hasta re-analizar). Nunca usamos
  datos de un archivo que el usuario ha reemplazado.
- **Duración efectiva** para la cuenta atrás = `(cue_end_s | duration_s) − cue_start_s`.
- `last_played` se incluye **desde ya** aunque la precarga sea futura, para no
  migrar el esquema otra vez (§12 explica cómo se actualiza sin sobrecargar).

## 6. DSP en Rust (el corazón, lo profesional)

### 6.1 Manual cue (punto de inicio / fin)
- **Inicio**: al construir la fuente en `audio_decode`, se salta `cue_start_s`.
  Para saltos pequeños, `Source::skip_duration`; para offsets grandes se evalúa
  `symphonia seek` (más eficiente que decodificar y descartar). Decisión final
  en la etapa E.d con medición.
- **Fin**: recorte con una fuente envolvente que corta tras `cue_end_s`.
- La **duración efectiva** se pasa al `ButtonState` para que la cuenta atrás de
  la rejilla sea honesta.

### 6.2 Normalizador (auto + manual)
- **Análisis** (una pasada de decodificación completa, en hilo de comando):
  - **Pico real** → máx. |muestra| → dBFS.
  - **Loudness integrado (LUFS)** con la crate **`ebur128`** (ITU-R BS.1770, pura
    Rust, ligera). Camino profesional en vez de un RMS casero impreciso.
- **Ganancia automática**: `norm_gain_db = objetivo_LUFS − LUFS_medido`,
  **limitada** para que el pico resultante no pase de un techo (`-1 dBFS` por
  defecto) y así nunca clippea.
  - Objetivo por defecto **−14 LUFS** (estándar de streaming), configurable.
- **Manual**: `gain_db` se suma encima (capa 1 del modelo de 3 capas).

### 6.3 Forma de onda (al vuelo, NO se guarda)
- `waveform.rs` decodifica y reduce a **N buckets** (≈ ancho del canvas, p.ej.
  1500), cada uno con (min, max). Devuelve el array a JS.
- Para no decodificar dos veces, el comando de análisis calcula **onda + pico +
  LUFS en la misma pasada** y persiste solo lo pequeño.

## 7. IPC nuevo (`cmd_tracks.rs`) — la UI es control remoto

| Comando | Entra | Devuelve / efecto |
|---|---|---|
| `get_track_meta` | `path` | fila de la DB (o `null`) — para abrir el editor con lo ya guardado |
| `analyze_track` | `path, buckets` | `{ waveform, duration_s, peak_db, lufs, suggested_norm_db }`; hace **upsert** de los datos medidos |
| `set_track_cue` | `path, start_s, end_s?` | persiste el cue |
| `set_track_gain` | `path, gain_db` | persiste el trim manual |
| `set_track_normalization` | `path, enabled, target_lufs?` | activa/desactiva auto-normalización |

La reproducción (`play_button_id` → `play_file`) **lee la fila de la DB** y aplica
cue + ganancia efectiva antes de mandar la fuente al bus.

## 8. UI (única parte en JS, justificada por Regla 4 y 8) — SE HACE AL FINAL

- **Contenedor: modal grande (~90vw × ~80vh)**, patrón `.modal-lg` ya existente.
  Decidido frente a ventana independiente por Regla 7 (cero parpadeo, comparte
  tema/i18n/titlebar) y por menor complejidad. **Preparado para "pop-out" futuro**:
  el contenido (onda + controles + lógica IPC) se monta dentro de un elemento
  `host` y NO asume modal; mañana ese host podría ser una ventana sin reescribir
  lógica. Esa opción de desacoplar queda como mejora futura, no se paga hoy.
- **Por qué JS aquí**: dibujar miles de barras de onda en un `<canvas>` y
  arrastrar el marcador de cue son operaciones de **pintado y eventos de
  puntero** — no es lógica de audio. Todo el cálculo (onda, cue, dB, LUFS,
  análisis, persistencia) vive en Rust. Se documentará explícitamente.
- **Módulos JS**: `trackEditor.js` (modal + estado), `waveformCanvas.js` (dibuja
  el array de buckets y el marcador, gestiona el arrastre). Cero `<canvas>` con
  lógica de negocio.
- **Entrada al editor**: botón "Editar pista" en `editModal.js` y en el menú
  contextual de un botón con audio.
- **Controles**: marcador de cue arrastrable sobre la onda, lectura de tiempo
  actual, slider/numérico de dB, botón "Normalizar" con lectura de LUFS/pico,
  interruptor auto-normalización. Todos los textos vía `es.json` (Regla 6).
- **Previa por PRE-ESCUCHA por defecto** (requisito del usuario): el botón de
  previa del editor reproduce con cue+ganancia aplicados a través del id de
  pre-escucha (`__prelisten__`), no por la salida de los botones. El motor ya
  está preparado (E.d: `play_audio` acepta `cue_start_s`/`cue_end_s`/`gain_db`).
  **Prerrequisito real**: la salida `out_pre` independiente (segundo
  `OutputStream`) sigue pendiente (ver "Lo que FALTA" en el contexto del
  proyecto); hoy la pre-escucha suena por la principal. Cuando ese bus exista,
  la previa del editor saldrá por él sin tocar el editor.

## 9. Plan de trabajo por etapas (cada una compilable y probable)

| Etapa | Entregable | Módulos (límite 150–200 líneas c/u) |
|---|---|---|
| **E.a Contrato congelado** | Este documento aprobado por ti | Documentación |
| **E.b Capa SQLite** | Conexión + migración (`user_version`, WAL) + CRUD por archivo + rutas multiplataforma + tests | `db.rs`, `track_store.rs`, `types_track.rs` |
| **E.c Análisis DSP** | Decodificar→pico+LUFS (`ebur128`) y onda en buckets; comando `analyze_track` | `audio_analysis.rs`, `waveform.rs`, `cmd_tracks.rs` |
| **E.d Cue + ganancia en reproducción** | Cue (inicio/fin) + modelo de 3 capas leyendo la DB; duración efectiva en la cuenta atrás | `audio_decode.rs`, `master_button.rs`, `cmd_button_playback.rs` (ediciones acotadas) |
| **E.e UI del editor** *(al final)* | Modal con onda, marcador arrastrable, dB, normalizar/LUFS; entrada desde editModal y menú contextual; `es.json` | `trackEditor.js`, `waveformCanvas.js`, `editModal.js` (edición) |
| **E.f Integración fina + pruebas** | Invalidación por `mtime`, formatos raros (Opus/WhatsApp), Windows 10/11/LTSC + Linux deb/appimage/rpm, límites, limpieza | matriz de pruebas |

**Dependencias Rust nuevas (justificadas, sin requisitos de sistema):**
- `rusqlite = { version = "0.32", features = ["bundled"] }` — SQLite estático,
  multiplataforma, sin instalación ni DLL.
- `ebur128` — medición de loudness LUFS estándar, pura Rust, ligera.

## 10. Lo que NO haremos (anti-patrones descartados)

- ❌ **Reescribir el archivo de audio** (destructivo). Cue/dB se aplican en vivo.
- ❌ **Guardar la onda en disco** (decidido: se calcula al vuelo).
- ❌ **Dibujar la onda con lógica de audio en JS**: JS solo pinta el array que
  Rust le entrega.
- ❌ **LUFS casero impreciso** si existe una crate probada (`ebur128`).
- ❌ **Bloquear el hilo de audio** con el análisis: decodificar el archivo
  completo va en el hilo del comando Tauri, jamás en `audio_thread`.
- ❌ **Meter cue/dB en el `config.json` gigante**: van en la DB, por archivo.
- ❌ **Rutas con separadores o `%APPDATA%` a mano**: siempre la API de Tauri.

## 11. Empezar la UI antes de tiempo (recordatorio)

La UI es lo último. E.b → E.d se prueban con tests y un comando de depuración
temporal; no se abre el editor visual hasta que el motor (DB + análisis + cue +
ganancia) esté sólido y compilando en los dos sistemas operativos.

---

## 12. FASE FUTURA — Precarga de audio (diseño preparado, no se implementa aún)

> Plan completo y aprobable en `Documentación/PLAN_PRECARGA.md`. Este §12 es el
> resumen; el detalle de implementación (módulos, etapas, IPC) está en ese doc.

Objetivo: reducir el jitter de disco en el disparo cargando PCM decodificado en
RAM. **Todo lo decide el usuario**, no se impone nada.

### 12.1 Configuración (vive en `AppConfig`, ajustes GENERALES, no por perfil)

```
PreloadConfig {
  enabled: bool,                 // ¿Desea usar la precarga?
  ram_budget_mb: u32,            // 32 | 64 | 128 | 256
  max_duration_s: u32,           // precargar solo archivos menores a: 5 | 10 | 15 | 30
  strategy: enum {               // qué se precarga
     FullProfile,                //  - perfil completo
     VisibleTabsOnDemand,        //  - pestañas visibles + las que se abran al hacer clic
     OnPlay,                     //  - a medida que se vayan reproduciendo
  },
  evict_after: { value: u32, unit: Hours | Days },  // solo para OnPlay
  prompted: bool,                // ¿ya se le preguntó tras actualizar?
}
```

### 12.2 Primer arranque tras la actualización
Si `prompted == false`, se muestra **una vez** un diálogo con exactamente estas
preguntas (y queda re-editable en Ajustes → General):
1. ¿Desea usar la precarga? (sí/no)
2. ¿Cuánta RAM desea usar? **32 / 64 / 128 / 256 MB**
3. ¿Precargar archivos menores a **5 / 10 / 15 / 30 s**?
4. Estrategia: **perfil completo** / **pestañas visibles + al hacer clic** / **a
   medida que se reproduzcan**.
5. (Solo "a medida que se reproduzcan") Borrar de la precarga al pasar **N horas
   / días** desde la última reproducción.

### 12.3 Caché en RAM
- PCM almacenado en **`i16`** (la mitad de RAM que `f32`; se convierte a `f32`
  al mezclar). Un estéreo 48 kHz ≈ **11,5 MB/min** en `i16`.
- Estructura: mapa `clave_archivo → Arc<[i16]>` + cola **LRU**. Al superar
  `ram_budget_mb` se descarta el menos usado recientemente.
- Solo entra audio con `duración < max_duration_s`. Los largos siguen en
  decodificación perezosa (como hoy).
- Los archivos que se abren en el **editor quedan calientes** sin coste extra.

### 12.4 El historial de "última reproducción" SIN sobrecargar nada
Esta es tu duda clave. Solución: **no creamos ningún archivo de historial nuevo**.
- Durante la sesión, el **orden LRU en memoria** ya es el "qué se usó hace poco";
  la expulsión por presupuesto es gratis (solo reordenar punteros en RAM).
- Para la regla "borrar al pasar N horas/días" **entre sesiones**, reutilizamos
  la **columna `last_played` de la DB que ya existe** (§5):
  - Al reproducir, se actualiza `last_played` **en memoria al instante** y se
    vuelca a SQLite **con debounce** (un único `UPDATE` cada ~30 s o al cerrar la
    app, en modo WAL = barato). Nunca un escribe-por-pulsación que sature el disco.
  - Al arrancar con estrategia `OnPlay`, se leen las filas cuyo `last_played`
    esté dentro de la ventana `evict_after` para decidir qué merece recalentarse;
    lo más viejo ni se toca.
- Resultado: cero archivos extra, cero formato propio, una sola columna entera
  por archivo y escrituras agrupadas. Profesional y sin sobrecarga.

### 12.5 Reglas de oro de esta fase futura
- Precargar y expulsar **NUNCA** ocurre en el hilo de audio (hilo aparte).
- "Borrar de precargas" = soltar de la RAM, **jamás** borra el archivo del disco.
- El módulo de caché será propio (`preload_cache.rs`) y respetará 150–200 líneas.

---

## 13. Consideraciones abiertas / a confirmar

1. **`.bdelf` no transporta cue/dB**: al ser datos **por archivo y locales a esta
   máquina** (DB), no viajan en la exportación. Coherente con "por archivo". Si
   se quisiera portabilidad sería una mejora aparte. **¿Lo aceptamos así?**
2. **Coexistencia `vol` (lineal) ↔ `gain_db` (archivo)**: propuesta = modelo de
   3 capas (§4). **¿De acuerdo, o el editor también edita el `vol` del botón en dB?**
3. **Objetivo de normalización por defecto**: −14 LUFS, techo de pico −1 dBFS,
   configurables. **¿Te sirve o prefieres otro (−16 / −23)?**

---
**Estado:** ✅ IMPLEMENTADO (v1, junio 2026). Etapas E.a–E.f completas y
verificadas (cargo test 32/32, frontend OK, prueba en vivo en la app). Contrato
y decisiones §4/§5/§9/§13 aprobados por el usuario.

**Pendiente como FASE APARTE (ya diseñado/anotado, no implementado):**
- Salida `out_pre` independiente (2º `OutputStream`) para que la previa salga
  realmente por pre-escucha (hoy id `__track_preview__` va por la principal).
- Precarga de audio (§12).
- Pop-out del editor a ventana (§8).
- Prueba física en Linux (.deb/.appimage/.rpm); el código es agnóstico y la CI
  ya empaqueta, pero falta validar en hardware Linux.
- `en.json` (la app aún solo tiene `es.json`).
