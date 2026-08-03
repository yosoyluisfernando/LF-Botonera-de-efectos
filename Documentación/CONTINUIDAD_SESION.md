# Continuidad de sesión — estado actual

**Actualizado:** 2026-08-02

**Rama de trabajo:** `main`

**Versión del código:** 1.4.0

Este archivo no es un historial. Conserva únicamente el trabajo activo, las
decisiones cerradas, la evidencia vigente y el siguiente paso real.

## 1. Objetivo activo

Publicar en GitHub la versión 1.4.0 ya disponible en Microsoft Store, usando el release
en borrador y los cinco paquetes previamente verificados.

No añadir funciones nuevas ni cambiar el número de versión durante esta etapa.

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

Sigue pendiente la prueba física final del autor con controlador MIDI en Linux y en
los distintos canales de instalación Windows. No bloquea la certificación ya enviada.

## 7. GitHub preparado

- `main` contiene el estado cerrado de 1.4.0.
- La rama temporal `codex/midi-linux-build` fue eliminada localmente y del remoto
  después de comprobar que estaba completamente integrada.
- Existe un release `v1.4.0` en borrador, dirigido a `main`, con notas para usuarios
  finales y una nota sobre la siguiente actualización visual.
- El borrador tiene adjuntos cinco paquetes: `.exe`, `.msi`, `.deb`, `.rpm` y
  `.AppImage`.
- Las copias locales verificadas están en `Compilados/GitHub-1.4.0/`.
- No existe todavía `refs/tags/v1.4.0`; publicar el borrador creará la etiqueta cuando
  corresponda.

## 8. Siguiente paso

1. Confirmar el cierre de documentación en `main`.
2. Publicar el release de GitHub y comprobar que la etiqueta `v1.4.0` apunta al estado
   final de `main` y que conserva los cinco instaladores.
3. Mantener `[Sin publicar]` vacío para el siguiente ciclo.

No tocar `Capturas_Tienda/`; es material ajeno a esta tarea.
