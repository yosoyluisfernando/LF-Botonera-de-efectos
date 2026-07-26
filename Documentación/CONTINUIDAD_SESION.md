# Continuidad de sesión — buscador interno y distribución

Este documento es el punto de entrada para retomar el trabajo después de completar la
publicación de **LF Botonera de Efectos 1.2.1** en Microsoft Store y fusionar la rama
de distribución con `main`.

No usar aquí planes históricos de funciones ya completadas.

## 0. Etapa activa — buscador interno

- **Rama:** `codex/buscador-interno`.
- **Base inicial:** `771adf7`, `Integra distribución en tiendas y prepara la etapa
  Linux (#6)`.
- **Inicio:** 2026-07-24.
- **Estado:** arquitectura base aprobada; implementación por etapas autorizada.
- **Documento rector:** [`PLAN_BUSCADOR_INTERNO.md`](PLAN_BUSCADOR_INTERNO.md).

Excepción completada por petición expresa del autor: mejora pequeña del reproductor
auxiliar para seleccionar filas con `Ctrl`/`Shift` y eliminarlas juntas. No forma
parte de la arquitectura del buscador ni la autoriza implícitamente.

La selección usa ids estables en `playerSelection.js`; Rust valida y elimina el lote
mediante `domain/player/queue_remove.rs` y `player_remove_tracks(indexes)`, con una
sola persistencia y sincronización. Verificación: 212 pruebas Rust aprobadas, 4
manuales ignoradas, `cargo build --lib` y `npm run build` correctos. La prueba de uso
real con ratón y lector de pantalla queda para el autor.

El objetivo es añadir al panel fijo una búsqueda rápida sobre una o varias carpetas
de audio elegidas por el usuario. Debe admitir más de 100.000 canciones además de
efectos, búsqueda difusa, índice persistente y actualización incremental sin bloquear
la interfaz ni los motores de audio.

Decisiones aprobadas el 2026-07-25:

- tercera presentación `fixed_panel.view = "search"` para búsqueda rápida;
- ventana independiente Biblioteca para administrar todo el catálogo;
- un solo motor Rust y el `tracks.db` existente para ambas superficies;
- dos colecciones explícitas, Música y Efectos;
- descubrimiento rápido y enriquecimiento posterior con duración y etiquetas;
- actualización automática, reconciliación al iniciar y prueba con 100.000/250.000;
- detección y unificación de raíces solapadas sin duplicar archivos.

La categoría la elige el usuario al añadir la carpeta. Si raíz y subcarpeta tienen la
misma categoría se unifican con aviso. Si son distintas, la subcarpeta específica se
conserva como excepción y manda dentro de su árbol.

Enter y doble clic siguen sin decidirse. El menú contextual sí queda aprobado, en
este orden: `Reproducir al aire`, `Escucha previa`, `Añadir al reproductor` y
`Editor de pista`.

`Reproducir al aire` usará el motor de efectos y el bus Programa, con cue, ganancia,
normalización, máster y Stop general. Compartirá el componente visual de escucha
previa, pero no su id ni su bus: CUE y LIVE deben poder sonar y controlarse a la vez.
La mini ventana CUE permanece abajo a la derecha y la mini ventana `LIVE` aparecerá
abajo a la izquierda, sin cubrirse. Traducciones aprobadas: `Play on air` en inglés
y `Reproduzir no ar` en portugués de Brasil y Portugal.

Selección aprobada: clic único, `Ctrl+clic`, `Shift+clic`, flechas, `Shift+flechas`,
Page Up/Down, Escape y menú contextual por tecla Menú o `Shift+F10`. Con varias
pistas, solo `Añadir al reproductor` estará habilitado y añadirá el lote completo en
orden mediante una sola operación Rust. La selección se identifica por ruta estable,
no por posiciones ni nodos visibles.

Arrastre aprobado: una pista sobre una celda la asigna y confirma antes de reemplazar;
una o varias pistas sobre una pestaña ocupan en orden sus primeros espacios vacíos.
Si no caben todas, la operación completa se rechaza sin inserción parcial.

Decisión de navegación para la futura interfaz, anotada el 2026-07-25: el encabezado
que muestra `Botones fijos`, `Reproductor` o `Buscador` debe permitir cambiar
directamente entre las tres vistas al activarlo. No obligar al usuario a entrar en
Ajustes para cada cambio. Cada idioma tendrá un nombre normal y otro compacto
comprensible —por ejemplo, `Botones fijos` y `B. fijos` en español—; la interfaz
elegirá según el espacio medido, no mediante recorte ciego. La presentación concreta
se decidirá al construir la UI.

La Biblioteca será una ventana independiente y tendrá su propio acceso cuando se
diseñe; no añadir un botón `Abrir Biblioteca` dentro del buscador. Sus listas no
mostrarán páginas. Usarán carga perezosa y virtualización: conservarán en el DOM lo
visible y un margen de 50 resultados por encima y 50 por debajo. El mismo
flujo debe responder al ratón, scroll y teclado con controles nativos y nombres
accesibles desde el principio. La auditoría integral de lector de pantalla puede ser
una etapa separada.

Primera base técnica completada el 2026-07-25:

- `tracks.db` migra del esquema 1 al 2 conservando las pistas;
- `library_root` guarda una ruta única y su colección Música/Efectos;
- `domain/library/root_plan.rs` decide duplicado, cobertura, unificación, excepción
  y reclasificación; manda siempre la raíz más específica;
- 221 pruebas Rust aprobadas, 4 manuales ignoradas;
- `cargo build --lib` y `npm run build` correctos.

Segunda base técnica y mediciones completadas el 2026-07-25:

- las cinco raíces reales autorizadas suman 18.201 audios y unos 77,75 GiB;
- el recorrido Rust compartido tarda 4.483 ms frío y 956-1.050 ms caliente;
- la lectura inicial secuencial de duración y tags tardó 480.976 ms;
- cuatro trabajadores son el punto óptimo medido: 2.809-2.907 ms con caché caliente;
- la reconciliación sin cambios tarda 623 ms y no abre el contenido;
- 476 archivos recuperan propiedades aunque sus tags fallen; 96 no entregan
  propiedades, incluidos los 78 WMA;
- el motor Release real de LF Automatizador se probó dos veces con bases temporales:
  primera indexación 11.963-12.905 ms y segunda sin cambios 10.224-11.689 ms;
- LF Automatizador omite WMA y vuelve a leer metadatos en la segunda sincronización;
  no copiar esa condición;
- el recorrido, el sello de archivo, la lectura robusta y el lote concurrente ya
  tienen una sola implementación Rust en la Botonera.
- verificación actual: 226 pruebas automáticas aprobadas, 10 pruebas manuales de
  archivos reales ignoradas por defecto, `cargo build --lib` y `npm run build`
  correctos.

Los detalles y la cobertura de etiquetas por Música/Efectos están en
`PLAN_BUSCADOR_INTERNO.md`, secciones 5.5 y 5.6.

Tercera base técnica completada el 2026-07-25:

- `tracks.db` migra al esquema 3: `library_root`, `library_track` y el índice derivado
  `library_track_search` FTS5; no existe una segunda base de biblioteca;
- cada colección admite cualquier cantidad de raíces independientes;
- las raíces solapadas de la misma colección se unifican y una subcarpeta de la otra
  colección se conserva como excepción, sin filas duplicadas;
- el catálogo inicial guarda nombres y rutas por lotes antes de enriquecer duración y
  etiquetas con cuatro trabajadores;
- una segunda sincronización compara tamaño y `mtime`: no reabre archivos sin cambios;
- retirar una raíz quita sus resultados, pero conserva cue, ganancia, normalización y
  demás datos técnicos de `track`;
- búsqueda compartida para panel y Biblioteca: acentos equivalentes, exacto, prefijo,
  subcadena y distancia Damerau acotada sobre candidatos FTS5;
- siete comandos IPC y el evento `library-index-progress` exponen el motor sin poner
  lógica de negocio en JavaScript;
- la dependencia `unicode-normalization` se usa únicamente para hacer equivalentes
  búsquedas con y sin diacríticos.

Evidencia Release con base descartable, nunca con la base actual:

- 100.000 filas: construcción 4.178 ms; consulta difusa media 11,5 ms;
- 250.000 filas: construcción 10.984 ms; consulta difusa media 29,4 ms y p95 34,5 ms;
- cinco raíces reales: 18.201 audios, 17.240 Música y 961 Efectos;
- primera lectura completa de duración y etiquetas: 257.471 ms, con 18.105 archivos
  enriquecidos y 96 formatos no legibles;
- segunda y tercera reconciliación: 2.805 y 2.792 ms, cero reaperturas de metadatos.
- verificación automática: 243 pruebas aprobadas y 12 manuales ignoradas; `cargo
  build --lib` y `npm run build` correctos.

El tiempo inicial alto pertenece a la lectura fría del contenido de miles de archivos.
Los nombres quedan consultables por lotes antes de terminar el enriquecimiento. No
presentar los 257 segundos como tiempo de descubrimiento.

Siguiente paso: observación incremental y reconciliación al iniciar; después, diseñar
la interfaz de Biblioteca y la tercera vista del panel con el autor. Enter, doble clic
y reproducción al aire continúan sin decidirse.

Cuarta base técnica completada el 2026-07-25:

- `notify` 8.2 observa las raíces mediante el mecanismo nativo de cada sistema;
- ráfagas de crear, modificar, mover y borrar se agrupan durante 250 ms;
- un archivo afectado se invalida y relee de forma puntual, aunque tamaño y `mtime`
  coincidan, para no perder cambios dentro de la resolución del sistema de archivos;
- los eventos de directorio y errores del observador solicitan reconciliación de
  seguridad; `notify` no se considera una fuente infalible;
- al iniciar se lanza una reconciliación en segundo plano sin bloquear UI ni audio;
- altas y retiradas de raíces actualizan también las carpetas observadas;
- las operaciones de catálogo se serializan, mientras las búsquedas usan conexiones
  SQLite independientes;
- prueba física temporal: crear, modificar y borrar un WAV fue reflejado
  automáticamente; actualización Release de un archivo real, 9,6 ms fría y 3,3 ms
  caliente.
- verificación actual: 245 pruebas automáticas aprobadas, 14 manuales ignoradas,
  `cargo build --lib` y `npm run build` correctos.

Quinta base técnica completada el 2026-07-25:

- `tracks.db` migra al esquema 4 con índices de recorrido estable por nombre y ruta;
- `library_browse` entrega bloques en ambas direcciones para scroll continuo sin
  cargar el catálogo completo en memoria ni exponer páginas al usuario;
- cursores estables permiten retirar o añadir pistas entre solicitudes sin depender
  de posiciones numéricas frágiles;
- en Release, con 100.000 pistas, los bloques de 100 tardaron 494 µs hacia delante,
  438 µs para el siguiente y 490 µs hacia atrás;
- con 250.000 pistas tardaron 449 µs, 451 µs y 472 µs respectivamente;
- la UI mantendrá solo las filas visibles más un margen de 50 arriba y 50
  abajo; este margen es una decisión de presentación ajustable, no una página.
- verificación actual: 249 pruebas automáticas aprobadas, 14 pruebas físicas
  ignoradas por defecto, `cargo build --lib` y `npm run build` correctos.

Sexta etapa implementada el 2026-07-25:

- `fixed_panel.view = "search"` funciona como tercera vista y el encabezado común
  alterna directamente Botones fijos, Reproductor y Buscador;
- las etiquetas normal/compacta se eligen según el ancho disponible y existen en los
  cuatro idiomas;
- la lista virtual conserva hasta 50 filas de margen arriba y abajo y una ventana de
  datos bidireccional acotada; la selección vive por ruta estable;
- clic, `Ctrl`, `Shift`, flechas, Page Up/Down, Escape, tecla Menú y `Shift+F10`
  están conectados sin asignar todavía una acción a Enter ni doble clic;
- el menú contextual respeta el orden aprobado y permite añadir un lote completo al
  reproductor mediante una sola persistencia;
- LIVE reutiliza el control visual de CUE, pero sale por Programa con id propio;
  ambas mini ventanas pueden coexistir abajo a izquierda/derecha;
- arrastrar una pista a una celda reutiliza la asignación existente; arrastrar una o
  varias a una pestaña usa una operación Rust atómica, sin inserciones parciales.
- verificación automática: 251 pruebas aprobadas y 14 pruebas físicas ignoradas;
  `cargo build --lib`, `npm run build` y `cargo build --release` correctos. El
  ejecutable Release resultante es `src-tauri/target/release/tauri-app.exe`.

Siguiente paso: prueba funcional Release por el autor. Después se corregirá cualquier
hallazgo y se construirá la ventana independiente Biblioteca sobre los mismos módulos,
sin duplicarlos.

Séptima etapa implementada el 2026-07-25 a partir de la primera prueba funcional:

- el encabezado ya no rota vistas por cada clic: abre un menú accesible para elegir
  directamente Botones fijos, Reproductor o Buscador;
- el alfiler del Buscador abre un modal con listas separadas para Música y Efectos;
- se pueden preparar todas las carpetas necesarias; Cancelar descarta el borrador sin
  escribir y solo Iniciar confirma;
- `library_add_roots` valida primero todas las rutas y guarda el lote completo dentro
  de una sola transacción. Una ruta inválida revierte todo el lote;
- la regla existente unifica raíz/subcarpeta sin duplicar y conserva excepciones de
  otra categoría;
- el evento `library-index-progress` muestra descubrimiento, alta de nombres y lectura
  de duración/etiquetas tanto en el modal como en el panel fijo;
- causa del ejecutable defectuoso: se entregó un `cargo build --release` directo, que
  no sustituye el flujo de producción de Tauri y conservó la URL
  `http://localhost:1420`. Las pruebas entregables se generan con
  `npm run tauri build -- --no-bundle`.

Verificación de esta corrección: 253 pruebas aprobadas, 14 pruebas físicas ignoradas,
prueba específica de lote atómico correcta, `cargo build --lib`, `npm run build` y
`npm run tauri build -- --no-bundle` correctos. El ejecutable entregable quedó en
`src-tauri/target/release/tauri-app.exe`.

Octava etapa implementada el 2026-07-25 después de medir la primera prueba del autor:

- la afirmación anterior de «pocos segundos» no describe la primera carga completa;
  mezclaba recorridos calientes y bases sintéticas con la lectura real de archivos;
- dos repeticiones Release con base descartable y las cinco raíces reales tardaron
  357.357 ms y 385.020 ms en frío para completar nombres, duración y etiquetas;
- en esas repeticiones, solo recorrer el almacenamiento frío y catalogar 18.201
  nombres tardó 115.750 ms y 122.536 ms. Las reconciliaciones calientes posteriores
  tardaron entre 6.577 y 8.020 ms;
- el indexador ahora guarda nombres en lotes de 500 durante el propio recorrido. No
  espera a terminar todas las carpetas ni a leer duración y etiquetas para hacerlos
  buscables;
- duración y etiquetas se ejecutan como una segunda fase global con cuatro
  trabajadores, el límite óptimo ya medido. Ambas fases ceden ejecución entre lotes;
- abrir de nuevo el administrador muestra primero la ventana y consulta después las
  raíces, evitando que una lectura de SQLite retrase su aparición;
- mientras existe una indexación, Cancelar desaparece y se muestra únicamente
  `Ocultar ventana`. Ocultarla no cancela ni detiene el trabajo;
- el progreso diferencia archivos que van apareciendo en el catálogo de la fase que
  completa duración y etiquetas;
- verificación técnica: 254 pruebas automáticas aprobadas y 14 físicas ignoradas,
  `cargo build --lib`, `npm run build` y `npm run tauri build -- --no-bundle`
  correctos. La prueba perceptiva de fluidez queda para el autor.

El objetivo verificable ya no es prometer una primera indexación completa en segundos
en cualquier disco. Es mostrar progreso verdadero, mantener utilizable la interfaz,
hacer buscables los nombres progresivamente y conseguir reconciliaciones posteriores
rápidas sin releer archivos sin cambios.

Corrección de arrastre registrada el 2026-07-26:

- el arrastre HTML5 del buscador competía en Windows/WebView2 con el receptor nativo
  de archivos de Tauri utilizado por el Explorador y no llegaba a los destinos;
- `libraryDnd.js` usa ahora el mismo patrón interno comprobado de `mousedown`,
  `mousemove` y `mouseup` que los botones existentes;
- al soltar una pista sobre una celda llama a `fileDrop.js::dropFileOnGrid`, la misma
  función del Explorador, por lo que no duplica asignación ni confirmación de
  reemplazo;
- varias pistas seleccionadas siguen usando una sola orden Rust atómica al soltarse
  sobre una pestaña;
- `Ctrl` y `Shift` continúan reservados para modificar la selección. Después se
  arrastra normalmente cualquiera de las filas seleccionadas;
- verificación: 254 pruebas Rust aprobadas, 14 físicas ignoradas, `npm run build`,
  `cargo build --lib` y `npm run tauri build -- --no-bundle` correctos. La prueba
  física del gesto queda para el autor porque no existe un harness del WebView.

Actualización de vúmetros registrada el 2026-07-26:

- antes, la medición Rust por ventanas de 1024 muestras se entregaba a ambas
  interfaces únicamente mediante `audio-tick` cada 100 ms: 10 actualizaciones
  visuales por segundo;
- el nuevo `meter-tick` contiene solo Programa, niveles de buses e `idle`, y se emite
  cada 20 ms: 50 FPS;
- el vúmetro principal y la consola virtual, en modal o ventana independiente,
  consumen esa misma telemetría. No existe un segundo cálculo de nivel;
- `audio-tick` conserva sus 100 ms para progreso, reloj, botones y pestañas. No se
  repinta toda la aplicación cinco veces más rápido;
- al entrar en reposo, el monitor VU emite un cero final y calla, por lo que no
  mantiene 50 eventos por segundo sin audio;
- la subida activa conserva 20 ms y la caída activa usa 50 ms en ambas superficies:
  mantiene la respuesta de 50 FPS, pero evita una caída visual demasiado agresiva;
  el decaimiento final de 800 ms del vúmetro principal se conserva;
- verificación: 255 pruebas automáticas aprobadas y 14 físicas ignoradas,
  `cargo build --lib`, `npm run build` y
  `npm run tauri build -- --no-bundle` correctos.

Presentación del buscador actualizada el 2026-07-26:

- el usuario elige directamente entre `Nombre del archivo` y `Título y artista`
  mediante un menú desplegable accesible, traducido a los cuatro idiomas;
- la elección vive en `FixedPanelConfig.library_display`, usa `metadata` como valor
  compatible predeterminado y se conserva al reiniciar. No existe una preferencia
  paralela en JavaScript;
- `Nombre del archivo` muestra el nombre literal como línea principal y conserva
  título y artista solo como referencia secundaria;
- `Título y artista` muestra el título en grande y artista/álbum en la línea pequeña;
- ciertos archivos reales traen los campos ID3 título y artista invertidos. La base
  conserva sus etiquetas originales; una corrección de presentación compartida por
  búsqueda y recorrido solo invierte los roles cuando hay evidencia: título repetido
  al menos tres veces con artistas variables en la misma carpeta, o un nombre
  estructurado que confirma ambos valores;
- los álbumes correctamente etiquetados no se alteran. Tres pruebas específicas
  cubren grupo invertido, grupo correcto y resultado individual confirmado por el
  nombre del archivo;
- verificación: 260 pruebas automáticas aprobadas, 14 físicas ignoradas,
  `cargo build --lib`, `npm run build` y
  `npm run tauri build -- --no-bundle` correctos.

Selección del reproductor corregida el 2026-07-26:

- buscador y reproductor usan ahora el mismo controlador de selección por identidad
  estable; los adaptadores solo traducen ruta o id de pista;
- el reproductor admite clic normal, `Ctrl+clic`, `Shift+clic`,
  `Ctrl+Shift+clic`, flechas, `Shift+flechas`, Page Up/Down, Escape, tecla Menú y
  `Shift+F10`;
- `Ctrl` y `Shift` quedan reservados para seleccionar y no inician un arrastre;
- la lista expone foco y fila activa mediante atributos accesibles;
- seis escenarios puros de selección fueron comprobados;
- verificación: 260 pruebas Rust aprobadas, 14 físicas ignoradas,
  `cargo build --lib`, `npm run build` y
  `npm run tauri build -- --no-bundle` correctos.

Diagnóstico de memoria del 2026-07-26:

- se midió el ejecutable Release recién compilado, no una sesión debug;
- primer arranque: máximo de 49,7 MiB de conjunto de trabajo y 31,4 MiB privados
  durante la reconciliación; al séptimo segundo bajó a 43,6 MiB y 25,1 MiB;
- durante 30 segundos en reposo y otros 10 con la ventana visible no hubo crecimiento:
  quedó estable entre 24,3 y 25,1 MiB privados;
- la precarga configurada es `full_profile`, con presupuesto de 256 MiB, pero el
  perfil activo solo reúne 21 efectos elegibles, 58,4 segundos y aproximadamente
  9,8 MiB PCM estéreo a 44,1 kHz;
- WebView2 usa procesos separados de navegador, GPU, red, almacenamiento y
  renderizador. Su conjunto de trabajo incluye páginas compartidas y no equivale a
  memoria privada retenida por el núcleo Rust;
- no se reprodujo una fuga ni una retención de 90 MiB en el proceso principal. No
  aplicar flags agresivos de WebView2 ni reducir la precarga sin una medición que
  identifique crecimiento sostenido;
- la instancia Release iniciada para la medición se cerró al terminar.

Se auditó `C:\LF Automatizador v1.0` como referencia, excluyendo completamente
`C:\LF Automatizador v1.0\LF Automatizador 2.0`. Los hallazgos útiles y los límites que
no deben copiarse quedaron registrados en `PLAN_BUSCADOR_INTERNO.md`.

Antes de continuar con código:

1. Leer completo `PLAN_BUSCADOR_INTERNO.md`.
2. Respetar las decisiones aprobadas; Enter y doble clic siguen aplazados.
3. Conservar una sola implementación para raíces, catálogo y búsqueda.
4. Al tocar rendimiento, repetir las pruebas sintéticas y la base real descartable.
5. Justificar cualquier dependencia antes de añadirla.
6. Mantener actualizados arquitectura, reglas y evidencia junto a cada etapa.

La prueba física en Linux y Flathub siguen pendientes, pero quedan pausados mientras
se diseña esta actualización. La documentación histórica de distribución permanece
vigente más abajo.

## 1. Lectura inicial obligatoria

Antes de proponer o modificar código:

1. Leer `AGENTS.md` en la raíz del repositorio.
2. Leer [`REGLAS_PROYECTO.md`](REGLAS_PROYECTO.md).
3. Leer este documento completo.
4. Para la etapa activa, leer
   [`PLAN_BUSCADOR_INTERNO.md`](PLAN_BUSCADOR_INTERNO.md).
5. Para retomar Linux, leer
   [`PLAN_DISTRIBUCION_TIENDAS.md`](PLAN_DISTRIBUCION_TIENDAS.md) y
   [`CHECKLIST_PREPUBLICACION_LOCAL.md`](CHECKLIST_PREPUBLICACION_LOCAL.md).
6. Auditar el código y la configuración actuales antes de decidir arquitectura.

Los documentos `MSIX_LOCAL.md`, `WACK_MSIX.md` y
`ACTUALIZAR_MICROSOFT_STORE.md` son referencia histórica o para futuras
actualizaciones de Microsoft Store. No forman parte de la lectura inicial de Linux.

---

## 2. Estado actual

- **Proyecto:** LF Botonera de Efectos.
- **Versión en Microsoft Store:** 1.2.1.
- **Última versión pública en GitHub Releases:** 1.2.0. La publicación 1.2.1 en
  GitHub sigue pendiente hasta que el autor decida unificar los canales.
- **Stack:** Tauri v2, backend Rust, frontend Vanilla JS con Vite.
- **Licencia:** GPL-3.0-or-later.
- **Repositorio local:** `C:\OVERLAY\BOTONERA`.
- **Rama de trabajo:** `codex/buscador-interno`.
- **Base de la rama:** `771adf7`,
  `Integra distribución en tiendas y prepara la etapa Linux (#6)`.
- **Identificador técnico común:**
  `io.github.yosoyluisfernando.LF-Botonera-de-efectos`.
- **Prioridad activa:** prueba funcional Release de la indexación progresiva y su
  respuesta visual; después, ventana Biblioteca independiente.
- **Prioridad pausada:** prueba física en Linux y después Flathub.
- **Primer destino previsto:** Flathub, después de una prueba física real en Linux.
- **Destinos posteriores:** evaluar repositorios oficiales de Debian, Fedora u otras
  distribuciones únicamente cuando el paquete y su mantenimiento sean sostenibles.
- **Cambios de distribución:** ya fusionados en `main` mediante el PR #6.
- **Última verificación local:** 209 pruebas Rust aprobadas, 4 pruebas manuales
  ignoradas, `cargo build --lib`, `npm run build` y aplicación Windows Release
  completados correctamente.

Antes de editar, ejecutar `git status` y conservar cualquier cambio ajeno a la tarea.
No limpiar, descartar ni sobrescribir el árbol de trabajo.

---

## 3. Microsoft Store — hito completamente cerrado

No hay trabajo pendiente de publicación en Microsoft Store para esta etapa.

- La aplicación está aprobada y disponible públicamente:
  `https://apps.microsoft.com/detail/9NJ8ST39QP7V`.
- El identificador de Store es `9NJ8ST39QP7V`.
- La versión pública es 1.2.1.
- La actualización de metadatos
  `1152921505701463690` fue aprobada.
- La ficha pública ya muestra la descripción definitiva, los enlaces directos al
  sitio oficial y al soporte, y las siete capturas de pantalla acordadas.
- La primera publicación y la actualización posterior fueron verificadas por el
  autor.
- El canal `store` deshabilita la consulta de actualizaciones en GitHub y deja que
  Microsoft Store administre las actualizaciones.
- La rutina de futuras versiones está documentada en
  [`ACTUALIZAR_MICROSOFT_STORE.md`](ACTUALIZAR_MICROSOFT_STORE.md).

La próxima sesión no debe entrar en Partner Center, modificar la ficha ni reconstruir
el MSIX, salvo que el autor abra expresamente una nueva tarea de Microsoft Store.

---

## 4. Decisión cerrada: cada canal administra sus actualizaciones

La experiencia de Microsoft Store deja una regla obligatoria para Linux:
**una instalación nunca debe quedar administrada a la vez por GitHub y por una
tienda o repositorio**.

Comportamiento requerido:

- **Microsoft Store:** Microsoft Store busca, descarga e instala actualizaciones. El
  comprobador de GitHub permanece deshabilitado.
- **Descarga directa desde GitHub Releases:** puede conservar el comprobador de
  GitHub y dirigir al usuario al canal directo correspondiente.
- **Flathub o Flatpak administrado por un repositorio:** Flatpak administra las
  actualizaciones. La aplicación no debe ofrecer ni iniciar actualizaciones desde
  GitHub.
- **Paquete instalado desde un repositorio DEB o RPM:** APT, DNF o el gestor propio
  de la distribución administra las actualizaciones. La aplicación no debe ofrecer
  ni iniciar actualizaciones desde GitHub.
- **DEB, RPM o AppImage descargado directamente desde GitHub:** debe tratarse como
  canal directo, salvo que la auditoría técnica demuestre que un formato necesita
  otra política explícita.

El canal de distribución debe quedar definido de forma explícita y reproducible
durante la compilación o el empaquetado. No se debe depender únicamente de
heurísticas frágiles en tiempo de ejecución ni mantener dos fuentes de verdad.

Para cada futuro canal administrado de Linux, antes de programar:

1. Auditar el pipeline real del destino y quién administra sus actualizaciones.
2. Revisar todos los puntos de interfaz y backend afectados por ese canal.
3. Extender la arquitectura existente sin crear otra fuente de verdad.
4. Explicar qué se decide en compilación, qué metadatos recibe cada paquete y cómo se
   prueba que dos gestores de actualización no puedan mezclarse.
5. Si la solución cambia estructura, flujo IPC o arquitectura, presentar el plan y
   esperar aprobación antes de implementarla.

Los nombres definitivos de Flatpak, APT, DNF u otros canales todavía no están
cerrados. Deben decidirse al diseñar cada pipeline, no inventarse por adelantado.

### 4.1 Estado real de la implementación

La base `direct`/`store` ya está implementada y no debe volver a duplicarse:

- `src-tauri/src/domain/distribution.rs` decide canal, plataforma y administrador.
- `get_distribution_info` entrega esos datos a «Acerca de».
- `cmd_updates.rs` consulta la misma fuente antes de acceder a GitHub.
- `scripts/build-store-msix.ps1` fija `LF_DISTRIBUTION_CHANNEL=store`.
- Los builds actuales de GitHub para Windows y Linux son `direct`; la ausencia de la
  variable conserva ese valor por compatibilidad.

Todavía **no** existen canales administrados para Flatpak, APT o DNF. Se añadirán a
la misma fuente cuando se prepare cada pipeline, nunca mediante detección de rutas,
extensiones o archivos instalados.

Decisión fundamental: plataforma, formato y canal son ejes independientes. Un DEB
de GitHub es directo; un futuro DEB de un repositorio APT será administrado. Linux no
significa automáticamente Flathub y Windows no significa automáticamente Store.

### 4.2 Cómo mejorar Linux sin dañar Windows

- Investigar primero si la causa es común o realmente exclusiva de Linux.
- Mantener la lógica de negocio y audio común siempre que sea posible.
- Aislar código nativo con `cfg(target_os)` y una API común solo cuando haya evidencia.
- No decidir comportamiento de plataforma en JavaScript.
- Ejecutar la matriz común y ambos jobs CI después de cada cambio.
- Probar audio y funciones del sistema en una compilación Release física.

La política completa y sus límites están en
[`ARCHITECTURE.md`](ARCHITECTURE.md#política-para-cambios-específicos-de-windows-o-linux);
los comandos y marcas de canal están en
[`COMPILACION_Y_VERSIONES.md`](COMPILACION_Y_VERSIONES.md#21-plataforma-formato-y-canal-no-son-lo-mismo).

---

## 5. Orden de trabajo para Linux

No iniciar Flathub, Debian y Fedora al mismo tiempo. Trabajar en etapas pequeñas y
verificables:

1. **Completado:** auditar soporte Linux, paquetes Tauri, dependencias, rutas,
   permisos y actualizador.
2. **Completado para los canales actuales:** implementar una fuente común para
   `direct` y `store`. Los canales administrados de Linux se añaden al preparar cada
   pipeline.
3. Verificar la compilación local común:
   - `cargo test --lib`;
   - `cargo build --lib`;
   - `npm run build`.
4. Preparar una compilación **Release** en Linux y generar los formatos que realmente
   soporte el proyecto. No evaluar audio con una compilación debug.
5. Probar físicamente en Linux antes de anunciar soporte estable:
   - inicio, cierre y persistencia;
   - reproducción de efectos y reproductor auxiliar;
   - salida principal, preescucha y selección de dispositivos;
   - archivos y carpetas externos;
   - arrastre, diálogos y rutas con espacios o caracteres no ASCII;
   - editor de pistas, onda, cue y normalización;
   - clima y acceso de red;
   - atajos locales y globales en X11 y Wayland;
   - importación y exportación de `.bdelf`, `.bdeplf` y `.LFPlay`;
   - PipeWire, PulseAudio o ALSA según el sistema de prueba.
6. Corregir las causas reales encontradas y repetir la matriz Release.
7. Preparar los metadatos Linux reutilizables: archivo `.desktop`, AppStream,
   iconos, capturas, licencia, privacidad, soporte y descripción en los idiomas que
   exija el destino.
8. Preparar Flatpak para Flathub:
   - compilación desde fuentes y sin descargar dependencias durante el build;
   - permisos mínimos y portales cuando correspondan;
   - acceso correcto a audio y archivos elegidos por el usuario;
   - identificador y metadatos coherentes;
   - actualización administrada exclusivamente por Flatpak.
9. Validar el manifiesto y el paquete, realizar prueba física del Flatpak y solo
   entonces iniciar la solicitud a Flathub.
10. Después de Flathub, evaluar por separado los requisitos de repositorios oficiales
    de Debian y Fedora, incluyendo disponibilidad de dependencias, políticas de
    empaquetado, firma, mantenimiento y canal de actualizaciones.

---

## 6. Evidencia y decisiones reutilizables

- El código está planteado como multiplataforma y Tauri tiene previstos paquetes
  DEB, RPM y AppImage, pero esto no equivale a soporte Linux verificado.
- Las rutas de datos usan la abstracción del proyecto y SQLite se integra con la
  aplicación; aun así, deben comprobarse en un sistema Linux real.
- El acceso a archivos absolutos, el audio, los dispositivos y los atajos globales
  son los riesgos principales dentro del sandbox de Flatpak.
- El proyecto es software libre bajo GPL-3.0-or-later y ya dispone de política de
  privacidad, soporte, avisos de terceros, capturas y sitio oficial.
- GitHub Releases seguirá siendo un canal válido para descargas directas. Publicar en
  Flathub o en un repositorio oficial no debe eliminar ese canal; debe separarlo.
- No hay telemetría ni publicidad. Las conexiones conocidas son GitHub Releases para
  el canal directo, Open-Meteo cuando el usuario usa el clima y PayPal únicamente
  mediante una acción consciente del usuario.
- Las pruebas automáticas no sustituyen la prueba física ni auditiva. El agente
  realiza compilaciones, tests y auditoría técnica; Luis Fernando valida la experiencia
  real y el audio.
- Si Luis Fernando oye cortes, clics, saturación o un ruteo incorrecto aunque los
  tests pasen, la prueba se considera fallida y se investiga la causa.

---

## 7. Primer punto de reanudación

La auditoría inicial y la base `direct`/`store` ya están completadas. Una sesión nueva
debe:

1. Confirmar rama, versión y árbol de trabajo; conservar todos los cambios locales.
2. No volver a Partner Center ni reconstruir MSIX salvo petición expresa del autor.
3. Revisar que los cambios de canal y preescucha continúan presentes y verificados.
4. Preparar la prueba física Release en Linux. La primera distribución recomendada
   es Ubuntu Desktop 26.04 LTS; comenzar con DEB y AppImage.
5. Registrar distribución, versión, Wayland/X11, PipeWire/PulseAudio/ALSA, formato
   instalado, una o dos tarjetas y resultados de la matriz funcional.
6. Corregir cada hallazgo en el núcleo común o en un adaptador Linux según su causa,
   repitiendo siempre la verificación Windows.
7. Solo después de esa prueba diseñar el canal administrado `flatpak` y los permisos
   de sandbox. Si exige cambios estructurales o IPC, presentar el plan al autor.

El objetivo inmediato no es publicar, sino obtener un paquete Linux Release
reproducible y físicamente probado sin alterar el comportamiento estable de Windows.
