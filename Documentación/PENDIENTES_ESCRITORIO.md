# Continuidad de escritorio después de Android estable

Solicitud del autor, 2026-11-11. Retomar después de preparar la primera
compilación estable Android. Trabajar en C:/OVERLAY/BOTONERA, codex/desarrollo.
No promover main ni generar una estable de escritorio solo por cerrar Android.

## Ya implementado en código compartido

- Conservar nombres conocidos de dispositivos Bluetooth MIDI.
- Mostrar u ocultar de forma independiente las etiquetas de teclado y MIDI de
  la rejilla principal. Ambas visibles por defecto, opciones en Atajos del teclado.
- Selección de actualizaciones GitHub por archivos de la plataforma actual.
- Notas de escritorio separadas de las notas públicas Android.

Estas mejoras no implican que se haya entregado un instalador nuevo de PC.
Verificar y empaquetar Windows/Linux cuando lo solicite el autor.

## Pendientes que deben retomarse

- Revisar con el autor los arreglos de PC que quiera priorizar: aún no se ha
  proporcionado en este chat una lista concreta de nuevos fallos de escritorio.
- Evaluar controles Bluetooth más amplios en PC; no se ha pedido ni implementado
  que Windows/Linux activen Bluetooth automáticamente. Android conserva la
  activación exclusivamente bajo el control del usuario.
- Pruebas físicas MIDI en Linux con DEB, RPM y AppImage; confirmación del autor
  con su controlador en Windows/Linux.
- Confirmación manual de los ajustes del editor que se limitan al viewport.
- Reorganizaciones visuales o cambios de módulos/IPC adicionales requieren el
  planteamiento y aprobación que establece AGENTS.md.

Leer primero CONTINUIDAD_SESION.md y CANALES_COMPILACION.md. La estable de
escritorio publicada sigue siendo 1.5.0; no confundir el override Android 1.6.0
con un cambio de identidad o publicación de escritorio.
