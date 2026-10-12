# Continuidad de sesión — estado actual

**Actualizado:** 2026-09-17

**Rama de trabajo:** `main`

**Versión del código:** 1.4.0

Este archivo no es un historial. Conserva únicamente el trabajo activo, las
decisiones cerradas, la evidencia vigente y el siguiente paso real.

## 1. Estado actual

El volumen del reproductor auxiliar se controla únicamente desde el reproductor.
Se retiró el deslizador duplicado de Panel fijo y su carga/guardado en
`settingsPlayer.js`; guardar otros ajustes ya no escribe ese volumen.

El panel fijo restaura ahora su última visibilidad guardada, sin la preferencia
independiente `show_on_start`. Se retiraron las dos casillas de sus ajustes;
Principal conserva la opción de mostrar el botón superior. Pendiente prueba del
autor: reiniciar con el panel abierto/cerrado y guardar ajustes sin cambiarlo.

Mic está implementado localmente tras aprobar el autor el [plan](PLAN_MIC.md):
mantener pulsado, alternar y automático, con destinos combinables y nivel absoluto
(60 → 20 → 60). Rust captura/mide la entrada, resuelve pulsaciones y aplica una
atenuación temporal por bus, conservando los faders y el master. Mic es un botón
compacto de la barra principal, entre Solo y Vol; abre sus ajustes
con clic derecho. Configuración → Atajos solo contiene su atajo opcional por perfil.
Automático requiere activación explícita con Mic; ante fallo
de entrada restaura el nivel. No mezcla, graba ni transmite el micrófono.
Falta la prueba funcional del autor con ratón, teclado y entrada física.
La medición de ajustes comparte captura con automático al seleccionar el mismo
micrófono por nombre o como predeterminado. La resolución ocurre al elegir/abrir
la entrada, sin enumeración periódica. Pendiente comprobar ambas combinaciones
con el dispositivo físico.

El autor indicó expresamente no publicar hasta que lo solicite. Las mejoras se
acumulan en `[Sin publicar]` de `CHANGELOG.md`, sin cambiar aún la versión.

La barra inferior mantiene ahora hora y fecha permanentemente a la izquierda. El
contador existente tiene un destino independiente, justo a la izquierda de Detener
todo y en su misma fila; el botón permanece en el extremo derecho. Se eliminó la
alternancia con el reloj y su temporizador de tres segundos. Rust sigue calculando
el tiempo y el contador queda en cero al terminar. Falta la prueba visual del autor.

Los atajos opcionales Reproducir, Detener y Siguiente pista del reproductor auxiliar
están implementados localmente, sin publicar. Se configuran en el apartado existente
de teclado, nacen vacíos, pertenecen al perfil y respetan `global_keys`. Rust comparte
resolución y ejecución local/global, valida conflictos antes de guardar y reutiliza
el transporte del reproductor. Los perfiles exportados conservan los tres campos
opcionales. Falta la prueba funcional del autor con la interfaz real.

El arrastre desde el Buscador fijo y la ventana Biblioteca hacia la aplicación
principal está implementado. Las filas inician un arrastre nativo de sus rutas
reales mediante `tauri-plugin-drag`; `fileDrop.js` continúa como receptor único y
delega en las operaciones existentes para reproductor, botones fijos, celdas y
pestañas. Las pestañas admiten lotes atómicos y una celda o botón fijo exige una sola
pista. Falta únicamente la prueba funcional del autor entre las dos ventanas.

La publicación coordinada de 1.4.0 está cerrada en Microsoft Store y GitHub. Los tres
paquetes Linux de esa misma versión recibieron una corrección de presentación: los
controles nativos claros de WebKitGTK podían conservar fondo blanco dentro del tema
oscuro. EXE y MSI no se sustituyeron. El issue #5 permanece abierto únicamente para
recibir la comprobación física del reportante en Debian/KDE. La próxima etapa prevista
sigue siendo una reorganización y mejora visual de la interfaz; antes de construirla
se debe revisar el alcance y aprobar su plan.

## 2. Estado de Microsoft Store

- Producto: LF Botonera de Efectos (`9NJ8ST39QP7V`).
- Entrega: Submission 4 (`1152921505701566131`).
- Estado confirmado en Partner Center: `¡Felicidades! El producto ya está actualizado`.
- Partner Center indica que el producto más reciente ya está disponible en Microsoft
  Store y muestra Submission 4 como la presencia actual.
- Fecha pública adoptada para el proyecto: 2026-08-02, según la zona horaria del autor.
- El paquete 1.4.0 está publicado para Windows Desktop x64 y Microsoft Store administra
  su firma y sus actualizaciones.

## 3. Paquete definitivo enviado

Archivo local:

`Compilados/Microsoft-Store-1.4.0/LF-Botonera-1.4.0.0-x64-unsigned.msix`

- Tamaño: 17.394.512 bytes.
- SHA-256: `DB506CD721AFFB7118342360DD911E458257FE9929B0A33ADB0B5A26575344D1`.
- Identidad: `LuisFernandoVelasquez.LFBotoneradeEfectos`.
- Publisher: `CN=AD90DE58-447F-47AE-AC1A-3D369955282B`.
- Versión MSIX: `1.4.0.0`.
- Arquitectura: x64.
- Familia: Windows Desktop, versión mínima `10.0.19041.0`.
- El paquete se envió sin firma local; Microsoft Store lo firma durante su proceso.

## 4. Contenido cerrado de 1.4.0

- Entrada MIDI para Windows mediante WinMM y para Linux mediante `midir 0.11` sobre
  ALSA Sequencer.
- Selección de varias entradas, reconexión en caliente y asignación de Note On,
  Control Change y Program Change a botones, botones fijos, pestañas y acciones.
- La lista MIDI puede actualizarse con el control desactivado; las casillas de
  selección quedan bloqueadas hasta permitir los disparos. Rust conserva la
  selección previa ante un intento IPC mientras MIDI está desactivado.
- Identificadores visuales offline para los botones mediante las colecciones Emojis y
  Básicos. La categoría Coloridos se descartó antes de publicar y no se menciona como
  corrección.
- Respaldo `.lfbackup`, retiro seguro de rutas y edición de metadatos de la Biblioteca,
  según las notas públicas cerradas en `CHANGELOG.md`.
- Los controles nativos reciben el esquema claro u oscuro de la aplicación; en Linux
  los desplegables activos y deshabilitados dejan de conservar el fondo claro de
  WebKitGTK dentro del tema oscuro.

## 5. Changelog y versión

- `CHANGELOG.md` conserva el release `[1.4.0] — 2026-08-02` y ahora documenta en
  `[Sin publicar]` el arrastre de Biblioteca, los atajos del reproductor auxiliar y
  la separación del reloj y el contador de sonidos.
- La ventana de bienvenida busca primero contenido en `[Sin publicar]`; esta copia
  local mostrará esas mejoras. El release publicado conserva sus notas de 1.4.0.
- Se preservó el cambio manual del autor que eliminó una segunda viñeta técnica del
  selector visual.
- `package.json`, `package-lock.json`, `Cargo.toml`, `Cargo.lock` y
  `tauri.conf.json` están sincronizados en 1.4.0.
- `SET-VERSION.ps1` ahora lee los archivos UTF-8 explícitamente para no alterar
  acentos en futuras actualizaciones.

## 6. Evidencia vigente

- Captura compartida por entrada resuelta, 2026-10-11:
  `cargo test --lib --locked --offline`, 349 aprobadas, 0 fallidas y 19 ignoradas;
  `cargo build --lib --locked --offline`, correcto con el mensaje informativo MSVC;
  `npm run build` y `git diff --check`, correctos. Captura y worker dentro del
  límite de 200 líneas. Sin abrir la aplicación; pendiente comprobar la entrada
  predeterminada y por nombre con hardware real.

- Volumen único del reproductor, 2026-10-11: `npm run build`, correcto;
  `cargo test --lib --locked --offline`, 347 aprobadas, 0 fallidas y 19 ignoradas;
  `cargo build --lib --locked --offline`, correcto con el mensaje informativo MSVC.
  Estructura de bloques HTML y retirada de referencias del volumen en ajustes
  comprobadas; `git diff --check`, correcto. Sin abrir la aplicación.

- Visibilidad del panel fijo, 2026-10-11: `cargo test --lib --locked --offline`,
  347 aprobadas, 0 fallidas y 19 ignoradas; `cargo build --lib --locked --offline`,
  correcto con el mensaje informativo MSVC; `npm run build`, correcto.
  Configuraciones antiguas y persistencia de abierto/cerrado comprobadas por test;
  claves i18n alineadas y `git diff --check` correcto. Sin abrir la aplicación.

- Medición temporal de ajustes Mic, 2026-10-11: `cargo test --lib --locked --offline`,
  346 aprobadas, 0 fallidas y 19 ignoradas; `cargo build --lib --locked --offline`,
  correcto con el aviso informativo MSVC de creación de biblioteca; `npm run build`,
  correcto. `git diff --check`, correcto; módulos tocados de Mic bajo 200 líneas.
  No se abrió la aplicación. Pendiente la comprobación física del autor.

- Mic, 2026-09-17: `cargo test --lib --locked --offline`, 345 aprobadas,
  0 fallidas y 19 ignoradas. Las 13 pruebas añadidas cubren modos, niveles,
  histéresis, rampas estéreo, destinos combinables, atajos y exportación compatible.
  `cargo build --lib --locked --offline`, correcto; únicamente el mensaje
  informativo del enlazador MSVC sobre creación de `.lib`/`.exp`.
  `npm run build`, correcto. Claves de traducción de los cuatro idiomas alineadas;
  XML de la plantilla MSIX válido y módulos de Mic por debajo de 200 líneas.
  No se abrió la aplicación ni se probó la entrada física; esa validación queda
  con el autor. No se publicaron cambios ni se incrementó la versión.

- Separación de reloj y contador, 2026-09-17: `npm run build`, correcto;
  `cargo test --lib --locked --offline`, 332 aprobadas, 0 fallidas y 19 ignoradas;
  `cargo build --lib --locked --offline`, correcto con el mismo aviso informativo
  de MSVC. `git diff --check` y claves i18n de los cuatro idiomas, correctos.
  `clockWidget.js` tiene 95 líneas, `bottomBar.css` 165 y `playbackControls.css` 70.
  No se abrió la aplicación ni se publicó; la comprobación visual queda con el autor.

- Atajos del reproductor, 2026-09-17: `cargo test --lib --locked --offline`, correcto
  con 332 aprobadas, 0 fallidas y 19 ignoradas. Incluye 11 pruebas nuevas de resolución
  local/global, cambio de perfil, limpieza, conflictos y compatibilidad JSON/LFA.
- `cargo build --lib --locked --offline` y `npm run build`, correctos. El build Rust
  emitió únicamente el aviso informativo de MSVC al crear la biblioteca de enlace.
- Formato Rust y `git diff --check`, correctos; claves i18n iguales en los cuatro
  idiomas; módulos Rust/JS tocados por esta mejora dentro del máximo de 200 líneas.
- Tras la reinstalación de Windows, se instaló Rust/Cargo estable con el instalador
  oficial y rustfmt, por autorización del autor. Visual Studio C++ ya estaba
  disponible. `npm ci` restauró las versiones fijadas; no se actualizaron dependencias.
- No se ejecutó la aplicación. Se conservaron los cambios locales previos de arrastre
  y no se tocó `Capturas_Tienda/`.

- Arrastre de Biblioteca: `cargo test --lib`, correcto con 321 aprobadas, 0 fallidas
  y 19 ignoradas; `cargo build --lib` y `npm run build`, correctos.
- `npm run licenses`, correcto; los avisos incluyen `drag 2.1.1` y
  `tauri-plugin-drag 2.1.1`, ambos MIT/Apache-2.0.
- `npm run tauri build -- --no-bundle`, correcto. Ejecutable Release para la prueba
  funcional: `src-tauri/target/release/tauri-app.exe`, 42.847.232 bytes y SHA-256
  `8FFE58EF0EDACE7D21F4920877801918EF4ADB6F8852CA2A7D5D5B461A457200`.

- `cargo check`: correcto.
- `cargo test --lib`: 321 aprobadas, 0 fallidas y 19 ignoradas.
- Prueba específica de notas de versión 1.4.0: correcta.
- `cargo build --lib`: correcto.
- `npm run build`: correcto.
- `npm run visuals:verify`: correcto.
- `npm run licenses`: correcto.
- Compilación Release del canal Store y creación del MSIX definitivo: correctas.
- GitHub Actions `30785123248`, sobre el commit `48f411a`, completó correctamente las
  pruebas y compilaciones Release de Windows y Linux.
- Corrección de controles nativos: `cargo test --lib` mantiene 321 aprobadas, 0
  fallidas y 19 ignoradas; `cargo build --lib`, `npm run build`,
  `npm run visuals:verify` y `tauri build --no-bundle` finalizaron correctamente.
- Ejecutable Release para la prueba visual Windows:
  `Compilados/Prueba-tema-1.4.0-Windows/LF-Botonera-1.4.0-prueba-tema.exe`;
  SHA-256 `1545C0314E6A9C03620916E990BAFEAD919B769E0479B5CC7A0E16CC831330F3`.
- El autor confirmó en Windows que los controles se ven correctamente y funcionan
  sin problemas tanto en modo claro como en modo oscuro.
- GitHub Actions `30844689983`, sobre el commit `59a7587`, ejecutó únicamente el job
  Ubuntu y finalizó correctamente. La publicación estaba desactivada; Windows se
  omitió y el release público no se modificó.
- Los paquetes privados descargados están en
  `Compilados/Linux-tema-1.4.0-run-30844689983/`: AppImage de 106.109.432 bytes
  (`205AD151DE73AEDDFD2630AF29ABDAA329CA53D449A329B0AB87E12235D4FD62`),
  DEB de 18.274.610 bytes
  (`0EFAF1ED157D9D881322321EAB0759E4410F4A04797FB1CEEF4926B588B12BF5`) y
  RPM de 18.278.577 bytes
  (`0427A85A9A7B9C563318F1CE72A79BF7A0CD0C5DDA91DADC81D9BC2774FCB5`).
- Esos tres hashes coinciden con los assets Linux públicos sustituidos el 2026-08-03.
  Los paquetes Windows públicos permanecen intactos: EXE
  `A7F7AAD9FA462452607941ADE493C67FB5D1D6C3D829B94EAAF59D4588A61979` y MSI
  `1759718E576A69AA2577E8764EFDB026B581125BB1DE789A1B14A2BE096E1479`.

Sigue pendiente la prueba física final del autor con controlador MIDI en Linux y en
los distintos canales de instalación Windows. No bloquea la certificación ya enviada.

## 7. GitHub publicado

- `main` contiene el estado cerrado de 1.4.0.
- El PR #8 integró la corrección de controles en `main` mediante `1b1955a`; su rama
  temporal fue eliminada localmente y del remoto.
- La rama temporal `codex/midi-linux-build` fue eliminada localmente y del remoto
  después de comprobar que estaba completamente integrada.
- El release público `v1.4.0` contiene notas para usuarios finales y una nota sobre la
  siguiente actualización visual.
- Tiene adjuntos cinco paquetes: `.exe`, `.msi`, `.deb`, `.rpm` y `.AppImage`.
- Las copias originales están en `Compilados/GitHub-1.4.0/`; los tres paquetes Linux
  renovados y verificados están en `Compilados/Linux-tema-1.4.0-run-30844689983/`.
- La etiqueta `v1.4.0` apunta al commit `75c3da7`, el mismo estado final usado para
  cerrar el changelog y publicar originalmente la versión. Los assets Linux renovados
  fueron compilados de forma controlada desde `59a7587` sin mover la etiqueta.
- Las notas del release incluyen la mejora Linux y recomiendan volver a descargar el
  paquete a quien lo hubiera obtenido antes del 2026-08-03.
- El issue #5 recibió el comentario de seguimiento `5171062242` y no se cerrará hasta
  conocer el resultado real en Debian 13, KDE Plasma y Brisa oscuro.

## 8. Siguiente paso

Probar [Mic](PLAN_MIC.md): mantener/alternar/automático, atajos locales y globales,
combinaciones de destinos, restauración desde 100/60/40%, desconexión de entrada y
transiciones rápidas. En ajustes, el nivel es absoluto, no porcentaje relativo.
Cuando se autorice publicar, sincronizar también la sección de micrófono de
`PRIVACY.md` con su página web. La plantilla MSIX ya declara la entrada de micrófono.
Conservar como pendientes las comprobaciones funcionales siguientes.

Comprobar visualmente la barra inferior con sonidos cortos, largos y simultáneos,
al terminar y al pulsar Detener todo: hora/fecha siempre visibles, contador junto
al botón y posición de Detener todo conservada. Revisar también al reducir la ventana.

1. Probar los atajos del reproductor: asignar y limpiar los tres campos; guardar y
   reabrir; comprobar Play, Stop y Siguiente con el panel en sus tres vistas, con y
   sin foco según la opción global, y cambiar de perfil. Comprobar que una combinación
   en conflicto se rechaza sin reemplazar asignaciones.
2. Probar el arrastre real desde la ventana Biblioteca hacia reproductor, botones
   fijos, botones principales y pestañas; comprobar también una selección múltiple
   sobre reproductor y pestaña.
3. Esperar la comprobación visual del reportante en Debian/KDE y cerrar el issue #5
   solo si confirma que el contraste quedó corregido.
4. Cuando el autor lo indique, estudiar y acordar el plan de reorganización visual
   antes de modificar la interfaz.
5. Mantener como pruebas pendientes no bloqueantes la comprobación física MIDI en
   Linux y en los distintos canales de instalación Windows.

No tocar `Capturas_Tienda/`; es material ajeno a esta tarea.
