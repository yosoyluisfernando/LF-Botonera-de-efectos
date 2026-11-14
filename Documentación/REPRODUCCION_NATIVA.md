# Reproducción nativa de contenedores de audio

## Estado vigente — 2026-11-13

Por decisión del autor, la botonera no convierte archivos mediante FFmpeg.
No conserva el módulo, procesos, caché de conversión, IPC/eventos, modal,
porcentajes, scripts de preparación ni el binario local de desarrollo.
Los cambios históricos de conversión permanecen únicamente en el historial Git.

Se mantienen las aportaciones seleccionadas de Danilo Wenner: rueda sobre las
pestañas, barra inferior ocultable y la base del detector de perfiles AAC,
adaptadas a la aplicación oficial. El crédito está en README, changelog y avisos.

## Formatos y validación

La lista central `engine/audio/formats.rs` admite MP4/M4B/3GP junto a los formatos
anteriores, incluido M4A/AAC. Una extensión admitida no garantiza que cualquier
códec dentro del archivo sea compatible. MP4 con vídeo utiliza solo audio.
Ajustes → Acerca de y README muestran MP4 entre los formatos compatibles.

`native_probe.rs` valida los perfiles MP4/AAC que el decodificador puede reproducir
completamente. AAC-LC y ALAC compatibles se leen directamente. HE-AAC/SBR/PS y
otros perfiles no admitidos devuelven `unsupported_audio_format`; no se reproduce
únicamente su núcleo ni existe una conversión de respaldo. El detector conserva
su criterio prudente para AAC de baja frecuencia, que puede indicar SBR implícito.

## Apertura y análisis

`decode.rs` utiliza `SeekSource` con un File completo para MP4/M4A/AAC/M4B/3GP.
Así mantiene tamaño y búsqueda, evitando el `unreachable!` de Rodio 0.19 ante
SeekError de inicialización. No se captura ni silencia ese panic como solución.
El lector selecciona una pista de audio decodificable y obtiene su frecuencia y
canales del primer bloque real, porque la cabecera puede estar incompleta.

`native_media.rs` permite leer AAC-LC ADTS MPEG-2 con el lector existente:
indexa las cabeceras y adapta solo el bit ID durante la lectura, conservando
longitud, búsqueda y todos los bytes del audio comprimido. Nunca escribe en el
original ni lanza procesos. Una prueba verifica además que una secuencia igual
al sincronismo dentro del payload no se modifica y que el seek aleatorio funciona.

El resto de formatos conserva el flujo previo. Precarga, reproductor, editor,
seek, cue y ganancias siguen usando las puertas de decodificación existentes.
El análisis y la reproducción comparten la validación nativa para evitar medir
un perfil que después no se pueda reproducir correctamente. No cambian modelos
persistentes, base de datos, formatos de intercambio ni ajustes del usuario.

## Verificación

Las pruebas unitarias no requieren programas externos. Las opcionales leen
archivos proporcionados por variables de entorno y reproducen en buses de memoria,
sin abrir tarjetas ni iniciar la aplicación:

- `LF_TEST_NATIVE_FILES`: rutas compatibles separadas por punto y coma; comprueba
  reproducción, seek, loop, análisis y conservación del original.
- `LF_TEST_NATIVE_VIDEO`: MP4 con vídeo y AAC-LC mono/44,1 kHz; comprueba que se
  obtienen solo las muestras de audio y su especificación correcta.
- `LF_TEST_UNSUPPORTED_AUDIO`: perfil HE-AAC no admitido; comprueba que no se
  reproduce ni analiza parcialmente.

Ejecutar `cargo test --lib native_ -- --include-ignored --test-threads=1` con esas
variables. La verificación general sigue siendo `cargo test --lib`,
`cargo build --lib` y `npm run build`, sin iniciar la app ni crear instaladores.
El autor realiza la prueba funcional con el DEV.bat del worktree variante-danilo.
