# Hora, clima y panel fijo en Android

En Bienvenida, la etiqueta y el interruptor táctil activan Hora y clima.
En Ajustes → Hora y clima se pueden elegir por separado las carpetas de hora,
temperatura y humedad. Se pueden elegir también en el editor de un botón de
locución que use una carpeta propia.

El comando `pick_named_folder` recibe una opción para identificar una carpeta
de locuciones. Android usa el selector de carpetas del sistema y la autorización
de lectura que concede para esa carpeta. Kotlin enumera los documentos del
primer nivel. Rust copia los archivos con extensiones de audio reconocidas al
directorio privado de la app, sin cambiar el nombre de cada audio. El motor de
locuciones usa esas copias. Para incorporar cambios del paquete original hay
que elegir la carpeta de nuevo. Windows y Linux conservan su selector y sus
rutas originales.

En vertical, al abrir el panel fijo, este ocupa el área de trabajo. El control
Volver a la botonera permite cerrarlo. En horizontal, el panel sigue a un
costado de la rejilla. La consola de audio mezcla en segundo plano en Android.
Sus buses internos de efectos, panel y preescucha arrancan al 100 %.
El volumen maestro y el volumen propio del reproductor conservan sus
preferencias y controles.
