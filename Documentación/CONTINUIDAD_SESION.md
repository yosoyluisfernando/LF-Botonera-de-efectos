# Continuidad de sesión — estado actual

**Actualizado:** 2026-08-02

**Rama de trabajo:** `main`

**Versión del código:** 1.4.0

Este archivo no es un historial. Conserva únicamente el trabajo activo, las
decisiones cerradas, la evidencia vigente y el siguiente paso real.

## 1. Objetivo activo

Esperar la certificación de Microsoft Store para la versión 1.4.0. GitHub se prepara
por separado en `main`, con compilaciones de Windows y Linux y un release en borrador,
pero no debe publicarse hasta confirmar que Microsoft hizo pública la actualización.

No añadir funciones nuevas ni cambiar el número de versión durante esta etapa.

## 2. Estado de Microsoft Store

- Producto: LF Botonera de Efectos (`9NJ8ST39QP7V`).
- Entrega: Submission 4 (`1152921505701566131`).
- Estado confirmado en Partner Center: `Actualización en certificación`.
- Fase mostrada al enviar: `Preprocesando`, paso 2 de 4.
- Publicación: automática tan pronto como supere la certificación; no hay una fecha
  programada ni una retención manual.
- El paquete 1.4.0 fue aceptado y marcado como `Validated` para Windows Desktop x64.
- La versión 1.3.0 continúa pública mientras Microsoft procesa la actualización.

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

- `CHANGELOG.md` conserva `[Sin publicar]` vacío y las novedades bajo `[1.4.0]`, sin
  fecha mientras la versión no esté publicada.
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
- GitHub Actions `30782348034`, sobre el commit `7472eaa`, compiló correctamente
  `.exe`, `.msi`, `.deb`, `.rpm` y `.AppImage` con la implementación MIDI.

Sigue pendiente la prueba física final del autor con controlador MIDI en Linux y en
los distintos canales de instalación Windows. No bloquea la certificación ya enviada.

## 7. Siguiente paso

1. Consultar el estado de Submission 4 hasta que Microsoft apruebe y publique 1.4.0.
2. Confirmar la versión desde la ficha pública y una instalación de Microsoft Store.
3. Comprobar que las compilaciones de GitHub para Windows y Linux terminaron y que sus
   paquetes quedaron adjuntos al release 1.4.0 en borrador.
4. Solo entonces publicar el release de GitHub y crear o confirmar la etiqueta
   `v1.4.0` con el mismo estado de `main`.
5. Al completar ambas publicaciones, añadir la fecha real a `[1.4.0]` y abrir una
   nueva sección `[Sin publicar]` para el siguiente ciclo.

No se ha creado todavía commit, etiqueta, pull request ni release de GitHub para
1.4.0. No tocar `Capturas_Tienda/`; es material ajeno a esta tarea.
