# Android ARM64 de prueba

## Control remoto LAN

Ajustes → Control remoto conecta el APK a la PC con su dirección y clave de
sesión. Muestra la paleta activa y envía disparo/parada; el audio sale en la PC.
El cliente usa HTTP desde Rust, sin navegar el WebView a contenido remoto.
Alcance, seguridad y pruebas pendientes: [CONTROL_REMOTO_LAN.md](CONTROL_REMOTO_LAN.md).

## Funcionamiento local

El port comparte el proyecto Tauri, Rust y Vite con Windows y Linux. Android
usa el directorio privado de la aplicación, el selector de documentos nativo
y controles táctiles de pulsación larga. La consola mezcla efectos, panel y
reproductor en la salida predeterminada de Android. Se conservan los faders y
los controles de parada independientes. La preescucha está desactivada.

Los audios seleccionados se copian al almacenamiento privado. La importación
de carpetas y las funciones móviles de Mic y segundo plano están
pendientes. La integración local de OBS queda fuera de esta rama.

El flujo Android de GitHub Actions genera un APK ARM64 de depuración, verifica
su firma y adjunta el hash SHA-256. Los flujos de escritorio son independientes.
La prueba en teléfono debe cubrir importación y persistencia tras reiniciar,
desplazamiento y pulsación larga, mezcla y parada independiente, cambio de
salida del sistema y exportación de documentos.

El adaptador usa tauri-plugin-fs 2.5.1, ya presente por tauri-plugin-dialog,
para abrir documentos autorizados desde Rust. SHA-256 identifica copias de
audio por contenido mediante sha2, ya presente en Cargo.lock. Ambos son
compatibles con GPL-3.0-or-later y solo se declaran directamente para Android.

## Arranque nativo y prueba en emulador

El APK original se instalaba, pero Android no podía cargar la biblioteca JNI
porque Oboe dejaba símbolos del runtime C++ sin resolver. El enlazado de Android
incorpora libc++ y su ABI de forma estática a la única biblioteca JNI, y falla
si quedan símbolos sin resolver.

CPAL consulta Android mediante ndk-context 0.1.1, ya incluido transitivamente.
Se declara directamente solo para Android para inicializarlo con la VM de Tauri
y una referencia global a Application antes de crear el motor. Esta referencia
sigue siendo válida al recrear la actividad. La dependencia es MIT/Apache-2.0,
sin cambio de versión, descarga adicional ni permisos de Android.

Actions instala el APK ARM64 en Android 35 y comprueba dos arranques, actividad
en primer plano y presencia del WebView antes de adjuntar el binario.

## Importación y presentación en Android

El selector devuelve un URI. La copia privada abre su descriptor con lectura
explícita, ya que OpenOptions de tauri-plugin-fs parte sin modo de acceso.
Android usa el reloj del sistema y mantiene el transporte dentro del área
visible. Las categorías de Ajustes se desplazan horizontalmente y el contenido
tiene su propio desplazamiento.

El selector del sistema concede acceso al URI elegido, sin permiso general
de almacenamiento. El empaquetado Android aplica los iconos LF versionados
después de generar el proyecto Tauri, que trae sus propios iconos de ejemplo.

## Navegación táctil de Ajustes

En vertical y en ventanas estrechas, Ajustes muestra las secciones primero.
Tocar una sección abre su formulario. Secciones vuelve a la lista sin
descartar cambios, y Guardar usa el flujo compartido. En horizontal, desde
600 px, la lista y el formulario se muestran a la vez. Los estilos del
asistente y los diálogos reservan espacio para el teclado y las barras del
sistema. Las reglas de presentación se aplican solo a Android.

## Editor de pista con controles táctiles

En Android, un toque sobre la onda sitúa el cursor. Fijar inicio y Fijar fin
aplican el punto elegido, y un deslizamiento recorre la onda ampliada. Los
controles de volumen y zoom mantienen su deslizador y añaden botones de paso.
El zoom mantiene el cursor en pantalla y limita el ancho del búfer de dibujo.
Los comandos Rust de guardado y el esquema de datos se comparten con escritorio.
El editor móvil respeta la exclusión de la preescucha de esta edición.

## Menús y volumen con navegación del sistema

Los menús Android se limitan al viewport visible y a los márgenes de sistema
que el WebView expone. Si no expone ninguno, se reserva margen de seguridad
solo para los menús. El contenido que excede el espacio disponible se puede
desplazar. Se aplica al menú contextual, perfiles, pestañas,
selector de panel fijo y menús de volumen. El volumen maestro usa una fila
propia al abrirse, con el contador y Detener todo visibles.

## MIDI por USB y Bluetooth LE

En Ajustes → Atajos del teclado → MIDI, activa MIDI y abre Añadir dispositivo.
USB usa el servicio MIDI de Android y lista controladores compatibles con
el transporte MIDI 1.0, sin pedir acceso general al bus USB. El hardware debe
ser reconocido por Android en modo anfitrión USB. Bluetooth busca el servicio
BLE MIDI solo desde su botón. Android 12 y posteriores piden Dispositivos
cercanos en ese momento. En Android 11 y anteriores se pide ubicación para
la búsqueda BLE, como exige el sistema. Ver USB no solicita estos permisos.

El adaptador Kotlin mantiene las conexiones a los dispositivos elegidos y
entrega bytes por un canal privado de Tauri. Rust reconstruye mensajes
fragmentados y aplica la captura y las asignaciones existentes. Se conserva
la selección con el esquema MIDI actual. El servicio puede volver a abrir
un dispositivo BLE guardado si está disponible y conserva una dirección
reconocible. Dos dispositivos USB idénticos sin serie pueden requerir volver
a elegirlos tras reconectar.

Para compilar localmente, tras `npm run tauri -- android init` ejecuta
`python scripts/prepare-android-midi.py src-tauri/gen/android` antes de
`npm run tauri -- android build`. El script copia el adaptador versionado y
aplica los permisos y reglas de empaquetado al proyecto generado. GitHub
Actions ejecuta el mismo paso.
