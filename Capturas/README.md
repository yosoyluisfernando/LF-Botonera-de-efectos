# Biblioteca de capturas

Esta carpeta conserva las capturas maestras de LF Botonera de Efectos. Las imágenes
actuales corresponden a Windows, interfaz en español y compilación Release. Todas son
PNG de `1440×860`; se reorganizaron y renombraron sin recomprimirlas.

La biblioteca sirve como fuente para GitHub, Microsoft Store, redes sociales, una
futura web y el manual. Una imagen puede servir a varios canales: no se deben guardar
copias idénticas en carpetas distintas.

## Estructura y nombres

La ruta sigue este patrón:

`plataforma/idioma/area/NN-descripcion-breve.png`

- `windows/es/`: material actual.
- `principal/`: rejilla, edición de botones y menús.
- `panel/`: botones fijos, reproductor y preescucha.
- `editor/`: forma de onda, cues y normalización.
- `configuracion/`: páginas de ajustes.
- `consola/`: mezclador y medidores.

Los nombres usan minúsculas, números, guiones y caracteres ASCII para evitar problemas
al reutilizarlos en Windows, Linux, URLs o scripts.

## Selección recomendada para publicación

Estas son las capturas más limpias y representativas:

1. [`01-rejilla-modo-claro.png`](windows/es/principal/01-rejilla-modo-claro.png)
   — vista general de la rejilla 5×5, los modos y los medidores. Es la imagen principal.
2. [`02-rejilla-modo-oscuro.png`](windows/es/principal/02-rejilla-modo-oscuro.png)
   — demuestra el tema oscuro sin cambiar la organización de trabajo.
3. [`01-botones-fijos.png`](windows/es/panel/01-botones-fijos.png)
   — muestra el panel lateral de acceso permanente.
4. [`01-forma-de-onda-y-cues.png`](windows/es/editor/01-forma-de-onda-y-cues.png)
   — presenta el editor, la forma de onda, los cues, la ganancia y la normalización.
5. [`01-consola-audio-modo-oscuro.png`](windows/es/consola/01-consola-audio-modo-oscuro.png)
   — enseña los buses, faders y vúmetros. No debe ser la imagen principal porque la
   propia interfaz informa que la consola sigue en fase de prueba.

Para Microsoft Store ya existe una captura enviada. Esta selección permite mejorar la
galería en una actualización futura, después de la aprobación inicial.

## Catálogo para manual y documentación

### Interfaz principal

- [`03-tipos-de-boton.png`](windows/es/principal/03-tipos-de-boton.png): selector de
  audio, carpeta aleatoria, hora, temperatura y humedad. Apta para el manual.
- [`04-menu-contextual-boton.png`](windows/es/principal/04-menu-contextual-boton.png):
  bucle, superposición, reinicio, detener otros, preescucha y editor. Apta para el manual.

### Panel lateral

- [`02-reproductor-auxiliar-cargando.png`](windows/es/panel/02-reproductor-auxiliar-cargando.png):
  cola y controles del reproductor. Fue tomada durante una importación; conviene repetirla
  en reposo antes de usarla como material promocional.
- [`03-escucha-previa.png`](windows/es/panel/03-escucha-previa.png): ventana de
  preescucha con progreso, posición y volumen. Apta como imagen secundaria o de manual.

### Editor de pistas

- [`02-ajustes-normalizacion-y-cue.png`](windows/es/editor/02-ajustes-normalizacion-y-cue.png):
  modo LUFS, techo de pico, detección de silencio y caché. Los valores visibles son una
  configuración de ejemplo, no una recomendación general.

### Configuración

- [`01-principal.png`](windows/es/configuracion/01-principal.png): tema, idioma, tamaño
  de texto, botones superiores y salidas de audio.
- [`02-reproduccion.png`](windows/es/configuracion/02-reproduccion.png): barra de
  progreso, salto y fundidos.
- [`03-panel-fijo.png`](windows/es/configuracion/03-panel-fijo.png): alcance, posición,
  presentación, columnas, filas y controles.
- [`04-precarga.png`](windows/es/configuracion/04-precarga.png): memoria, duración máxima
  y alcance de la precarga.
- [`06-atajos-de-teclado.png`](windows/es/configuracion/06-atajos-de-teclado.png): modo
  de mapeo, atajos globales y combinaciones reservadas.

## Casos especiales

- [`05-hora-y-clima-aragua-de-barcelona.png`](windows/es/configuracion/05-hora-y-clima-aragua-de-barcelona.png)
  muestra deliberadamente Aragua de Barcelona, Anzoátegui, Venezuela, ciudad de origen
  del autor. Su aparición pública está autorizada y forma parte de la identidad humana
  del proyecto. La ruta visible no contiene un nombre de usuario ni una credencial. Es
  apta para GitHub, el manual, la web o una publicación sobre las locuciones de clima;
  no se recomienda como primera imagen comercial porque requiere contexto.
- [`07-acerca-de-version-1.2.0.png`](windows/es/configuracion/07-acerca-de-version-1.2.0.png)
  muestra una versión anterior. Solo sirve como registro histórico hasta repetirla con
  la versión vigente.

## Reglas para capturas nuevas

1. Usar siempre una compilación Release y la versión que se va a publicar.
2. Utilizar perfiles y audios de demostración con nombres genéricos y derechos claros.
3. Ocultar rutas personales, cuentas, notificaciones y datos privados. Una ubicación
   solo puede mostrarse cuando el autor haya decidido publicarla expresamente.
4. Esperar a que termine cualquier carga o análisis, salvo que ese proceso sea el tema.
5. Mostrar una acción comprensible: reproducción, cue, medidores o controles activos.
6. Mantener al menos `1366×768`; conservar el PNG maestro antes de crear recortes.
7. No simular Linux con una captura de Windows. La serie Linux se hará en Linux real.
8. Capturar otros idiomas solo cuando una tienda o campaña realmente los necesite.
9. Acompañar cada imagen pública con texto alternativo; el documento debe entenderse
   incluso si la imagen no se ve.

## Derivados para web y redes

Los PNG de esta carpeta son maestros. Los recortes, composiciones, marcos y versiones
con texto se generarán en una carpeta `derivados/` cuando exista una necesidad concreta.
Nunca se sobrescribe el maestro ni se presenta una composición como si fuera una
captura literal de la aplicación.

La planificación de mensajes y canales está en
[`GUIA_CONTENIDOS_PUBLICOS.md`](../Documentación/GUIA_CONTENIDOS_PUBLICOS.md).
La separación entre código público, archivos de Release y material privado está en
[`POLITICA_REPOSITORIO_PUBLICO.md`](../Documentación/POLITICA_REPOSITORIO_PUBLICO.md).
