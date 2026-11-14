# Conversión auxiliar de audio con FFmpeg

## Alcance de la primera fase

Integración para desarrollo en Windows y Linux, basada en la aportación de
Danilo Wenner a LF Botonera Radio 1.5.1. No modifica las tarjetas, buses,
faders, Mic ni el transporte de audio existente. Android sigue usando la
ruta nativa; no se instala ni ejecuta un binario de Windows en Android.

La base de detección M4A/MP4/AAC/M4B/3GP proviene de `probe.rs` y sus pruebas del
commit `7c1f1ae484cdb4bd0110b4761b8a221023bdea6f`. Se corrigió la detección
de extensiones SBR desactivadas para no convertir AAC-LC innecesariamente.
Cola, caché, avisos e
integración se adaptan al proyecto oficial. La copia auxiliar usa FLAC en lugar
de MP3: evita una compresión adicional con pérdida, sin recuperar información
que el original ya hubiera perdido.

## Dependencia justificada

FFmpeg permite decodificar variantes que Rodio/Symphonia no resuelven
completamente, incluido HE-AAC. Se usa como proceso externo, sin nuevas crates
ni dependencias npm. El proyecto mantiene FFmpeg en https://ffmpeg.org/;
la compilación local preparada es Gyan 9.0.2 essentials, GPLv3, compatible con
la licencia de esta aplicación. Su ejecutable ocupa aproximadamente 105 MB.
La licencia concreta depende de las opciones del build, no solo del nombre.

El proceso recibe rutas como argumentos, sin shell; restringe los protocolos
de entrada a archivos, usa la primera pista de audio, no procesa vídeo y no
abre ventana de consola en Windows. Tiene límite de cinco minutos por trabajo
y elimina copias parciales en errores. Los límites de formatos y extensiones
se validan en Rust; no se amplía el permiso de ejecución del frontend.

## Desarrollo local con DEV.bat

En este equipo el worktree está en
`C:/OVERLAY/BOTONERA/.worktrees/variante-danilo`.
Ejecutar su `DEV.bat`, no el de la carpeta principal (que conserva desarrollo).
No hace falta compilar en GitHub, instalar otra app ni crear instaladores.

Se preparó una copia local de FFmpeg ya instalado en LF Botonera Radio:
`src-tauri/binaries/ffmpeg.exe`, con licencia y origen/SHA-256 en esa carpeta.
Está excluida de Git. El SHA-256 local es
`3256173f3f8bffd7df12227c68adf68025edb1832273a9530688a7bb1ed8edec`.

Para preparar otra copia local desde PowerShell en la raíz del worktree:

```powershell
./scripts/prepare-ffmpeg.ps1 -SourcePath C:/ruta/ffmpeg.exe -LicensePath C:/ruta/LICENSE
```

Se puede añadir `-ExpectedSha256` con la huella verificada del ejecutable.
También se admite `LF_FFMPEG_PATH` con una ruta absoluta. En desarrollo se
busca después en `src-tauri/binaries`; en ejecución se considera la carpeta
del ejecutable y los directorios de PATH. Reiniciar la app tras instalar o
cambiar el conversor. Linux puede usar el FFmpeg de la distribución en PATH.
No se descarga software automáticamente durante el arranque de la aplicación.

## Funcionamiento y persistencia

- No hay una conversión manual: la asignación o consulta prepara el audio.
  El modal de seguimiento se puede ocultar sin cancelar. Su acceso flotante
  permite reabrirlo; los botones de rejilla y panel fijo mantienen el estado.
- Rust calcula el porcentaje con `out_time_us` de `ffmpeg -progress pipe:1`
  y la duración disponible. Se limita a 99 hasta validar/publicar la copia;
  solo entonces se emite 100. Si falta duración, muestra preparación sin
  inventar un porcentaje. La UI nunca calcula ese progreso.
- `get_audio_conversions` hidrata eventos perdidos al arrancar; cada snapshot
  trae una revisión para no sustituir eventos nuevos por resultados viejos.
  El modal oculto no reconstruye su lista durante cada actualización.
- Una única cola hace la detección y conversión en segundo plano. Las peticiones
  concurrentes de una misma ruta y sello comparten un solo trabajo.
- Si el decodificador nativo resuelve el formato, se usa el original.
  MP4/M4A/AAC/M4B/3GP y las copias FLAC convertidas pasan por `SeekSource`
  con File completo, evitando el
  `unreachable!` de Rodio 0.19 ante SeekError de inicialización. Se selecciona
  una pista decodificable de audio, no el vídeo. Canales y frecuencia se toman
  del primer bloque real, porque la cabecera MP4 puede estar incompleta.
  HE-AAC/HE-AAC v2 siguen usando FFmpeg; no se añaden nuevas dependencias.
- Si necesita FFmpeg, muestra un aviso de preparación. Un disparo pendiente
  devuelve `audio_converting`; no detiene los otros efectos por Solo/Stop others.
- No se reproduce automáticamente al terminar la conversión. Reintentar el
  disparo cuando la preparación termine. La cola del reproductor se refresca
  conservando sus identificadores y el audio activo.
- Los botones, exportaciones y `tracks.db` conservan la ruta del original.
  Playback, seek, duración y análisis resuelven la misma copia FLAC.
- El análisis espera en su worker, antes de consultar cachés. Las ondas convertidas
  usan un namespace distinto de las antiguas; los cues y ganancias manuales se
  mantienen al actualizar las mediciones del archivo original.
- Caché: directorio de datos de la app, subcarpeta `converted-flac-v1`.
  La clave usa SHA-256 de la ruta normalizada, tamaño y mtime con nanosegundos.
  Un original cambiado genera otra copia; un cambio durante el trabajo invalida
  el resultado. Copias incompletas nunca se publican como válidas.
- Tras convertir se retiran copias FLAC antiguas (30 días) o por presupuesto de
  512 MiB, empezando por las más viejas. La copia recién creada se conserva,
  aunque ella sola supere el presupuesto. Archivos abiertos que el sistema
  impida retirar quedan para una limpieza posterior; no se toca el original.
- Los fallos se recuerdan durante la sesión y se notifican sin reintentos en bucle.
  Corregir el archivo (cambia su sello) o reiniciar tras corregir el conversor.

## Distribución futura

Esta fase NO cambia los workflows ni incluye FFmpeg en los instaladores.
Antes de distribuirlo junto a la app hay que fijar build y hashes por sistema,
incluir licencia, configuración de compilación y código fuente correspondiente
(también de dependencias enlazadas), y definir los recursos del empaquetado.
Un enlace genérico al código más reciente no sustituye esa correspondencia.
La edición sin conversor continúa funcionando con los formatos nativos.

Referencias: https://ffmpeg.org/legal.html y https://www.gyan.dev/ffmpeg/builds/.

## Verificación sin abrir la botonera

Las pruebas opcionales de integración generan audios sintéticos y comprueban
la conversión, el camino nativo, seek/análisis, conservación del original y
rechazo de audio dañado. Ejecutar con `LF_TEST_FFMPEG` y `LF_FFMPEG_PATH`
apuntando al mismo FFmpeg; redirigir APPDATA a un perfil descartable de tests:

```text
cargo test --lib engine::audio::transcode::integration_tests -- --ignored --test-threads=1
```

Las pruebas unitarias normales no requieren FFmpeg. Las pruebas opcionales adicionales reciben `LF_TEST_CONVERSION_FILES`, una lista
separada por punto y coma de muestras reales. Reproducen en buses de memoria,
sin abrir tarjetas; otra prueba genera un MP4 con vídeo y AAC-LC y comprueba
que solo se lee audio nativo, con seek y especificación correctos.
La prueba funcional con DEV.bat y hardware corresponde al autor.
