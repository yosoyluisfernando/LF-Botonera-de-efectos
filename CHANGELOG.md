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
- Caché persistente de waveforms del editor de pistas en disco, con límites configurables de tamaño y antigüedad y opción para limpiarla desde los ajustes del normalizador.
- Progreso de análisis del editor de pistas con etapas visibles mientras Rust revisa caché, reconstruye waveform, decodifica, guarda y limpia.

### Cambiado
- El análisis del editor de pistas se ejecuta en un worker bloqueante de Tauri y reutiliza caché en memoria, `tracks.db` y waveforms persistidas antes de decodificar de nuevo.
- El recordatorio de donación deja de mostrarse seguido, ahora solo se muestra cada 100 aperturas de la botonera.
- El editor solo inserta PCM en la caché RAM si la precarga está activa y la duración del archivo entra en el límite configurado.
- El código interno fue reorganizado alrededor de un núcleo central y motores especializados. No es un cambio pensado para mejorar el rendimiento de forma directa; la app debería sentirse igual, pero el proyecto queda más ordenado y más fácil de entender para quienes quieran apoyar con mejoras en el futuro.

### Corregido
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
