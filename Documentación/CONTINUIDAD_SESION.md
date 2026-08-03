# Continuidad de sesión — estado actual

**Actualizado:** 2026-08-02

**Rama de trabajo:** `codex/midi-linux-build`

**Versión del código:** 1.3.0

Este archivo no es un historial. Conserva únicamente el trabajo activo, las
decisiones aprobadas, la evidencia disponible y el siguiente paso real.

## 1. Objetivo activo

Completar la entrada MIDI de la próxima versión para Windows y Linux. No hay otro
trabajo activo en esta sesión.

La función todavía no ha llegado al público. Todo cambio relacionado con MIDI debe
describirse en `CHANGELOG.md` dentro de `Añadido`, nunca como corrección.

## 2. Decisiones aprobadas

- Windows conserva el backend WinMM existente sin cambios.
- Linux usa `midir 0.11` sobre ALSA Sequencer.
- `midir` es una dependencia exclusiva del target Linux.
- Se descartó implementar directamente sobre el crate `alsa`.
- Captura, conflictos, asignaciones, acciones, IPC, persistencia y reconexión en
  caliente siguen siendo lógica común para ambas plataformas.
- Los paquetes Linux en alcance son `.deb`, `.rpm` y `.AppImage`.
- No crear un plan temporal separado: la arquitectura permanente está en
  `ARCHITECTURE.md` y este archivo conserva el punto de reanudación.
- No crear commits, preparar staging ni hacer push hasta que el autor lo pida.
- No tocar `Capturas_Tienda/`; es material ajeno a esta tarea.

## 3. Implementación actual

Se añadieron dos adaptadores exclusivos de Linux:

- `engine/input/midi_ports_linux.rs` enumera entradas mediante `midir`.
- `engine/input/midi_backend_linux.rs` abre cada entrada seleccionada y entrega sus
  bytes al analizador MIDI común.

El adaptador Windows continúa en `midi_ports_windows.rs` y
`midi_backend_windows.rs`. `midi_ports.rs` y `midi_backend.rs` eligen el adaptador con
`cfg(target_os)`; otros sistemas conservan el backend vacío.

Linux no persiste directamente la dirección ALSA `cliente:puerto`, porque puede
cambiar al reconectar. Guarda un identificador derivado del nombre estable y de la
posición entre puertos con el mismo nombre. La dirección ALSA actual solo sirve para
abrir el puerto durante esa sesión. Dos controladores iguales pueden usarse a la vez;
si el sistema invierte su orden, no se puede distinguir cuál unidad física era cada
una, la misma limitación práctica documentada para WinMM.

`Cargo.toml` declara `midir = "0.11"` únicamente para Linux. El árbol resuelto usa la
misma dependencia `alsa 0.9.1` que ya incorporaba `rodio` mediante `cpal`, por lo que
MIDI no añade otra biblioteca nativa al sistema. `Cargo.lock` y los avisos de
licencias fueron regenerados.

## 4. Alcance funcional compartido

- Activar MIDI y seleccionar una o varias entradas desde Ajustes.
- Aplicar la selección y las conexiones o desconexiones sin reiniciar.
- Asignar Note On, Control Change y Program Change a botones, botones fijos,
  pestañas y acciones globales.
- Ignorar Note Off y Note On con velocidad cero para evitar dobles disparos.
- Capturar la siguiente orden MIDI sin bloquear la ventana y cancelar de inmediato.
- Conservar en la interfaz una entrada seleccionada aunque esté temporalmente
  desconectada.
- Mostrar y actualizar los dispositivos aunque MIDI esté desactivado, pero bloquear
  sus casillas de selección hasta permitir los disparos. Rust conserva la selección
  anterior si un IPC intenta cambiarla mientras MIDI está desactivado.

## 5. Documentación y texto público

- `ARCHITECTURE.md`, `LIBRO_PROYECTO.md`, `GLOSARIO.md` y `AGENTS.md` describen los
  backends WinMM y ALSA y la separación por plataforma.
- `CHANGELOG.md` presenta MIDI como una función nueva para Windows y Linux con texto
  orientado al público.
- La única modificación relativa al respaldo fue aclarar en `CHANGELOG.md` qué
  contiene `.lfbackup` y que los audios deben conservarse por separado. No se cambió
  código de respaldo, pistas, cue, ganancia ni normalización.

## 6. Evidencia disponible

- `cargo test --lib`: 321 aprobadas, 0 fallidas y 19 ignoradas; incluye la protección
  backend de la selección cuando MIDI está desactivado.
- `cargo build --lib`: correcto.
- `npm run build`: correcto.
- `npm run licenses`: correcto después de descargar el contenido ya fijado por
  `Cargo.lock`.
- `cargo tree --target x86_64-unknown-linux-gnu -i alsa`: `rodio/cpal` y `midir`
  convergen en `alsa 0.9.1`.
- Los dos módulos Linux tienen menos de 200 líneas.
- GitHub Actions `30782348034`, sobre el commit `7472eaa`, compiló correctamente los
  instaladores Windows `.exe` y `.msi` y los paquetes Linux `.deb`, `.rpm` y
  `.AppImage` en Ubuntu 22.04.
- `scripts/build-store-msix.ps1` generó correctamente el MSIX 1.3.0.0 sin firma con
  la identidad oficial de Microsoft Store.
- Los seis paquetes y sus hashes SHA-256 están en
  `Compilados/MIDI-1.3.0-7472eaa/`. Esta carpeta es local y está ignorada por git.

La compilación nativa de Linux está verificada. Todavía no se ha realizado una prueba
funcional con controlador físico en Linux ni en cada canal de instalación Windows.

## 7. Siguiente paso

1. Probar en Linux con un controlador real: selección, captura, disparo, desconexión y
   reconexión sin reiniciar.
2. Probar en Windows los canales `.exe`, `.msi` y Microsoft Store; todos comparten el
   backend WinMM, pero la confirmación física sigue siendo necesaria.

La implementación y los paquetes están listos; la etapa pendiente es la prueba física
del autor.
