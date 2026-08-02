# Continuidad de sesión — estado actual

**Actualizado:** 2026-08-02

**Rama de trabajo:** `codex/midi-input`

**Versión del código:** 1.3.0

Este archivo no es un historial. Conserva el punto real de reanudación, las decisiones
que evitan repetir trabajo y la evidencia técnica reciente. Las decisiones
permanentes viven en los documentos de arquitectura y planes temáticos.

## 1. Trabajo activo

Las ramas locales `codex/buscador-interno` y `codex/midi-input` apuntan al mismo
commit base, `43e67b0`. La rama actual ya contiene todo el historial de Biblioteca y
Buscador; no hay commits que fusionar entre ambas. MIDI e identificadores visuales
son cambios locales posteriores todavía sin commit y sin publicación pública.

### Entrada MIDI para atajos

En la rama `codex/midi-input` está implementado el soporte MIDI para atajos en Windows
sobre WinMM. La función y los identificadores visuales forman parte del mismo estado
de desarrollo local y se documentan como funciones nuevas de la próxima versión.

Alcance implementado:

- Ajustes generales → Atajos permite activar MIDI y seleccionar entradas.
- La selección se aplica en caliente y el motor reconcilia dispositivos conectados y
  desconectados cada segundo.
- Botones, botones fijos, pestañas y acciones globales pueden tener un `MidiBinding`.
- La captura MIDI se reutiliza en Ajustes y en los modales directos de edición/mapeo.
- La espera de captura no bloquea el hilo de la ventana y se puede cancelar con
  Escape o con el ratón sin dejar una captura pendiente.
- Se admiten Note On con velocidad mayor que cero, Control Change con valor mayor que
  cero y Program Change. Note Off se ignora para evitar dobles disparos.
- Varios dispositivos pueden estar seleccionados a la vez. Dos dispositivos iguales
  se distinguen simultáneamente por índice de puerto WinMM; si Windows reordena dos
  unidades idénticas tras una desconexión, WinMM no garantiza identidad física
  persistente.
- WinMM vive en dos módulos compilados solo para Windows. Linux conserva la interfaz
  común con un backend vacío y no recibe la dependencia nativa de Windows.

Evidencia técnica de esta fase:

- i18n: 738 claves idénticas en `es`, `en`, `pt-BR` y `pt-PT`.
- `cargo build --lib --offline`: correcto.
- `cargo test --lib --offline`: 320 aprobadas, 0 fallidas y 19 ignoradas; incluye
  regresiones de cancelación inmediata y liberación después del timeout.
- `npm run build`: correcto.
- `$env:LF_DISTRIBUTION_CHANNEL='store'; npm run tauri build -- --no-bundle`:
  correcto.
- El ejecutable Release conjunto más reciente se identifica en la sección 5.

### Identificadores visuales

La ampliación de identificadores visuales está implementada y se encuentra en
verificación final. El diseño definitivo está en
[`PLAN_EMOJIS_BOTONES.md`](PLAN_EMOJIS_BOTONES.md).

Colecciones:

- `Emojis`: 3.953 valores de Unicode Emoji 17.0 y CLDR 48.2; 1.918 se muestran por
  defecto al ocultar 2.035 variantes con modificador de piel.
- `Básicos`: 8.388 monocromáticos: 24 Material Symbols, 4.231 Tabler Icons y 4.133
  Game Icons.

Todo funciona sin Internet. Los nombres y palabras clave existen en español, inglés,
portugués de Brasil y portugués de Portugal. Se generaron traducciones auxiliares
para 5.149 palabras y las correcciones manuales de vocabulario importante tienen
prioridad.

## 2. Arquitectura cerrada

`ButtonData.visual` contiene:

```text
ButtonVisual {
  kind: "auto" | "emoji" | "basic"
  value: identificador estable
  mode: "text" | "visual_text" | "visual"
}
```

- `auto + visual_text` es el valor predeterminado y se omite del JSON.
- Los archivos antiguos conservan su icono según el tipo.
- Los valores históricos de los 24 Material Symbols no cambiaron.
- Tabler y Game Icons usan
  `colección:categoría:nombre`, por ejemplo `tabler:animals:dog`.
- Rust busca, pagina, valida y persiste. JavaScript presenta y dibuja.
- Los recursos SVG se dividen por categoría; ninguno puede superar
  1.250.000 bytes.
- El mismo pintor sirve a rejilla, panel fijo y reproductor.
- El dibujo es decorativo (`aria-hidden`) y el nombre textual del sonido se conserva.

No crear una base de datos para estos catálogos estáticos ni un `kind` distinto por
cada paquete. `basic` significa monocromático.

## 3. Selector

- Pestañas visibles: `Emojis` y `Básicos`.
- No existe `Mostrar más`.
- La cuadrícula virtual conserva como máximo 300 elementos y un colchón de 100.
- Hay desplazamiento bidireccional, separadores e indicador de categoría.
- La búsqueda ignora acentos y permite términos localizados de categoría.
- `animal` y `animales` encuentran la categoría completa.
- Los tonos de piel se muestran solo cuando el usuario activa la casilla.
- El modal de edición usa una fila compacta con vista previa, selector y un único
  desplegable para `Restaurar icono original`, `Solo texto`, `Visual y texto` y
  `Solo visual`.
- Favoritos y recientes quedan para una ampliación posterior.

## 4. Recursos y selección

- Tabler está fijado a `v3.46.0`, commit
  `8ac7d81b72ece11072ef25ea9fd92e80c6f3c9fc`; se excluyen marcas y variantes
  terminadas en `-off`.
- Game Icons está fijado al commit
  `82d948812bfe3f269ef8f731dcdb07b08160edc4`; se excluye `badges`, se deduplican
  identificadores y se conserva su atribución CC BY 3.0.
- Una instantánea local de las 134 etiquetas oficiales de Game Icons clasifica
  4.131 de sus 4.133 conceptos; los dos restantes quedan en `Otros`.
- Los recursos derivados se reproducen con `visuals:generate-emojis`,
  `visuals:generate-basics` y `visuals:generate-packs`.
- `npm run visuals:verify` comprueba hashes, orden, cantidades, seguridad SVG,
  correspondencia catálogo/sprite, tamaño y cobertura de categorías.

Cobertura monocromática actual:

- Animales: 486.
- Naturaleza: 600.
- Oficina: 449.
- Objetos: 412.
- Audio: 247.
- Acciones: 742; antes de usar las etiquetas oficiales concentraba erróneamente
  3.396 elementos.

## 5. Evidencia conjunta más reciente

Verificación completada:

- generación determinista: Básicos 8.388;
- `npm run visuals:verify`: correcto;
- `cargo test --lib`: 320 aprobadas, 0 fallidas y 19 ignoradas;
- `cargo build --lib`: correcto;
- `npm run build`: correcto;
- 130 archivos JavaScript con sintaxis válida;
- i18n: 738 claves idénticas y no vacías en los cuatro idiomas;
- 27 módulos nuevos auditados, todos con un máximo de 200 líneas;
- prueba local Chromium: Tabler y Game Icons renderizaron correctamente
  desde sprites offline;
- `npm run tauri build -- --no-bundle`: correcto.

Ejecutable Release final:

- ruta: `src-tauri/target/release/tauri-app.exe`;
- tamaño: 42.848.768 bytes, 40,86 MiB;
- fecha local: 2026-08-02 07:39:45;
- versiones de producto y archivo: 1.3.0;
- SHA-256:
  `A782C4E988239B2BF22507A73319D7EBC84C2AE617E5113F9881E6A63D036269`;
- el binario contiene el recurso de producción `main-bmZx2EC-.js`.

Para crear el ejecutable autónomo se debe usar siempre:

```powershell
npm run tauri build -- --no-bundle
```

No usar `cargo build --release` como entrega: conserva el destino de desarrollo y
puede mostrar `localhost rechazó la conexión` sin Vite abierto.

## 6. Estado de las etapas anteriores

La Biblioteca y el Buscador fijo comparten un único catálogo en `tracks.db`, esquema
6. Están cerrados:

- raíces de Música y Efectos, observación incremental y lista virtual;
- Centro de procesamiento, retiro reversible, retención y purga segura;
- respaldo/restauración `.lfbackup`;
- metadatos, tags, renombrado físico opcional y escritura opcional en una pista;
- inicio informativo y activación de la observación de Biblioteca en segundo plano.

No reabrir la publicación 1.3.0, la Biblioteca, el retiro, el respaldo ni los
metadatos como continuidad activa.

## 7. Reglas de reanudación

1. Leer `AGENTS.md`, este archivo y `PLAN_EMOJIS_BOTONES.md`.
2. Continuar por la verificación pendiente; no rediseñar los catálogos.
3. No tocar `Capturas_Tienda/`: es material ajeno a esta fase.
4. No introducir parches ni mecanismos duplicados; resolver cualquier fallo desde su
   causa.
5. No crear commits, no preparar staging y no hacer push. El autor indicó que solo
   habrá commit cuando lo solicite expresamente.
6. MIDI e identificadores visuales no tuvieron beta ni versión pública anterior:
   describirlos en `CHANGELOG.md` como funciones nuevas, no como correcciones.

## 8. Pendientes no bloqueantes

- Prueba física del selector por el autor en el ejecutable Windows Release.
- Prueba posterior en Linux de `.deb`, `.AppImage` y WebKitGTK.
- Auditoría integral futura con lector de pantalla.
- Representación extensa de `master_volume` y `ButtonData.vol` por usar `f32` en JSON.
