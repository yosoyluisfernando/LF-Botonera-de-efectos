# Continuidad de sesión — estado actual

**Actualizado:** 2026-08-03

**Rama de trabajo:** `codex/close-linux-theme-release`

**Versión del código:** 1.4.0

Este archivo no es un historial. Conserva únicamente el trabajo activo, las
decisiones cerradas, la evidencia vigente y el siguiente paso real.

## 1. Estado actual

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

- `CHANGELOG.md` conserva `[Sin publicar]` vacío y cierra las novedades bajo
  `[1.4.0] — 2026-08-02`.
- La ventana de bienvenida busca primero contenido en `[Sin publicar]`; al estar vacío,
  carga la sección que coincide con `CARGO_PKG_VERSION`, por lo que mostrará 1.4.0.
- Se preservó el cambio manual del autor que eliminó una segunda viñeta técnica del
  selector visual.
- `package.json`, `package-lock.json`, `Cargo.toml`, `Cargo.lock` y
  `tauri.conf.json` están sincronizados en 1.4.0.
- `SET-VERSION.ps1` ahora lee los archivos UTF-8 explícitamente para no alterar
  acentos en futuras actualizaciones.

## 6. Evidencia vigente

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

1. Esperar la comprobación visual del reportante en Debian/KDE y cerrar el issue #5
   solo si confirma que el contraste quedó corregido.
2. Cuando el autor lo indique, estudiar y acordar el plan de reorganización visual
   antes de modificar la interfaz.
3. Mantener como pruebas pendientes no bloqueantes la comprobación física MIDI en
   Linux y en los distintos canales de instalación Windows.

No tocar `Capturas_Tienda/`; es material ajeno a esta tarea.
