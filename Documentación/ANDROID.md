# Android ARM64 de prueba

El port comparte el proyecto Tauri, Rust y Vite con Windows y Linux. Android
usa el directorio privado de la aplicación, el selector de documentos nativo
y controles táctiles de pulsación larga. La consola mezcla efectos, panel y
reproductor en la salida predeterminada de Android. Se conservan los faders y
los controles de parada independientes. La preescucha está desactivada.

Los audios seleccionados se copian al almacenamiento privado. La importación
de carpetas y las funciones móviles de Mic, MIDI y segundo plano están
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
