# Integración de aportaciones de Danilo Wenner

## Estado y alcance — 2026-11-13

Primera fase autorizada e implementada localmente: rueda, barra inferior y FFmpeg.
Las otras tres funciones siguen pendientes; no se compila ni publica en GitHub.
El autor seleccionó seis aportaciones de LF Botonera Radio. La integración parte
de la rama de desarrollo oficial 1.6.0, no de sustituirla por el fork 1.5.1.
El autor aprobó explícitamente la primera fase; las fases restantes requieren
revisión individual antes de modificar módulos, modelo o IPC.

## Entorno aislado

- Rama local y GitHub: `variante-danilo`.
- Base: `3502072a5ff9ba3993bd4ef7ef63044f85e47114` de `codex/desarrollo`.
- Worktree: `C:/OVERLAY/BOTONERA/.worktrees/variante-danilo`.
- Referencia descargada: `C:/OVERLAY/BOTONERA/.references/danilo`.
- Las dos carpetas están excluidas localmente del repositorio principal.
- La carpeta principal conserva `codex/desarrollo` y sus cambios sin commit.
  Esos cambios NO están incluidos en esta base; conciliarlos antes de integrar
  el resultado o preparar una entrega, sin sobrescribirlos ni descartarlos.
- No modificar main, publicar releases ni cambiar versiones durante la preparación.
- D: queda para experimentos reversibles; el trabajo de integración reside en C:.

## Procedencia y crédito permanente

Autor de las aportaciones: **Danilo Wenner** (GitHub: `danilo-wenner-1`).
Repositorio: https://github.com/danilo-wenner-1/LF-Botonera-de-efectos
Variante: LF Botonera Radio 1.5.1, bajo GPL-3.0-or-later.
Commit funcional: `7c1f1ae484cdb4bd0110b4761b8a221023bdea6f`.
Referencia descargada: `97d935c9eb145eb9ac9c25eef61b244ec2504fac`.

El commit funcional agrupa más funciones que las seleccionadas: NO fusionarlo
completo. Extraer por función conservando avisos y una referencia al commit
original. Preservar su autoría en los cambios importados; registrar por separado
las adaptaciones del mantenedor, con identidad humana real. No atribuir a Danilo
código nuevo que no escribió. No reutilizar su nombre/ID de aplicación ni su
actualizador: conservar la identidad oficial y los canales PC/Android vigentes.

Antes de publicar, revisar los créditos de CHANGELOG, README y avisos legales;
las notas deben nombrar las funciones realmente incorporadas y enlazar el fork.
No presentar este plan como funciones ya disponibles.

## Funciones seleccionadas y propuesta de adaptación

### 1. Rueda sobre la barra de pestañas

Arriba: pestaña anterior; abajo: siguiente, mediante el comando Rust existente
`cycle_paleta`. JS solo interpreta el gesto y refresca la vista. Mantener el
scroll horizontal y evitar saltos excesivos de trackpad. Fuera de la barra,
la rueda mantiene su comportamiento actual. Sin nuevos campos ni IPC.

### 2. Colores HEX y degradados

Incorporar edición HEX y degradados con vista previa, revisando la claridad del
editor antes de implementarlo. Preservar colores anteriores, temas y selección
múltiple. Proponer campos opcionales con `serde(default)` y validación en Rust;
CSS/JS solo representan y editan visualmente. Comprobar exportación/importación
con LF Automatizador y los cuatro idiomas. La antigua política automática de
colores descartada no forma parte de este trabajo. Tamaño de fuente por botón
no está seleccionado en esta primera fase.

### 3. Salida de audio por pestaña integrada en consola

Aprovechar la intención del ruteo, no copiar el bypass del fork. La consola
oficial debe seguir siendo dueña de las tarjetas y de sus buses; una tarjeta
física no debe abrirse varias veces. La propuesta detallada debe explicar cómo
se aplican máster, Mic, faders y medidores a los destinos independientes y cómo
se representan en la consola, sin doble ganancia ni mezcla física ficticia.
Mantener CUE separado; definir explícitamente reconexión y destino alternativo
si falta una tarjeta antes de programar. La salida propia no puede quedar fuera
del máster/Mic/medición como ocurre en el fork. Revisar Windows y Linux; no
habilitar este ruteo en Android sin comprobar su backend.

### 4. Unificar el acceso a los dos buscadores

Decisión expresa del autor, aclarada durante la preparación:
- Clic izquierdo: buscar botones colocados en todas las pestañas del perfil.
- Clic derecho: buscar archivos del catálogo existente (`tracks.db`).

Reutilizar la entrada visual del buscador actual y conservar las capacidades
del catálogo. Buscar botones en Rust sobre AppConfig, sin duplicarlos en SQLite
ni añadir otro catálogo: la ubicación de botones vive en la configuración.
Al elegir un botón, abrir su pestaña y resaltarlo sin dispararlo ni interrumpir
audio. Proponer cómo se accede a ambos modos con teclado y toque/Android, donde
no hay clic derecho, y mostrar cuál está activo. Requiere propuesta de UI e IPC.

### 5. Conversión automática con FFmpeg

Conservar la idea de una conversión auxiliar para archivos no decodificables
de forma nativa. Procesar en Rust con trabajador y cola fuera del hilo de audio;
no modificar originales. Proponer caché, invalidación, limpieza, progreso y
errores; integrar reproducción, editor, cues y análisis con la misma resolución.
En el fork se convierte a MP3 320 kbps. Esta integración usa FLAC para no
introducir otra compresión con pérdida; ver CONVERSION_AUDIO.md.

Antes de añadir FFmpeg, justificar necesidad, origen y mantenimiento, licencia
y distribución de su código fuente, tamaño de instaladores, seguridad y builds.
Separar disponibilidad Windows/Linux de Android: no copiar un EXE de Windows
al paquete Android ni prometer paridad sin una solución verificada.

### 6. Barra inferior ocultable

Añadir una preferencia persistente, visible por defecto para configuraciones
anteriores. Mantener un acceso claro en Ajustes para volver a mostrarla y no
alterar atajos ni audio. JS/CSS solo aplican visibilidad; Rust valida/persiste
la preferencia. Revisar escritorio y Android antes de definir el alcance final.

## Orden propuesto y comprobaciones

1. Primera fase: rueda, barra inferior y conversión FFmpeg para desarrollo local.
2. Buscador unificado y editor de colores, cada uno con su revisión individual.
3. Ruteo integrado en consola, con diseño de buses y reconexión aprobado.
4. Distribución de FFmpeg en instaladores, después de validar la primera fase.

Trabajar en cambios pequeños por función, con procedencia y créditos trazables.
Mantener archivos de máximo 200 líneas y traducciones es/en/pt-BR/pt-PT.
Al modificar código: `cargo test --lib`, `cargo build --lib` y `npm run build`,
más pruebas dirigidas cuando verifiquen comportamiento real. No lanzar la app;
la validación funcional y de tarjetas la realiza el autor con su equipo.
Verificaciones locales de biblioteca y frontend, sin abrir la app ni crear
instaladores. El autor prueba con el DEV.bat del worktree aislado.

## Encontrar las opciones en el instalador de Danilo

- Barra: Ajustes → Principal → «Mostrar la barra inferior (reloj, modos, Mic,
  volumen, Detener todo)»; desmarcar para ocultarla.
- FFmpeg: no tiene botón ni pantalla de conversión. Al asignar/importar un
  M4A/MP4/AAC candidato, decide si puede reproducirlo nativamente; si no,
  prepara una copia MP3 en segundo plano. No todo M4A fuerza conversión.
  Si se dispara durante la preparación, muestra un aviso para reintentar.
- La prueba manual del instalador de Danilo sigue a cargo del autor.
- Esta rama tiene pruebas sintéticas de conversión y pruebas DOM; su prueba
  funcional con DEV.bat queda a cargo del autor.
