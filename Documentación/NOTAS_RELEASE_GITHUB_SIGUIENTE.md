# Borrador de publicación para GitHub

## Estado

- Versión propuesta: **1.3.0**.
- Última versión publicada en GitHub: **1.2.0**.
- Versión actualmente publicada en Microsoft Store: **1.2.1**.
- Microsoft Store validó el paquete **1.3.0.0** y la Submission 3
  (`1152921505701501616`) está en certificación desde el 2026-07-27.
- No publicar este texto ni crear el tag hasta que Microsoft apruebe la certificación
  y comience la publicación automática de la misma versión funcional.

La propuesta es 1.3.0 porque incorpora una capacidad nueva importante —Biblioteca y
buscador interno— sin romper compatibilidad. Microsoft Store recibirá técnicamente el
paquete `1.3.0.0`; GitHub y la aplicación mostrarán `1.3.0`.

---

# LF Botonera de Efectos 1.3.0

## YA DISPONIBLE EN MICROSOFT STORE PARA WINDOWS

LF Botonera de Efectos ya puede instalarse y actualizarse directamente desde
Microsoft Store:

https://apps.microsoft.com/detail/9NJ8ST39QP7V

La descarga tradicional desde GitHub continúa disponible. Las instalaciones de
Microsoft Store reciben sus actualizaciones mediante Windows; las instalaciones
directas consultan GitHub Releases. Ambas ediciones comparten las mismas funciones y
el mismo número de versión.

## Nueva Biblioteca y buscador interno

Esta versión incorpora un catálogo rápido para encontrar Música y Efectos sin
recorrer el disco completo en cada búsqueda.

- Se pueden añadir tantas carpetas de Música y Efectos como sea necesario.
- Las raíces y subcarpetas solapadas se unifican para no duplicar archivos.
- Una subcarpeta puede pertenecer a una categoría distinta y se respeta como excepción
  por ser más específica.
- La primera indexación descubre los nombres rápidamente y completa después duración,
  título, artista, álbum, género, año y número de pista.
- Los cambios posteriores son incrementales: crear, modificar, mover o borrar un
  archivo actualiza únicamente lo necesario.
- La búsqueda difusa tolera errores de escritura sin devolver coincidencias al azar.
- El catálogo comparte la base de pistas existente; no se crea una segunda base ni se
  duplican los metadatos técnicos, cue o ajustes de ganancia.

## Ventana Biblioteca

Una nueva ventana independiente reúne todo el contenido indexado y permite explorar
también las unidades de almacenamiento sin añadirlas automáticamente al catálogo.

- Navegador separado para Efectos, Música, carpetas y unidades.
- El panel derecho muestra exclusivamente archivos de audio.
- Doble clic para desplegar u ocultar subcarpetas en el árbol.
- Vista por `Nombre del archivo` o por `Título y artista`.
- Lista virtual preparada para bibliotecas grandes: la barra representa desde el
  principio la cantidad total y puede saltar directamente a cualquier posición.
- Centro de procesamiento para preparar varias carpetas, cancelar antes de guardar,
  iniciar la indexación y ocultar la ventana sin detener el trabajo.
- Menú contextual con Reproducir al aire, Escucha previa, Añadir al reproductor y
  Editor de pista.

## Buscador en el panel fijo

El panel fijo dispone ahora de tres vistas elegibles directamente desde su encabezado:
Botones fijos, Reproductor y Buscador.

- Filtro entre todo el catálogo, Música y Efectos.
- Selección múltiple mediante clic, `Ctrl`, `Shift`, flechas y menú contextual por
  teclado.
- Presentación por nombre de archivo o por título y artista.
- Arrastre hacia botones principales o pestañas reutilizando las mismas operaciones
  existentes.
- Reproducción LIVE mediante un reproductor visual abajo a la izquierda, separado de
  la escucha previa que aparece abajo a la derecha.

## Reproductor auxiliar

- Selección múltiple consistente con el Buscador.
- Eliminación conjunta de las filas seleccionadas.
- `Ctrl`, `Shift`, flechas, Page Up, Page Down y menú contextual mediante teclado.
- Conserva su motor independiente, sus modos normal, repetición y aleatorio, el
  marcado de siguiente pista, Loop, detener al finalizar, volumen y salida propios.

## Audio e interfaz

- El vúmetro principal y los vúmetros de la consola trabajan a 50 FPS mediante
  telemetría ligera y una caída visual más suave.
- El editor de pistas decodifica por bloques y reutiliza mejor la forma de onda
  persistente, reduciendo la espera al abrir y reabrir canciones largas.
- Disparar un botón no detiene la escucha previa ni el audio del editor, incluso con
  Solo o Detener otros.
- Si la ventana del editor ya existe, cambia correctamente a la pista solicitada, se
  restaura si estaba minimizada y pasa al frente.
- El apartado Acerca de identifica si Microsoft Store o GitHub Releases administra
  las actualizaciones.

## Funciones incorporadas desde GitHub 1.2.0

La versión anterior de GitHub introdujo el panel lateral fijo, el reproductor auxiliar
y la consola de audio. Todo ello continúa incluido:

- Panel de botones fijos global o por perfil, a izquierda o derecha y redimensionable.
- Reproductor independiente con listas `.LFPlay` compatibles con LF Automatizador,
  tres modos de avance, pista siguiente, Loop, detener al finalizar, barra de progreso
  y salida de audio propia.
- Colchón de reproducción para reducir cortes ante accesos lentos al disco.
- Consola con fader y vúmetro por efectos, panel, reproductor, preescucha y programa.
- Cambio de dispositivo de audio en caliente.
- Selección y color conjunto de varios botones.
- Compatibilidad ampliada con locuciones de ZaraRadio, Salamandra y RadioBOSS.
- Preescucha separada del programa aunque comparta el mismo dispositivo físico.
- Editor de pistas con forma de onda, cue, ganancia y normalización no destructiva.

## Instalación

### Microsoft Store

Usa el enlace anterior para una instalación administrada por Windows.

### GitHub Releases

La publicación incluirá los instaladores directos de Windows y, cuando termine la
validación física correspondiente, los paquetes Linux disponibles.

## Actualización

La configuración, perfiles, pestañas, botones, listas y metadatos existentes se
conservan. Aun así, antes de una actualización importante siempre es recomendable
exportar los perfiles de trabajo más valiosos.

---

## Lista interna antes de publicar

- [x] Microsoft Store valida el paquete 1.3.0.0 y recibe la Submission 3.
- [ ] Microsoft Store aprueba la certificación y comienza la publicación automática.
- [x] `package.json`, `Cargo.toml`, `Cargo.lock` y `tauri.conf.json` están en 1.3.0.
- [ ] CHANGELOG cerrado como 1.3.0 con la fecha real.
- [x] Build Store identifica el canal `store`.
- [ ] Builds de GitHub identifican el canal `direct`.
- [x] Pruebas Rust, frontend y Release aprobadas.
- [ ] Prueba funcional del paquete MSIX completada.
- [ ] Tag `v1.3.0` y GitHub Release creados solo después de la aprobación de Store.
