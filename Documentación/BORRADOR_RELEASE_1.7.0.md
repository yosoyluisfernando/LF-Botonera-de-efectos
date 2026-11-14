# LF Botonera de Efectos 1.7.0 — borrador para Windows y Linux

Esta versión incorpora y adapta funciones desarrolladas originalmente por **Danilo Wenner** para [LF Botonera Radio](https://github.com/danilo-wenner-1/LF-Botonera-de-efectos). Se integraron con los perfiles, el panel fijo, la consola de audio y los controles existentes de LF Botonera de Efectos; además, se añadieron mejoras de uso y compatibilidad.

## Novedades

- **Navegación por pestañas:** desplázate entre pestañas con la rueda del ratón sobre la barra.
- **Barra inferior configurable:** ocúltala o vuelve a mostrarla desde Ajustes → Principal.
- **Búsqueda rápida de botones:** haz clic izquierdo en la lupa para buscar y resaltar botones en las pestañas del perfil. El clic derecho abre el Buscador del panel fijo, lo muestra si está oculto y cambia a esa sección si el panel ya está abierto.
- **Más opciones de color:** elige los colores predefinidos, introduce valores HEX o crea degradados en cuatro direcciones. La vista previa se actualiza mientras editas.
- **Favoritos de color:** guarda combinaciones completas con nombre para reutilizarlas en todos los perfiles. Los favoritos se conservan al cerrar la aplicación.
- **Atajos para pestañas:** asigna un atajo desde el menú contextual de la pestaña. La asignación MIDI también está disponible cuando MIDI está habilitado y hay una entrada conectada.
- **Reproducción de MP4, M4A y AAC:** la aplicación reproduce directamente los perfiles de audio compatibles. En un MP4 que contiene vídeo, reproduce solo la pista de audio. MP4 también aparece en la lista de formatos compatibles de Acerca de.

## Mejoras y correcciones

- El editor de botones se ajusta a la altura de la ventana; su contenido puede desplazarse para acceder a todas las opciones y conservar visibles la cabecera y las acciones.
- Al cambiar el archivo o el tipo de un botón existente se conservan sus colores y degradados. La asignación aleatoria de color se aplica al crear botones nuevos.
- El botón de la consola vuelve a mostrar y enfocar su ventana si ya estaba abierta, incluso cuando había quedado detrás de la ventana principal.

La salida de audio independiente por pestaña no forma parte de esta versión. La reproducción compatible de MP4/M4A/AAC es nativa; esta entrega no incorpora un convertidor FFmpeg.

## Disponibilidad

Edición de escritorio para **Windows y Linux**. Los instaladores y sus comprobaciones se añadirán a este borrador después de las compilaciones correspondientes. El borrador queda pendiente de revisión y autorización antes de publicarse.
