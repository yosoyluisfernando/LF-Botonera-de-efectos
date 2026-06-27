# PLAN — Precarga de Audio (precisión de disparo)

> Plan de trabajo aprobable ANTES de escribir código (igual que PLAN_FASE7 y
> PLAN_EDITOR_PISTAS). Objetivo: que pulsar un botón dispare el sonido SIN el
> jitter de abrir y decodificar el archivo en el momento, manteniendo a Rust
> como dueño de la lógica (Regla 4). Todo lo decide el usuario: nada se impone.

---

## 1. Hallazgos del estudio del código actual

| Hecho | Implicación |
|---|---|
| Al pulsar, el archivo se **abre y decodifica EN EL MOMENTO**, dentro del hilo de audio ([audio_thread.rs:104](../src-tauri/src/audio_thread.rs) → `cue_source::cued_source` → `audio_decode::source_from_path`) | Ese I/O + parseo en caliente es el **principal jitter** de disparo. La precarga lo elimina sirviendo PCM ya decodificado desde RAM |
| El cue (salto) y la ganancia se aplican **encima** de cualquier `Source` ([cue_source.rs](../src-tauri/src/cue_source.rs), [master_button.rs](../src-tauri/src/master_button.rs)) | La caché guarda el **PCM del archivo completo** (por archivo); cue/ganancia siguen aplicándose por botón al reproducir. No hay que cachear por botón |
| Ya existe la columna `last_played` en `tracks.db` ([db.rs](../src-tauri/src/db.rs), añadida en E.b) | El "historial de última reproducción" para la expulsión por tiempo **ya tiene dónde vivir**, sin archivo nuevo |
| `AppConfig` centraliza la configuración global ([types.rs](../src-tauri/src/types.rs)) | `PreloadConfig` vive ahí (ajuste GLOBAL, no por perfil) |
| El hilo de audio NO debe bloquearse | Decodificar para precargar va SIEMPRE en un hilo aparte (preloader), nunca en `audio_thread` |

## 2. Decisiones ya cerradas con el usuario

1. **La precarga es OPCIONAL y configurable por el usuario.** Se pregunta una vez
   al primer arranque tras la actualización y queda re-editable en Ajustes → General.
2. **Preguntas exactas** (del usuario):
   - ¿Desea usar la precarga? (sí/no)
   - ¿Cuánta RAM? **32 / 64 / 128 / 256 MB**
   - ¿Precargar archivos menores a **5 / 10 / 15 / 30 s**?
   - Estrategia: **perfil completo** / **pestañas visibles + las que se abran al
     hacer clic** / **a medida que se vayan reproduciendo**.
   - (Solo "a medida que se reproduzcan") Borrar de la precarga tras **N horas /
     días** desde la última reproducción.
3. **PCM en `i16`** (la mitad de RAM que `f32`).
4. **El historial NO usa archivos nuevos**: reutiliza `last_played` con escritura
   agrupada (debounce) en la misma SQLite (modo WAL).
5. **Es una fase posterior al editor de pistas** (ya entregado). Este documento es
   su plan completo.

## 3. Arquitectura (decisión)

```
  ┌──────────────── Ajustes / Primer arranque ────────────────┐
  │ PreloadConfig (enabled, RAM, umbral, estrategia, TTL)      │ ──► guardado en AppConfig
  └───────────────────────────┬───────────────────────────────┘
                              │ gobierna
        ┌─────────────────────▼─────────────────────┐
        │  PRELOADER (hilo dedicado + cola mpsc)     │  decodifica → i16 → inserta
        │  enqueue según estrategia (perfil/tab/play)│
        └─────────────────────┬─────────────────────┘
                              │ llena (con tope de RAM)
                    ┌─────────▼──────────┐
                    │  PreloadCache (RAM)│  HashMap<clave, Arc<[i16]>> + LRU + presupuesto
                    └─────────┬──────────┘
                              │ consulta al disparar
  PULSAR ─► audio_thread::play_file ─► ¿cache hit?
        sí → CachedSource(Arc<[i16]>)  (instantáneo, sin I/O)
        no → audio_decode::source_from_path (perezoso, como hoy)
                              └─► + cue + ganancia (igual que hoy) ─► MasterBus
```

- **Clave de caché**: la misma ruta normalizada que usa el editor
  (`db::normalize_key`), para compartir criterio entre módulos.
- **No destructivo y transparente**: si la precarga está desactivada o el archivo
  no está cacheado, el disparo funciona EXACTAMENTE como hoy.

## 4. Configuración: `PreloadConfig` (en `AppConfig`, global)

```rust
struct PreloadConfig {
    enabled: bool,                 // ¿usar precarga?
    ram_budget_mb: u32,            // 32 | 64 | 128 | 256
    max_duration_s: u32,           // precargar solo < 5 | 10 | 15 | 30 s
    strategy: PreloadStrategy,     // FullProfile | VisibleTabs | OnPlay
    evict_after_hours: u32,        // TTL para OnPlay (en horas; "días" = ×24 en la UI)
    prompted: bool,                // ¿ya se preguntó tras actualizar?
}
enum PreloadStrategy { FullProfile, VisibleTabs, OnPlay }
```

Valores por defecto (si el usuario no decide): `enabled=false` (no sorprender),
y al activarlo: `128 MB`, `< 10 s`, `OnPlay`, `evict_after_hours=72` (3 días).

## 5. Diálogo de primer arranque + panel de Ajustes

- Al arrancar, si `preload.prompted == false`, se muestra **una vez** un modal con
  las 5 preguntas del §2.2; al guardar, `prompted = true`.
- El mismo formulario vive en **Ajustes → General** (re-editable siempre).
- Solo UI de configuración (es un "control remoto", Regla 4); textos en `es.json`
  (Regla 6); tema dinámico sin parpadeo (Regla 7).

## 6. Caché en RAM (`PreloadCache`)

- Estructura: `HashMap<String, Arc<[i16]>>` + cola **LRU** + `bytes_actuales` +
  `presupuesto`. PCM intercalado en `i16`, con su `sample_rate` y `channels`.
- **Coste**: estéreo 48 kHz en `i16` ≈ **11,5 MB/min** → con 128 MB caben ~11 min
  de audio, o **~110 efectos de 6 s**.
- **Inserción**: si supera el presupuesto, expulsa el menos usado (LRU) hasta caber.
- **Solo entra** audio con `duración < max_duration_s`. Los largos siguen en
  decodificación perezosa (como hoy).
- **`CachedSource`**: fuente que envuelve `Arc<[i16]>` + índice e implementa
  `Source<Item=f32>` (convierte `i16→f32` al vuelo). Comparte el PCM sin clonarlo
  en cada disparo. Sobre ella se aplican cue y ganancia igual que hoy.

## 7. Estrategias de llenado (hilo preloader)

Un hilo dedicado con cola `mpsc<PathBuf>`; el motor le encola rutas, él decodifica
y llena la caché. **Nunca** decodifica en el hilo de audio.

- **FullProfile**: al cargar/cambiar de perfil, encola todos los audios del perfil
  (bajo umbral) hasta agotar presupuesto.
- **VisibleTabs**: encola los audios de la pestaña activa; al abrir otra pestaña,
  encola los suyos (caché caliente de lo que el usuario ve).
- **OnPlay**: al pulsar un botón cuyo archivo no está cacheado y cumple umbral, se
  encola tras dispararlo. La primera pulsación es "fría" (disco); las siguientes,
  instantáneas. Los archivos del **editor** quedan calientes sin coste extra.

## 8. Historial de "última reproducción" SIN sobrecargar (clave del usuario)

- **No se crea ningún archivo nuevo.** En sesión, el **orden LRU en RAM** ya es el
  "qué se usó hace poco" (expulsar = reordenar punteros, gratis).
- Para la regla "borrar tras N horas/días" **entre sesiones** se reutiliza la
  columna **`last_played`** de `tracks.db`:
  - Al reproducir, se actualiza `last_played` **en memoria al instante** y se
    vuelca a SQLite **con debounce** (un `UPDATE` agrupado cada ~30 s o al cerrar,
    en modo WAL = barato). Nunca un escribe-por-pulsación que sature el disco.
  - Con estrategia `OnPlay`, al arrancar se pueden **recalentar** solo los archivos
    cuyo `last_played` esté dentro de la ventana `evict_after_hours`; lo más viejo
    ni se toca.

## 9. Integración con el motor (mínima y reversible)

- `audio_thread::play_file`: antes de `cue_source::cued_source`, consulta
  `PreloadCache`. Si hay PCM, construye `CachedSource`; si no, decodifica perezoso.
  Cue y ganancia se aplican igual sobre cualquiera de las dos fuentes.
- `cue_source`: acepta una fuente base ya construida (cacheada) o una ruta a
  decodificar — un único punto de entrada para no duplicar la lógica de cue.
- El `AppState` gana `preload: Mutex<PreloadCache>` y el handle del preloader.

## 10. IPC nuevo (`cmd_preload.rs`)

| Comando | Efecto |
|---|---|
| `get_preload_config` | Devuelve `PreloadConfig` (para el diálogo y Ajustes) |
| `set_preload_config` | Guarda la config; reconfigura caché/preloader en caliente |
| `mark_preload_prompted` | Marca `prompted=true` tras el primer diálogo |
| `get_preload_stats` *(opcional)* | MB usados / nº de archivos en caché (diagnóstico) |

## 11. Plan de trabajo por etapas (cada una compilable y probable)

| Etapa | Entregable | Módulos (límite 150–200 líneas) |
|---|---|---|
| **P.a Config + UI** | `PreloadConfig` en AppConfig + diálogo 1er arranque + panel Ajustes (solo guarda preferencias; sin cachear aún) | `types.rs` (edición), `cmd_preload.rs`, `preloadDialog.js`, `settingsPreload.js` |
| **P.b Caché + disparo** | `PreloadCache` + `CachedSource` + consulta en `audio_thread`; relleno manual para probar | `preload_cache.rs`, `audio_thread.rs`/`cue_source.rs` (ediciones) |
| **P.c Preloader + OnPlay** | Hilo decodificador + cola + estrategia OnPlay + `last_played` con debounce | `preloader.rs`, `track_store.rs` (debounce) |
| **P.d Estrategias** | FullProfile + VisibleTabs (encolado al cargar perfil / cambiar pestaña) | `preloader.rs` (ampliación), enganches en perfiles/pestañas |
| **P.e TTL + pruebas** | Expulsión por `evict_after_hours`, recalentado al arranque, medición de jitter, Windows 10/11 + Linux | matriz de pruebas |

**Dependencias nuevas**: ninguna (rodio/symphonia ya decodifican; `Arc<[i16]>` es
estándar). Multiplataforma sin requisitos extra.

## 12. Lo que NO haremos (anti-patrones descartados)

- ❌ **Decodificar para precargar en el hilo de audio** (va en el preloader).
- ❌ **Cachear en `f32`** pudiendo usar `i16` (doble de RAM).
- ❌ **Archivo de historial propio**: se reutiliza `last_played` + debounce.
- ❌ **Precargar todo al arrancar sin tope** (siempre presupuesto + LRU).
- ❌ **"Borrar de precargas" = borrar del disco**: solo se suelta de la RAM.
- ❌ **Imponer la precarga**: desactivada por defecto; la decide el usuario.

## 13. Multiplataforma (Windows 10/11 Home/Pro/LTSC + Linux deb/appimage/rpm)

- Solo RAM + hilos estándar de Rust; sin FFI ni dependencias de sistema nuevas.
- `last_played`/debounce sobre la SQLite que ya es multiplataforma.
- El presupuesto en MB es independiente del SO.

## 14. Consideraciones abiertas / a confirmar

1. **Recalentado al arranque** (estrategia OnPlay): ¿precargar los recientes dentro
   del TTL al abrir, o esperar a la primera pulsación de cada uno? (Propongo
   recalentar los recientes, mejor experiencia.)
2. **Indicador en la UI** de cuánta RAM se está usando (panel Ajustes): ¿lo quieres
   visible o lo dejamos interno?
3. **Loop con precarga**: un botón en bucle sobre PCM cacheado repite el `Arc<[i16]>`
   sin volver a decodificar (gratis). Confirmar que es el comportamiento deseado.

---
**Estado:** PENDIENTE DE APROBACIÓN. No se escribe código de precarga hasta que
confirmes: el esquema `PreloadConfig` (§4), la estrategia de caché en `i16` + LRU
(§6), el uso de `last_played` con debounce (§8) y el orden de etapas (§11).
