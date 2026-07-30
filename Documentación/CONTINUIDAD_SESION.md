# Continuidad de sesión — estado actual

**Actualizado:** 2026-07-30

**Rama de trabajo:** `codex/buscador-interno`

**Versión del código:** 1.3.0

Este archivo es temporal. Solo conserva el punto real de reanudación, la evidencia
reciente y las advertencias que evitan repetir errores. Las decisiones permanentes
pertenecen a `ARCHITECTURE.md`, `LIBRO_PROYECTO.md`, `GLOSARIO.md` y a los planes
temáticos aprobados.

## 1. Estado actual

La etapa de Biblioteca y buscador interno está implementada sobre un único catálogo
en `tracks.db`. Incluye:

- buscador como tercera vista del panel fijo;
- ventana Biblioteca independiente;
- múltiples raíces de Música y Efectos, con solapamientos normalizados;
- indexación incremental, observador de archivos y reconciliación al iniciar;
- árbol local, navegación por unidades y lista virtual;
- LIVE, CUE, envío al reproductor, arrastre y menú contextual compartidos;
- retiro reversible de raíces, retención configurable entre 30 y 365 días y purga
  que protege cualquier pista todavía usada en rejillas, botones fijos o reproductor;
- respaldo y restauración completos mediante un único archivo `.lfbackup`;
- editor de metadatos propios y tags para Música y Efectos;
- renombrado físico opcional y escritura opcional de metadatos dentro de una pista
  individual de Música, ambos con recuperación ante fallos.

El esquema vigente de `tracks.db` es el **6**. No se debe crear otra base, otro
catálogo ni otro buscador para estas funciones.

La publicación y actualización de la versión 1.3.0 ya no son trabajo activo. GitHub
Release `v1.3.0` está publicado desde el 2026-07-28. Los documentos de distribución y
las notas de publicación se conservan como registro histórico.

## 2. Último bloque terminado

El editor de metadatos y tags quedó integrado en Biblioteca y Buscador fijo:

- Música: título, artista, álbum, artista del álbum, género, año, número de pista,
  compositor, comentario y tags;
- Efectos: nombre descriptivo, categoría, descripción y tags;
- selección múltiple con campos explícitamente habilitados y selección mixta limitada
  a tags;
- búsqueda y recorrido usan los metadatos efectivos y los tags propios;
- los valores propios prevalecen sobre las etiquetas leídas del archivo sin destruir
  el original;
- el renombrado físico nunca es automático y actualiza las referencias propias;
- la escritura dentro del archivo es una acción independiente, explícita y solo para
  una pista de Música.

También se corrigieron dos detalles de interfaz:

- los campos del editor respetan el tema oscuro y ya no aparecen blancos;
- el Centro de procesamiento usa siempre `Ocultar ventana`; el botón, la X y Escape
  ocultan sin detener el proceso y conservan el borrador de la sesión en memoria.

Los diseños, reglas y pruebas de esta etapa están en:

- `Documentación/PLAN_BUSCADOR_INTERNO.md`;
- `Documentación/PLAN_EDITOR_METADATOS.md`;
- `Documentación/PLAN_RESPALDO_RESTAURACION.md`.

## 3. Evidencia técnica reciente

Verificación completada el 2026-07-30:

- `cargo test --lib`: **298 aprobadas, 0 fallidas, 19 ignoradas**;
- `cargo build --lib`: correcto;
- `npm run build`: correcto;
- `npm run tauri build -- --no-bundle`: correcto;
- ejecutable Release:
  `src-tauri/target/release/tauri-app.exe`, 23.567.872 bytes;
- i18n: **646 claves idénticas** en español, inglés, portugués de Brasil y portugués
  de Portugal;
- `git diff --check`: sin errores; solo avisos de finales de línea CRLF.

Pruebas de datos:

- metadatos probados sobre una copia desechable de una base real de 38.576.128 bytes;
  el archivo original no se modificó;
- búsqueda y recorrido probados con catálogos sintéticos de 100.000 y 250.000 pistas;
- retiro, restauración y vencimiento probados sobre una copia real con 20.411 pistas;
- respaldo, restauración y recuperación ante fallos tienen pruebas automatizadas.

La prueba visual y de uso real sigue correspondiendo al autor. Para audio o interacción
física se usa siempre una compilación Release.

## 4. Decisiones que siguen vigentes

- `botonera_config.json` conserva perfiles, paletas, botones y ajustes.
- `tracks.db` conserva datos técnicos, catálogo, metadatos propios y tags.
- Retirar una raíz nunca borra archivos de audio.
- Una pista usada por una rejilla, botón fijo o cola del reproductor protege sus datos
  técnicos frente a la purga.
- El editor de metadatos no altera cue, ganancia, normalización ni análisis.
- Los tags propios se guardan en LF Botonera; no se escriben dentro del audio salvo
  que el usuario active expresamente esa opción permitida.
- Renombrar un archivo físico es individual, opcional y nunca automático.
- Toda lógica crítica, validación y persistencia vive en Rust. JavaScript presenta
  estado y envía órdenes.
- Todo texto visible debe existir en los cuatro idiomas.
- Enter y doble clic en Biblioteca siguen sin acción; el menú contextual es la puerta
  a las acciones de pista.

## 5. Próximo trabajo: emojis en los botones

El diseño funcional ya está recogido en
[`PLAN_EMOJIS_BOTONES.md`](PLAN_EMOJIS_BOTONES.md). **Todavía no hay código iniciado
para esta función.**

Decisiones cerradas:

- `Emojis` será el catálogo principal, colorido y completamente offline;
- `Básicos` será una segunda colección local de 300 a 500 Material Symbols
  seleccionados y traducidos;
- la búsqueda usará nombres y palabras clave locales en los cuatro idiomas; para
  emojis partirá de Unicode CLDR;
- un botón admite un solo visual: emoji o icono básico, con texto visible opcional;
- el visual se guarda separado de `name` y `label`; el nombre textual se conserva
  aunque el modo visible sea solo dibujo;
- el visual será decorativo para tecnologías de asistencia y no se leerá como parte
  redundante del nombre del botón;
- los botones antiguos conservan su icono automático de tipo; un visual elegido lo
  sustituye, y quitarlo restaura el comportamiento actual;
- no habrá red en selección, búsqueda ni renderizado;
- imágenes personales o arbitrarias quedan fuera de la primera versión.

Antes de programar hay que medir y aprobar el empaquetado exacto de Noto Emoji,
cerrar el modelo opcional de `ButtonData`, definir favoritos/recientes y coordinar la
portabilidad por `.bdelf`, `.bdeplf` y `.LFPlay` con LF Automatizador. Todo campo
nuevo llevará `#[serde(default)]`.

## 6. Pendientes conocidos no bloqueantes

- Probar físicamente en Linux los paquetes `.deb` y `.AppImage`.
- Corregir en una etapa futura la representación extensa de `master_volume` y
  `ButtonData.vol` por usar `f32` en JSON.

## 7. Orden de lectura al reanudar

1. `AGENTS.md`.
2. Este archivo.
3. `Documentación/PLAN_EMOJIS_BOTONES.md`.
4. `Documentación/ARCHITECTURE.md` y `Documentación/LIBRO_PROYECTO.md` solo para las
   áreas afectadas.

No hay que reabrir como pendientes la publicación 1.3.0, la Biblioteca, el retiro
seguro, el respaldo ni el editor de metadatos: esas etapas ya están cerradas.
