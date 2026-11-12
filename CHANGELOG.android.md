# Novedades de LF Botonera de Efectos para Android

Este archivo contiene las notas que muestra la edición Android. Las notas
para Windows y Linux se mantienen en [CHANGELOG.md](CHANGELOG.md).
La versión 1.6.0 prepara la primera distribución pública de Android.

## [Sin publicar]

## [1.6.0] — Primera versión pública

### Botonera de sonidos para teléfonos y tablets

- Organiza tus sonidos en perfiles y pestañas, asigna audios a los botones y
  reprodúcelos con controles táctiles. Mantén pulsado un botón para abrir su menú.
- Importa audios con el selector de Android y guarda copias privadas en la app.
  Comparte pestañas y perfiles mediante los formatos compatibles con escritorio.
- Usa el panel fijo con botones, reproductor auxiliar o buscador, según tus ajustes.
- Ajusta inicio, final, ganancia y normalización en el editor de pistas táctil.
  Las locuciones admiten las carpetas de hora, temperatura y humedad.
- Conecta controladores MIDI por USB o Bluetooth LE y asigna pads, notas o
  controles a botones y acciones. La búsqueda y conexión Bluetooth MIDI se hacen
  desde Ajustes → MIDI y teclado. La conexión Launchpad por Bluetooth MIDI fue
  comprobada por el autor en un teléfono Android.
- Activa Bluetooth en los ajustes de Android antes de buscar. El permiso de
  dispositivos cercanos permite buscar y conectar, pero no activa Bluetooth;
  la aplicación mantiene su activación bajo el control del usuario.
- Los atajos de teclado y MIDI del editor se agrupan en desplegables según la
  disponibilidad de dispositivos. Puedes mostrar u ocultar sus etiquetas en
  los botones; ambas etiquetas se muestran de forma predeterminada.
- Consulta los teclados físicos USB/Bluetooth conectados y abre la vinculación
  Bluetooth de Android desde los ajustes. El teclado Bluetooth de escritura
  todavía no cuenta con una prueba física del autor.
- Usa Atrás para regresar desde formularios, diálogos y paneles. En la pantalla
  principal se solicita confirmación antes de salir.
- Conecta el control remoto por red local a una PC para disparar y detener sus
  efectos desde Android; en ese modo el audio se reproduce en la PC.

### Alcance de esta edición

- La integración local con OBS Studio, Mic, la ventana Biblioteca, las ventanas
  flotantes y la selección independiente de salidas físicas no se incluyen en
  Android. La preescucha y el funcionamiento de audio en segundo plano tampoco
  están disponibles en esta edición.
- Esta primera versión pública de Android incluye apartados cuya adaptación
  continúa en desarrollo. Según el tamaño y la resolución del teléfono o la
  tablet, algunas pantallas pueden presentar desajustes visuales.

### Distribución y actualizaciones

- La distribución disponible se realiza mediante GitHub Releases. La app
  identifica Android y muestra el canal responsable de las actualizaciones.
- El reconocimiento de instalaciones desde Google Play y F-Droid está preparado
  para una posible distribución futura. La aplicación aún no se publica en
  esas tiendas. Cuando corresponda, las actualizaciones quedarán a cargo del
  canal desde el que se haya instalado.

## [1.6.0-beta.1] — 2026-11-10

Registro histórico del prelanzamiento; no forma parte de las notas de Android 1.6.0.

### Android ARM64 · Alfa

- Primera edición móvil en desarrollo. Incluye importación de audio mediante el selector de Android, modos de reproducción de la botonera y MIDI por USB y Bluetooth LE. La conexión MIDI Bluetooth ya se probó en un teléfono.
- Interfaz táctil adaptada para vertical y horizontal, con menús por pulsación larga, Ajustes por secciones, volumen accesible y panel fijo que empieza cerrado en una instalación limpia.
- Editor de pistas táctil con cursor en la onda, marcadores de inicio y fin, zoom y ajuste de ganancia.

**Android sigue en alfa.** Algunas funciones pueden fallar y algunas pantallas pueden verse incompletas o desalineadas según el dispositivo.
