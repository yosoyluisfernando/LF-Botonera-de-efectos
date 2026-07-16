# Registro de cambios — LF Botonera de Efectos

Este archivo documenta los cambios relevantes de cada versión, siguiendo el estándar
[Keep a Changelog](https://keepachangelog.com/es/1.1.0/) y versionado semántico ([SemVer](https://semver.org/lang/es/)).

---

## Cómo mantener este archivo

**Categorías disponibles:** `Añadido` · `Cambiado` · `Corregido` · `Eliminado` · `Seguridad`

**Flujo de trabajo:**
1. Mientras desarrollas, anota los cambios en la sección `[Sin publicar]`.
2. Al publicar una versión: renombra `[Sin publicar]` a `[X.Y.Z] — YYYY-MM-DD`, crea una nueva sección `[Sin publicar]` vacía encima, y añade el enlace de comparación al pie.
3. Actualiza la versión con `SET-VERSION.bat X.Y.Z` antes del commit de release.

**Qué registrar:** funcionalidades nuevas, cambios de comportamiento, bugs corregidos, cosas eliminadas.

**Qué NO registrar:** refactorizaciones internas sin impacto en el usuario, commits de CI/CD, actualizaciones de documentación técnica, renombrado de variables.

---

## [Sin publicar]

### Añadido
- Panel lateral de botones fijos con alcance global o por perfil, posición izquierda/derecha, columnas configurables, DnD y controles de reproducción independientes.
- Panel fijo redimensionable, con una a cinco columnas y filas ilimitadas o capacidad limitada configurable hasta veinte filas.
- **Modo reproductor en el panel lateral:** una lista de reproducción propia, pensada para dejar música de fondo mientras se disparan los efectos. Tiene su propio motor de audio, así que suena de forma independiente: el Stop general y el Solo de los efectos no la cortan, y el reproductor tiene su propio botón de detener.
- Reproductor con tres modos de avance: normal, repetir la lista y aleatorio. El modo decide qué canción viene; que el reproductor se detenga al terminar la actual lo decide el botón **Detener al finalizar**, que se combina con cualquiera de los tres (por ejemplo: pararse en cada canción y que la siguiente salga al azar). La pista que suena se ve en verde y la marcada como siguiente en naranja, con los mismos colores que usa LF Automatizador. Lo que se marca como siguiente se respeta siempre, sin importar el modo, y sigue a su canción aunque se reordene la lista.
- El naranja indica siempre qué sonará al pulsar reproducir, también con el reproductor detenido: no desaparece al parar, y al añadir canciones a una lista vacía la primera queda marcada sola.
- En la lista del reproductor, un doble clic reproduce la canción si está detenido, o la marca como siguiente si ya hay música sonando, sin cortarla.
- Limpiar la lista o abrir otra **no corta la música**: la canción que está sonando termina, y al acabar entra la lista nueva.
- Botón **Detener al finalizar** en el reproductor: al terminar la pista actual la siguiente no arranca sola hasta pulsar reproducir, conservando la que estaba marcada como siguiente. No se recuerda al cerrar la aplicación: siempre arranca apagado.
- El reproductor tiene **volumen y dispositivo de salida propios**, configurables en Ajustes → Panel fijo, para poder mandar la música a otra tarjeta distinta de la de los efectos. El volumen también se puede ajustar desde el propio panel, sin entrar en Ajustes.
- Barra de progreso en el reproductor: muestra por dónde va la canción y permite adelantar o atrasar a un punto concreto. Las locuciones de hora y clima no se pueden adelantar, porque son varios archivos encadenados.
- Botón **Loop** en el reproductor: repite la canción actual hasta desactivarlo. Es distinto del modo Repetir, que repite la lista entera; por eso Repetir usa el icono ∞ y el Loop el 🔂. Ponerlo o quitarlo mientras suena no corta la música.
- El contador del reproductor cambia entre tiempo transcurrido y restante al pulsarlo, y con el reproductor parado muestra el tiempo total de la lista. La preferencia se recuerda.
- El modo de reproducción se puede cambiar desde el propio panel, con un menú donde el icono indica el modo activo.
- Guardar y abrir listas de reproducción en formato `.LFPlay`, **compatible con LF Automatizador**. Limpiar y abrir preguntan antes si se desea guardar la lista actual.
- En el reproductor se puede arrastrar una **carpeta entera** (incluidas sus subcarpetas) o **varios archivos a la vez**. Si son más de 250 canciones avisa antes, con opción de no volver a preguntar; esa decisión se puede cambiar después en Ajustes → Panel fijo. La carga va en segundo plano: la aplicación no se congela y la lista va creciendo a la vista. En la botonera de botones fijos se mantiene como estaba: un archivo por vez y sin carpetas.
- Con **Detener al finalizar** activo, la canción marcada como siguiente se ve en gris en vez de naranja: sigue marcada y se respeta, pero avisa de que no sonará sola.
- Arrastrar y soltar en la lista del reproductor: añadir canciones desde el explorador, arrastrar botones de la botonera a la lista, y reordenar las canciones arrastrándolas. Soltar sobre una fila inserta en esa posición; soltar en el espacio vacío añade al final.
- El reproductor admite los mismos tipos que los botones: audio, carpeta aleatoria, locución horaria, temperatura y humedad. La hora y el clima se resuelven en el momento de sonar, y respeta los cortes de inicio y fin marcados en el editor de pistas. Los tipos que no tienen una duración conocida de antemano se muestran como `--:--` y no cuentan para el total, igual que en LF Automatizador. Si alguno no se puede resolver (una carpeta vacía, o el clima sin conexión), se salta y la música continúa.
- **Cambiar el color de varios botones a la vez:** con **Ctrl + clic** se seleccionan los botones que se quieran (en la botonera o en el panel fijo) y, al hacer clic derecho, se ofrece pintarlos todos del mismo color. Ctrl+clic no dispara el sonido, y la selección se suelta con Escape o con un clic normal. Los botones nuevos se siguen creando con un color al azar.
- Caché persistente de waveforms del editor de pistas en disco, con límites configurables de tamaño y antigüedad y opción para limpiarla desde los ajustes del normalizador.
- Progreso de análisis del editor de pistas con etapas visibles mientras Rust revisa caché, reconstruye waveform, decodifica, guarda y limpia.

### Cambiado
- El análisis del editor de pistas se ejecuta en un worker bloqueante de Tauri y reutiliza caché en memoria, `tracks.db` y waveforms persistidas antes de decodificar de nuevo.
- El recordatorio de donación deja de mostrarse seguido, ahora solo se muestra cada 100 aperturas de la botonera.
- El editor solo inserta PCM en la caché RAM si la precarga está activa y la duración del archivo entra en el límite configurado.
- Reorganización interna del código fuente alrededor de un núcleo central y motores especializados. Al ser un proyecto de código abierto, este cambio deja una base más clara y ordenada para programadores que en el futuro quieran apoyar con mejoras o nuevas funciones. No está pensado como una mejora directa de rendimiento; la app debería sentirse igual, pero será más fácil mantenerla y ampliarla con seguridad.
- **La paleta de colores de los botones tiene ahora variedad real.** Antes eran 32 colores, pero en la práctica se veían 16: la mitad eran el mismo tono en otra intensidad, y la app iguala las intensidades para que el texto se lea en tema claro y oscuro. Además había seis azules y seis rojos, pero un solo verde. Ahora son 24 colores repartidos por todo el círculo de color, y ninguno se repite. Los botones que ya tienes conservan su color.
- El texto de los botones nuevos se lee mejor sobre los colores más vivos: antes se elegía blanco o negro con una regla fija que en algunos fondos acertaba mal; ahora se elige el que de verdad contrasta más. Los botones que ya tienes conservan sus colores.

### Cambiado
- **El volumen máster ahora también gobierna la música del reproductor.** Antes el reproductor era un motor completamente aparte y el máster no lo tocaba: bajarlo dejaba la música sonando igual de alta. Ahora todo lo que sale por la salida principal —botonera, panel fijo y música— responde al máster. El reproductor **sigue teniendo su propio volumen**, que es lo que se usa para bajar la música y hablar encima sin tocar los efectos. Lo que no cambia es el transporte: el Stop general y el Solo de los efectos siguen sin detener la música.
- **La música del reproductor ya aparece en el vúmetro.** Antes la aguja se quedaba plana con música sonando si no había ningún efecto, porque el reproductor iba por su cuenta. Ahora el vúmetro muestra todo lo que sale al aire.
- **Si el reproductor comparte salida con los efectos, ya no abre la tarjeta por segunda vez.** Antes cada uno abría la suya y era el sistema operativo quien las mezclaba, fuera del control de la aplicación. Ahora comparten la misma salida. Si le asignas una tarjeta propia al reproductor, sigue saliendo por ella y entonces no le afecta el máster: es una salida directa.

### Corregido
- **El vúmetro iba muy lento y no llegaba a marcar el nivel real cuando solo sonaba la música del reproductor.** La aguja tiene un decaimiento suave para bajar con elegancia cuando se acaba el sonido, pero se estaba aplicando en todos los ticks si no había ningún efecto sonando: cada actualización tardaba casi un segundo en llegar, así que con música de fondo la aguja siempre iba por detrás y nunca alcanzaba el volumen real. Ahora el decaimiento suave se usa solo cuando de verdad se acaba el sonido.

- **La pre-escucha se colaba en la salida principal si no tenías una tarjeta de sonido dedicada para ella.** Al no haber una segunda salida configurada —que es el caso por defecto—, la pre-escucha y la previa del editor acababan mezcladas con lo que sale al aire: les afectaba el volumen máster y movían el vúmetro como si fueran programa. Con una tarjeta dedicada no pasaba, así que el mismo botón se comportaba distinto según el equipo. Ahora la pre-escucha es siempre un canal aparte: aunque comparta altavoces con la salida principal porque solo tengas una tarjeta, no pasa por el máster ni cuenta en el vúmetro.

- **Algunas canciones no mostraban su duración** (y por eso no dejaban adelantar ni atrasar, ni mostraban el tiempo). El audio estaba bien: fallaba porque el archivo tenía el título o el artista guardados con una codificación inválida, algo habitual en MP3 antiguos, y eso tiraba la lectura entera. Ahora solo se lee la duración, sin las etiquetas. Lo que ya estuviera guardado sin duración —botones de la botonera, del panel fijo y canciones de la lista— la recupera al abrir la aplicación.
- Las pestañas nuevas se llaman ahora **Pestaña 2**, **Pestaña 3**… en vez de nombres extraños como "BOTONERA 1 4". El número sigue la posición, así que renombrar una pestaña no descoloca a las siguientes.
- **Adelantar o atrasar un audio ya no deja silencios.** Saltar a un punto lejano de una canción podía tardar varios segundos (más de seis al ir al minuto dos de un tema largo), porque el salto no era real y había que recorrer el audio por dentro hasta llegar. Ahora es inmediato sin importar la distancia, y afecta tanto al reproductor como a la barra de progreso de la botonera principal. Los efectos cortos no estaban afectados porque ya se cargan en memoria.
- El editor de pistas evita congelamientos al analizar audios largos y reabre más rápido archivos ya analizados.

---

## [1.1.3] — 2026-06-28

### Añadido
- **Fundidos globales (Fade In / Fade Out):** configurables en segundos desde Ajustes → Principal. Valores independientes para fade-in al iniciar, fade-out al detener y fade-out al terminar naturalmente. Se aplican a todos los botones.
- **Modo y objetivo de normalización configurable:** botón ⚙ en el editor de pistas permite elegir entre LUFS (volumen percibido) o Pico (dBFS) con valor objetivo y techo de pico personalizables. La configuración es global.
- **Detección automática de cue:** el editor de pistas puede detectar silencio inicial y final para proponer puntos de inicio y fin al abrir un audio. Incluye interruptores globales para activar la detección completa, solo inicio o solo fin, y umbrales independientes en dBFS.
- **Barra de progreso opcional para reproducción principal:** configurable desde Ajustes → Reproducción, con retroceso/avance por 1, 2, 5, 10, 20 o 30 segundos y seek directo sobre el último audio disparado desde los botones.
- Aviso de primera apertura del editor de pistas para presentar los ajustes de normalización y detección de cue, con opción de no volver a mostrarlo.
- Modal **Qué hay de nuevo** al abrir una versión instalada por primera vez, usando el changelog local de la aplicación.

### Cambiado
- Ajustes generales reordenado: Principal, Reproducción, Precarga, Hora y Clima, Atajos del Teclado, Acerca de. Los fundidos globales ahora viven en Reproducción.
- El normalizador automático ahora respeta el modo configurado por el usuario (LUFS/Peak) en lugar de usar siempre −14 LUFS.
- El botón **Normalizar** del editor recalcula la ganancia con la configuración global actual sin volver a decodificar el archivo.
- `stop_audio` y `stop_all_audio` aplican fade-out al detener si está configurado; si no, corte inmediato (comportamiento anterior).

### Corregido
- La barra de progreso ya no bloquea durante varios segundos al adelantar en canciones largas no precargadas; el backend usa seek real del decodificador cuando el formato lo permite.
- El modal de ajustes del normalizador vuelve a mostrarse con fondo, cabecera y botones consistentes con el resto de modales.

---

## [1.1.2] — 2026-06-27

### Añadido
- Ventana flotante para el editor de pistas: se puede sacar como ventana independiente y moverla o minimizarla sin cerrar la app principal.
- El editor recuerda si fue abierto en modo modal o ventana flotante (`editor_mode` en configuración).

### Corregido
- El normalizador LUFS ahora aplica la ganancia correctamente al reproducir desde el editor.
- Se eliminó el re-análisis innecesario al pasar el editor de modal a ventana flotante.

---

## [1.1.1] — 2026-06-27

### Añadido
- **Editor de pistas:** forma de onda en canvas (envolvente estilo Adobe Audition), punto de inicio (cue), punto de fin opcional, zoom 1×–30× con Ctrl+Rueda, cursor de reproducción animado a 60 fps.
- **Normalizador automático:** objetivo −14 LUFS con techo de pico a −1 dBFS, activable por archivo. Ajuste manual de ganancia en dB adicional.
- **Precarga de audio en RAM:** caché LRU configurable (32–256 MB) con estrategias FullProfile, VisibleTabs y OnPlay; TTL configurable; seek O(1) para archivos cacheados.
- **Salida de pre-escucha independiente:** segundo dispositivo de audio para escuchar el efecto antes de emitirlo al aire.
- Seek por clic en la barra de pre-escucha.
- Los exports `.bdelf`/`.bdeplf` incluyen opcionalmente cue y ganancia del archivo, que se restauran al importar en otro equipo.
- Traducciones actualizadas al inglés, portugués (Brasil) y portugués (Portugal).

---

## [1.1.0] — 2026-06-24

### Cambiado
- Refactorización interna: módulos de tipos y comandos de perfil divididos en archivos más pequeños para facilitar el mantenimiento.

---

## [1.0.4] — 2026-06-17

### Corregido
- Recompilación para resolver un falso positivo de Windows Defender en el instalador NSIS generado por GitHub Actions (el código fuente y el MSI no estaban afectados).

---

## [1.0.3] — 2026-06-17

### Añadido
- Reordenar pestañas arrastrándolas.
- Mover botones entre pestañas con Alt + arrastre.
- Workflow de CI/CD en GitHub Actions para compilación y publicación automática en Windows y Linux.
- Dependencias de audio para compilación nativa en Linux (`libasound2-dev`).

### Cambiado
- Refinamiento visual de estados activos y hover en la rejilla de botones.
- Mejoras en la apariencia de la barra inferior.

### Corregido
- El color del perfil se conserva correctamente al editarlo.
- Redimensionamiento de la rejilla al cambiar filas o columnas.
- Recuperación del estado de modales en escenarios de error.

---

## [1.0.2] — 2026-06-16

### Añadido
- Enlaces al canal y grupo de la comunidad en Telegram.

---

## [1.0.1] — 2026-06-15

### Cambiado
- Mejoras de interfaz en el arranque de la aplicación.

### Añadido
- Verificación de actualizaciones disponibles al iniciar.

---

## [1.0.0] — 2026-06-13

### Añadido
- Botonera de efectos de sonido para radio y streaming en vivo.
- Perfiles ilimitados con configuración de audio independiente por perfil.
- Pestañas (paletas) con cuadrículas de filas y columnas configurables.
- Botones con colores personalizables, etiquetas y volumen individual.
- Modos de reproducción por botón: loop, superposición (overlap), reiniciar, detener otros.
- Atajos de teclado locales y atajos globales del sistema operativo.
- Modo de mapeo visual: muestra los atajos asignados sobre la rejilla.
- Arrastrar y soltar archivos de audio desde el explorador.
- Modo solo global: detiene todos los sonidos al reproducir uno nuevo.
- Locuciones de hora y clima con archivos de audio configurables.
- Botón de carpeta secuencial: reproduce archivos de una carpeta en orden.
- Exportar e importar pestañas (`.bdelf`) y perfiles completos (`.bdeplf`).
- Compatibilidad bidireccional con LF Automatizador.
- Tema claro, oscuro y automático según el sistema operativo.
- Cuatro idiomas: español, inglés, portugués (Brasil), portugués (Portugal).
- Asistente de primer arranque (wizard).
- Vúmetro estéreo L/R en tiempo real en la barra inferior.
- Reloj, fecha y contador regresivo en la barra inferior.
- Compilación para Windows (`.exe`, `.msi`) y Linux (`.deb`, `.rpm`, `.AppImage`).

---

[Sin publicar]: https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/compare/v1.1.3...HEAD
[1.1.3]: https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/compare/v1.1.2...v1.1.3
[1.1.2]: https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/compare/v1.1.1...v1.1.2
[1.1.1]: https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/compare/v1.0.4...v1.1.0
[1.0.4]: https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/compare/v1.0.2...v1.0.3
[1.0.2]: https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/releases/tag/v1.0.0
