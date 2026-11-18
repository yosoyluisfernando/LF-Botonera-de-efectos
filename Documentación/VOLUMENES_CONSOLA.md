# Volúmenes de consola y controles compartidos — 2026-11-18

## Alcance autorizado

El autor confirmó salidas por pestaña y continuidad. Solicitó que los faders de
canal solo atenúen de 0 a 100 %, con el máster como único control que puede superar
100 % al habilitar expresamente el recorrido hasta 150 %. El volumen y los límites
deben coincidir visualmente en consola, barra inferior y reproductor, incluidos
modo modal, consola acoplada, ventana independiente y audio en pausa/reposo.
Trabajo exclusivamente local en codex/desarrollo, sin commits ni push.

## Corrección de ganancia

El reproductor ya compartía el fader de BusId::Reproductor con su control de panel,
pero la consola, el IPC y el motor permitían 1.5 mientras el panel solo mostraba
100 %. No había una segunda etapa: el problema era ese recorrido adicional.

Ahora Reproductor, Efectos, Panel, CUE y Salida(n) tienen techo lineal 1.0.
ConsoleEngine::set_fader aplica ese límite a todos los canales. PlayerEngine y
player_set_volume usan 0.0..=1.0; get_console_view anuncia max=1.0 al reproductor.
PlayerConfig y ConsoleConfig normalizan al deserializar los valores antiguos
por encima de 1.0, también los faders por dispositivo. Se conservan nombres,
rutas, cola y demás preferencias. El formato no gana campos obligatorios.

Programa sigue controlado por set_master_volume y sus opciones de perfil: máximo
1.0 o 1.5 según master_volume_boost. Deshabilitar boost reduce también el volumen
actual si supera 1.0 mediante una transición de 500 ms en Rust. Los valores
menores o iguales a 1.0 se conservan exactamente, también al activar recordar. El máster compartido de los destinos de pestaña mantiene su
etapa única. No se alteran cue, normalización ni ganancia editorial de las pistas.
No se añade un control de gain a la consola.

## Transición y guía visual

MasterRamp comparte un reloj monotónico entre Programa y las salidas de pestaña.
FaderSource consulta la envolvente por cuadro estéreo, sin bloquear el hilo de audio
ni lanzar temporizadores por tarjeta. El objetivo visible se limita inmediatamente
a 100 %; la señal efectiva baja linealmente durante 500 ms. Una orden manual de
máster o restauración de perfil cancela la transición. CUE queda independiente.
No se crean fuentes ni se reconstruye el grafo para esta operación. La atenuación
del Mic calcula su techo contra ese máster efectivo durante la transición.

consoleFaders.js muestra un recuadro porcentual junto al capuchón durante el
arrastre y los ajustes de teclado de cualquier canal, incluidos los destinos de
pestaña. Se oculta 400 ms después de terminar; no captura eventos ni añade fondo
sobre la aplicación. Usa el valor local durante el gesto para evitar respuestas
atrasadas. El carril del máster distingue el tramo superior al 100 % con el mismo
color de ampliación del control inferior y una marca fija de unidad. Todos los
colores proceden de variables de tema. La información accesible se actualiza igual.

## Estado compartido y contratos

- get_volume_controls devuelve {master, player_volume, faders}.
- master conserva {volume, remember, boost, max}.
- faders es un mapa por bus (programa, efectos, panel, reproductor, cue, salida0…7)
  con {value, max} calculados por Rust desde los mismos atómicos del motor.
- volume-controls-changed emite ese estado para todas las ventanas cuando cambia.
- engine/console/controls.rs observa los controles cada 50 ms en un hilo propio,
  con taps creados una vez. Sigue observando en reposo, independientemente de
  meter-tick y player-tick. No guarda ni recalcula audio y solo emite diferencias.
- MasterVolumeState se comparte entre los comandos existentes y esta lectura;
  los setters públicos anteriores conservan sus nombres y sus respuestas.

volumeSync.js registra las vistas, escucha el evento y hace una lectura inicial.
Descarta una respuesta de lectura si durante la espera recibió un evento más
reciente. Consola y controles de panel se actualizan sin disparar nuevos setters.
El volumen del reproductor ya no se repinta desde player-tick: tiene una única
fuente de sincronización. Redibujar la cola pide el estado común actual.

volumeSlider.js conserva el arrastre local, serializa/coalesce los movimientos
pendientes y confirma el estado al terminar. Los faders ajenos siguen actualizándose.
Se permite que un control enfocado se actualice cuando no se está arrastrando;
no se confunde foco con gesto. consoleFaders.js actualiza valores, límites, tooltip
porcentual y aria-valuetext sin reconstruir las tiras ni los medidores.
JS solo interpreta gestos, representa porcentajes y envía comandos; los límites,
el volumen efectivo y la aplicación de ganancia permanecen en Rust.

## Verificación

492 pruebas Rust aprobadas, 23 omitidas; pruebas nuevas con muestras en memoria
confirman ganancia unitaria, techo 100 % en todos los canales, máster como única
amplificación, normalización de configuraciones anteriores y estado compartido
leído desde el volumen real. cargo build --lib y npm run build completados.

14 pruebas DOM/consola aprobadas. Incluyen panel → consola y consola → panel,
dos documentos independientes, pausa sin ticks de audio, 140 % con boost,
deshabilitación de boost, porcentajes del reproductor, arrastre y lectura IPC
atrasada. Temas claro/oscuro y regresiones de salidas/consola cubiertos.
Sin abrir la aplicación: la prueba auditiva y nativa final pertenece al autor.

## Prueba del autor con DEV.bat

1. Reiniciar DEV.bat de C:/OVERLAY/BOTONERA.
2. Mover el volumen del reproductor a 35 % y comprobar su fader de consola;
   moverlo desde la consola y revisar deslizador, icono y porcentaje del panel.
3. Repetir con el máster inferior y el fader Máster de consola.
4. Permitir 150 % desde el menú de VOL: la consola debe ampliar el recorrido.
   Llevarlo a 140 % y comprobar ambos controles. Desactivar el permiso:
   ambos deben volver como máximo a 100 % y compartir ese límite. El sonido baja
   suavemente en medio segundo; repetir al 25 %, 90 % y 100 % para comprobar que
   esos valores no cambian. Mover el máster durante la transición debe prevalecer.
   Revisar recuadro porcentual de todos los canales y marca del 100 % en el máster.
5. Repetir en pausa, detenido, consola acoplada y ventana independiente.
6. A volumen 100 % de reproductor, la consola no debe añadir amplificación.
   Efectos, fijos, preescucha y destinos de pestaña también deben limitarse a 100 %.
