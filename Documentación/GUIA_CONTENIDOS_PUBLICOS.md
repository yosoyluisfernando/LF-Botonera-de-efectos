# Guía de contenidos públicos

Documento maestro para presentar LF Botonera de Efectos de forma coherente en GitHub,
Microsoft Store, catálogos Linux, redes sociales, una futura página web y el manual.
No sustituye las fuentes técnicas: las conecta y define cómo reutilizarlas.

## 1. Fuentes de verdad

- Descripciones, funciones, palabras clave y declaraciones:
  [`FICHA_PUBLICACION.md`](FICHA_PUBLICACION.md).
- Capturas, calidad, restricciones y selección por uso:
  [`../Capturas/README.md`](../Capturas/README.md).
- Funcionamiento para desarrolladores: [`LIBRO_PROYECTO.md`](LIBRO_PROYECTO.md).
- Estrategia del manual: [`PLAN_MANUAL_USUARIO.md`](PLAN_MANUAL_USUARIO.md).
- Privacidad, soporte y licencia: `PRIVACY.md`, `SUPPORT.md` y `LICENSE` en la raíz.

Si cambia una función, primero se actualiza su fuente de verdad. No se corrigen textos
copiados de forma independiente en varios canales.

## 2. Presentación común del producto

**Frase principal:** botonera profesional de sonidos para radio, locución y streaming
en directo.

**Problema que resuelve:** permite tener efectos, identificaciones, locuciones y música
ordenados y listos para disparar durante una producción, sin buscar archivos mientras
se está al aire.

**Diferencias principales:**

- perfiles, pestañas y rejillas adaptables a cada programa o proyecto;
- motor nativo con reproducción simultánea y preescucha independiente;
- editor no destructivo con forma de onda, cues, ganancia y normalización;
- panel fijo, reproductor auxiliar y consola con controles separados;
- atajos locales y globales para trabajar sin depender del ratón;
- software libre GPL-3.0-or-later, sin publicidad, cuenta obligatoria ni telemetría.

## 3. Públicos y enfoque

- **Operadores de radio:** rapidez al aire, locuciones de hora y clima, perfiles y
  compatibilidad con LF Automatizador.
- **Locutores y productores:** preescucha, editor de pistas, normalización y orden por
  cliente o proyecto.
- **Streamers y podcasters:** atajos globales, mezcla simultánea, panel fijo y salidas
  de audio diferenciadas.
- **DJs y animadores:** rejillas grandes, colores, modos de reproducción y música de
  fondo independiente.

No todos necesitan todas las funciones. Cada publicación debe elegir un problema y una
audiencia, en lugar de enumerar todo el programa.

## 4. Honestidad y límites

- La versión `1.2.1` prepara la distribución en Microsoft Store; no promete nuevas
  funciones ni mejoras de rendimiento respecto de `1.2.0`.
- No anunciar aprobación o disponibilidad en Microsoft Store hasta que Microsoft la
  confirme públicamente.
- Linux sigue siendo experimental hasta completar una prueba física.
- No afirmar accesibilidad completa sin una auditoría específica, aunque los textos y
  documentos se diseñen para funcionar bien con lector de pantalla.
- La consola puede mostrarse, pero debe conservarse la indicación de que está en prueba.
- Los audios los aporta el usuario; nunca sugerir que la aplicación incluye una
  biblioteca comercial de música o efectos.

## 5. GitHub

El README debe explicar qué problema resuelve antes de listar funciones. Galería mínima:

1. rejilla principal en modo claro;
2. panel de botones fijos;
3. editor de pistas;
4. consola de audio, con su estado experimental indicado.

Cada imagen tendrá una frase explicativa y texto alternativo específico. No se cargará
la galería ni se cambiará la rama principal hasta conocer la respuesta de Microsoft.

## 6. Microsoft Store

La primera galería mejorada debería seguir una narración: vista general, panel fijo,
reproductor, editor, consola y tema oscuro. Las imágenes de configuración son material
de ayuda, no de venta. La captura con ruta y ubicación local queda excluida.

La ficha ya enviada se mantiene estable durante la certificación. Si se aprueba, las
capturas adicionales se tratarán como una actualización posterior, no como un cambio
apresurado del primer envío.

## 7. Linux y Flathub

Las capturas para Linux deben obtenerse en una sesión Linux real y mostrar los temas y
diálogos nativos de esa plataforma. Se puede conservar la misma paleta de demostración,
pero no reutilizar una ventana de Windows como evidencia de compatibilidad.

Antes de crear esa serie se verificará instalación, audio, selector de archivos,
atajos compatibles y persistencia. Después se añadirá `Capturas/linux/<idioma>/` con la
misma nomenclatura de la biblioteca actual.

## 8. Redes sociales

Las publicaciones deben ser breves y centrarse en una sola función. Series posibles:

- «Tu programa, tus botones»: perfiles, pestañas y rejilla;
- «Escucha antes de salir al aire»: preescucha independiente;
- «Ajusta sin tocar el original»: editor, cues y normalización;
- «Música y efectos por separado»: reproductor auxiliar y consola;
- «Software libre para producción en directo»: licencia y ausencia de telemetría.

Los recortes para formato cuadrado o vertical se derivan de los PNG maestros. Si se
añaden rótulos, se describirán como composición promocional, no como captura literal.

## 9. Futura página web

Una primera web puede construirse con esta estructura:

1. nombre, frase principal y captura de la rejilla;
2. tres problemas resueltos: rapidez, organización y control de audio;
3. galería comentada por función;
4. descargas para Windows y, cuando esté validado, Linux;
5. compatibilidad con LF Automatizador;
6. licencia libre, privacidad y ausencia de telemetría;
7. soporte, documentación y código fuente.

La web no debe inventar otra descripción. Su texto se adapta desde
`FICHA_PUBLICACION.md`, y su galería se selecciona desde `Capturas/README.md`.

## 10. Manual de usuario

El manual debe seguir siendo comprensible sin imágenes. Las capturas sirven para
confirmar visualmente un paso, nunca para reemplazar instrucciones. Cada uso incluirá:

- nombre exacto del control, no «arriba» o «a la derecha»;
- texto alternativo que explique la información relevante;
- versión y plataforma de la captura cuando puedan afectar el resultado;
- advertencia si los valores mostrados son ejemplos y no recomendaciones.

Las capturas actuales cubren rejilla, tipos de botón, menú contextual, panel fijo,
reproductor, preescucha, editor, consola y siete secciones de configuración. Aún faltan
asistente inicial, perfiles, pestañas, importación/exportación y solución de problemas.

## 11. Flujo para nuevos materiales

1. Confirmar que la función existe y está documentada.
2. Preparar datos de demostración sin información personal ni derechos dudosos.
3. Capturar en Release, en la plataforma y versión que se afirma mostrar.
4. Guardar el PNG maestro en la biblioteca y registrarlo en su catálogo.
5. Seleccionar el canal y crear solo los derivados necesarios.
6. Revisar texto alternativo, afirmaciones, versión y privacidad antes de publicar.
