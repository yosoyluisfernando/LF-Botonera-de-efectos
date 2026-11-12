# Continuidad de escritorio después de Android estable

Estado actualizado con el autor el 2026-11-11. Repositorio C:/OVERLAY/BOTONERA,
rama codex/desarrollo. Android 1.6.0/1006001 está cerrado para su primera entrega.
No recompilar Android ni generar instaladores de escritorio ahora. Las pruebas
manuales se harán desde dev.bat. No ejecutar emuladores ni compilar por iniciativa
propia; el autor pidió reducir ese consumo y probar personalmente.

## Confirmado por el autor: ya probado

- Editor de pistas en pantallas pequeñas y con escalado.
- Predeterminado y Moderno, con temas claro y oscuro.
- OBS y Control remoto ante interrupciones y errores de conexión.

No volver a presentar estos puntos como pruebas pendientes.

## Ajustes aplicados para probar desde dev.bat

- MIDI en PC: guía de vinculación desde Bluetooth del sistema y Actualizar MIDI
  como flujo principal. Conexión BLE directa conservada como alternativa plegada,
  porque WinMM/ALSA no garantizan que un dispositivo vinculado aparezca como MIDI.
  En el editor (sonidos, fijos y OBS), MIDI queda oculto sin entrada MIDI activa;
  teclado siempre disponible. No se borran asignaciones al ocultar. Android conserva
  su presentación. Comprobar con hardware antes de retirar el adaptador BLE directo.
- Última revisión del autor: remoto usa una casilla (marcar/configurar/Guardar
  para activar, desmarcar para detener). Aviso corto sin duplicados. OBS, MIDI
  y avance de Reproducción usan controles atenuados y bloqueados como Precarga.
  Modo Mapeo queda primero y etiquetas debajo. Sustituye la presentación anterior.
- Próximo DEV.bat preparado para mostrar las novedades una vez; para repetir,
  usar DEV.bat novedades. Las notas describen funciones nuevas, sin correcciones
  del desarrollo de estas mismas funciones. 34 pruebas Node/DOM aprobadas.
- Acerca de queda después de OBS y Control remoto también en PC.
- Las opciones de etiquetas de teclado/MIDI aparecen al principio de Atajos del
  teclado en PC, independientes de activar MIDI. Ambas visibles por defecto;
  guardar o volver a cargar preferencias aplica la visibilidad sin borrar atajos.
- El visor compartido une continuaciones del Markdown. Pruebas de notas reales
  Android y PC a 390/768/1366px. El bloque histórico Android alfa se movió a su
  changelog propio; la sección pública Android 1.6.0 conserva su contenido.
- Control remoto recupera los controles tras fallar la acción y la consulta de
  estado. OBS presenta un mensaje traducido también para un error desconocido.
- Acerca de muestra información no disponible si falla IPC, en vez de atribuir
  falsamente el canal a GitHub. No se cambió el mecanismo de selección del canal.
- Control remoto PC continúa apagado en cada arranque: no se crea el hilo/puerto
  de escucha hasta activar y Desactivar lo cierra e invalida la clave. Aviso HTTP
  sin cifrado siempre visible al final del apartado. El cliente remoto pasa a
  optativo por defecto (configuraciones nuevas o sin el campo); se respeta una
  elección guardada explícitamente. Al apagar el cliente se corta el sondeo y
  se cierra el diálogo inmediatamente. Sin recompilar el APK ya entregado.

Validación: siete pruebas Node/DOM aprobadas, sintaxis JS, i18n y diff correctos.
Después del ajuste remoto: tres pruebas dirigidas aprobadas, incluida ausencia
de solicitudes estando apagado y cancelación de sondeo al desactivar. Se añadió
regresión Rust de valores predeterminados sin ejecutarla para respetar no compilar.
Sin cargo test/build, npm build, GitHub Actions ni nuevos instaladores en este turno.

## Bluetooth MIDI: verificación del código para PC

Implementado para Windows y Linux: input/mod.rs incluye midi_ble y
midi_ble_connection en ambas plataformas; Cargo.toml usa btleplug para ambas.
midi_ble.rs obtiene properties.local_name, guarda dispositivos por dirección y
reutiliza preferred_name de midi_ports.rs. cmd_midi.rs conserva nombres previos
al guardar la selección. Si nunca hay nombre anunciado ni conocido, se muestra
el identificador; no se puede inventar el nombre del controlador.
Esta revisión del código no sustituye la prueba física ni certifica drivers.

## Pendientes para esta nueva versión

- Pruebas físicas del autor de MIDI/Bluetooth y reconexión en Windows/Linux,
  incluidos paquetes DEB, RPM y AppImage cuando se generen.
- Comprobaciones físicas documentadas de Mic/captura compartida, atajos locales
  y globales, panel fijo tras reiniciar y arrastre entre ventanas.
- Prueba manual de los ajustes de este turno desde dev.bat.
- Empaquetar Windows/Linux solo cuando lo solicite el autor. Escritorio sigue
  en 1.6.0-beta.5; main conserva la estable publicada 1.5.0.

Las futuras reorganizaciones de módulos, IPC o interfaz requieren el planteamiento
previo que indica AGENTS.md. No generar HTML informativo para comunicar pendientes.
