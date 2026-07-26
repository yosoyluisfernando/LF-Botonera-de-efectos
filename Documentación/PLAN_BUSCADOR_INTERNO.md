# Plan — buscador interno e índice de archivos de audio

Documento rector para la nueva actualización de LF Botonera de Efectos. Conserva el
análisis, las decisiones aprobadas y las cuestiones que siguen abiertas. Toda sesión
debe leerlo antes de modificar el buscador o la Biblioteca.

**Estado:** arquitectura base aprobada; implementación por etapas autorizada.

**Rama:** `codex/buscador-interno`.

**Base:** `main` en `771adf7`, después de integrar la distribución en tiendas.

**Inicio:** 2026-07-24.

---

## 0. Decisiones aprobadas por el autor

Decisiones cerradas el 2026-07-25:

1. El buscador rápido será una tercera vista del panel fijo.
2. Existirá además una ventana independiente llamada **Biblioteca** para recorrer y
   administrar todo lo indexado.
3. Panel y Biblioteca compartirán el mismo motor Rust, el mismo estado y el mismo
   catálogo. No habrá dos buscadores ni dos bibliotecas.
4. `tracks.db` seguirá siendo la única base. No se creará `search_index.db`.
5. La indexación será progresiva: primero rutas buscables; después duración y
   etiquetas en segundo plano.
6. Desde la primera versión se leerán título, artista, álbum, género, año y número
   de pista cuando existan.
7. El catálogo se dividirá en dos colecciones: **Música** y **Efectos**.
8. El usuario asignará la colección al añadir una carpeta. No se intentará adivinar
   por duración, nombre o contenido si un audio es música o efecto.
9. Los solapamientos entre raíces se detectarán y resolverán sin duplicar archivos.
10. Se mantendrán la observación automática mientras la app esté abierta, la
    reconciliación al iniciar y la prueba previa con 100.000 y 250.000 registros.

Decisión aplazada hasta construir la interfaz:

- qué hacen Enter, doble clic y la acción explícita de reproducir al aire;
- cuáles acciones aparecen primero en el menú contextual.

No se debe fijar ese comportamiento por adelantado ni interpretarlo como aprobado.

---

## 1. Problema que debe resolver

El operador necesita encontrar con rapidez un audio que no está necesariamente
asignado a una rejilla, una pestaña, un perfil, los botones fijos o la cola del
reproductor. El catálogo puede contener efectos cortos, canciones largas y más de
100.000 archivos.

Buscar el disco en cada consulta sería lento e impredecible. El sistema necesita un
índice persistente de una o varias carpetas elegidas por el usuario. El usuario puede
elegir una unidad completa, pero esa no debe ser la opción implícita.

La búsqueda debe tolerar errores razonables de escritura sin devolver coincidencias
arbitrarias. En directo importan tanto la rapidez como la confianza en el resultado.

---

## 2. Requisitos iniciales aportados por el autor

- Usar el panel fijo existente, independientemente de que esté situado a la izquierda
  o a la derecha.
- Permitir una o varias carpetas raíz y, si el usuario lo decide expresamente, una
  unidad completa.
- Clasificar cada carpeta como Música o Efectos.
- Ofrecer búsqueda rápida en el panel fijo y administración completa en una ventana
  Biblioteca.
- Indexar catálogos de 100.000 canciones más efectos de sonido.
- Mantener la interfaz, el audio y el reloj fluidos mientras se indexa.
- Conseguir la primera indexación en pocos segundos en condiciones normales; llegar
  al minuto no debe considerarse normal.
- Actualizar el índice existente de forma incremental.
- Resolver las búsquedas con latencia prácticamente inmediata.
- Incluir búsqueda difusa para errores de escritura razonables.
- Indexar duración y etiquetas musicales; el tiempo adicional es aceptable si se
  informa progreso real y la aplicación continúa disponible.
- Admitir efectos cortos y canciones largas sin intentar cargar la biblioteca en RAM
  como audio.
- Reutilizar las funciones, dependencias y persistencia existentes en la Botonera. No
  crear un segundo catálogo, otro lector de duración ni otra lista de formatos.
- Estudiar LF Automatizador v1.0, adaptar lo bueno y no copiar su implementación.
- Ignorar completamente `C:\LF Automatizador v1.0\LF Automatizador 2.0`.

---

## 3. Precisión necesaria sobre los tiempos

El índice completo incluirá:

- ruta, nombre, extensión y carpeta relativa;
- tamaño y fecha de modificación;
- duración;
- etiquetas disponibles, como título, artista, álbum, género, año y número de pista.

Esto obliga a abrir cada audio al menos una vez. El proyecto actual documenta que
sondear solamente la duración puede costar alrededor de 40 ms por archivo. En un
catálogo de 100.000 archivos, una lectura secuencial con ese coste tardaría unos
4.000 segundos. Leer etiquetas añade trabajo y el almacenamiento puede ser SSD, disco
mecánico, USB o red. Por tanto, «índice completo con duración y etiquetas en pocos
segundos» no es un compromiso técnicamente honesto.

La experiencia sí puede ser progresiva:

1. descubrir y registrar las rutas rápidamente;
2. hacer que esos nombres ya sean buscables;
3. enriquecer duración y etiquetas en segundo plano por lotes;
4. actualizar cada resultado a medida que se completa;
5. no volver a abrir archivos cuyo tamaño y `mtime` no cambiaron.

El estado debe distinguir «archivos descubiertos» de «metadatos completados». Finalizar
la primera fase no se presentará falsamente como indexación completa. Tampoco se
calcularán LUFS, forma de onda o PCM durante la catalogación: esos análisis existentes
decodifican el audio completo y pertenecen al editor o a una acción explícita.

Una actualización individual sí puede reflejarse en milisegundos mediante eventos del
sistema de archivos. Una comprobación completa de 100.000 rutas siempre necesita
recorrer el almacenamiento y se medirá en segundos. La interfaz debe distinguir:

- **sincronización por eventos:** altas, cambios, movimientos y borrados individuales;
- **reconciliación completa:** recorrido explícito para recuperar eventos perdidos o
  cambios ocurridos mientras la aplicación estaba cerrada.

No se prometerá un tiempo idéntico para SSD, disco mecánico, USB, red, antivirus y una
unidad completa. Sí se garantizará que el trabajo nunca bloquee la interfaz ni el
audio.

---

## 4. Propuesta independiente anterior a revisar LF Automatizador

### 4.1 Panel fijo y ventana Biblioteca

`fixed_panel.view` tiene hoy dos valores: `buttons` y `player`. Se añadirá `search`
como tercera presentación. La posición, el ancho, la visibilidad y el botón de
mostrar/ocultar siguen perteneciendo al mismo panel fijo.

La vista de búsqueda tendrá su propio encabezado, entrada, estado del índice y lista
de resultados. No reutilizará la lista del reproductor ni la rejilla de botones como
almacén: compartir superficie visual no significa mezclar estados.

El valor predeterminado seguirá siendo `player`. Cualquier campo nuevo del modelo
llevará `#[serde(default)]`.

La ventana **Biblioteca** mostrará todo el catálogo y permitirá:

- cambiar entre Música y Efectos;
- buscar y ordenar;
- revisar duración, etiquetas, carpeta y estado;
- añadir, unificar, reclasificar, actualizar o quitar raíces;
- ver el progreso y los errores de indexación.

La ventana no tendrá una base ni un proceso de indexación propios. Consultará el mismo
motor de `AppState` que el panel. Cerrar la ventana no detendrá el catálogo ni el audio.

La Biblioteca tendrá un acceso propio que se diseñará con su ventana. No colocar
`Abrir Biblioteca` dentro de la vista Buscador del panel fijo.

Las listas extensas no tendrán paginación visible. Usarán una lista virtual con carga
perezosa: se renderizan las filas visibles y un margen de 50 resultados por
encima y 50 por debajo. Al desplazarse con rueda, barra de scroll o teclado, la UI
pedirá el siguiente bloque en la dirección necesaria y retirará del DOM las filas
lejanas. El cursor y los bloques pertenecen al protocolo interno Rust; el usuario
percibe una lista continua.

La navegación básica usará controles y semántica nativos, foco visible y nombres
accesibles. Esto permite una base útil con lector de pantalla sin declarar terminada
la auditoría de accesibilidad de toda la aplicación.

### 4.2 Motor Rust propio

La propuesta es un `SearchEngine` o `LibraryEngine` dentro de `engine/`, compartido
desde `AppState`, con estas responsabilidades:

- administrar las raíces;
- recorrer el sistema de archivos fuera del hilo de interfaz y de los hilos de audio;
- persistir el índice;
- mantenerlo mediante eventos y reconciliaciones;
- ejecutar y ordenar las consultas;
- emitir progreso y estado reales.

La UI solo enviará comandos y representará resultados. No recorrerá carpetas, no
calculará similitud y no decidirá qué coincidencia es válida.

### 4.3 Ampliar `tracks.db` como única fuente de verdad

La revisión del código de la Botonera cambia la propuesta inicial: **no se creará
`search_index.db`**. Ese archivo repetiría ruta, tamaño, `mtime` y duración que ya
pertenecen a `tracks.db`.

Se ampliará el esquema existente mediante `PRAGMA user_version`:

```text
track
  datos técnicos existentes: path, mtime, size, duration_s, sample_rate, channels
  ediciones existentes: cue, ganancia, normalización, análisis, last_played

library_root
  carpeta elegida, colección Música/Efectos, recursive, enabled
  state, generación y último escaneo

track_catalog
  relación con track y library_root
  colección efectiva Música/Efectos
  nombre visible, carpeta relativa, extensión
  título, artista, álbum, género, año, número de pista
  estado de metadatos y generación vista

track_catalog_fts
  índice derivado de los campos buscables
```

El diseño exacto puede ajustar nombres y relaciones después de la prueba técnica, pero
debe respetar estas reglas:

- la duración vive una sola vez, en `track.duration_s`;
- tamaño y `mtime` viven una sola vez, en `track`;
- las etiquetas viven una sola vez, en el registro de catálogo;
- cue, ganancia, normalización, LUFS y `last_played` no se copian;
- las raíces viven en SQLite, no también en `botonera_config.json`;
- cada archivo tiene una sola fila y una sola colección efectiva;
- FTS contiene una representación derivada necesaria para buscar, no una segunda
  fuente editable;
- quitar una raíz del buscador no puede borrar cue, ganancia ni análisis de una pista.

El worker de catálogo puede usar otra conexión al mismo `tracks.db`, abierta por
`engine/persist/db.rs`. Eso no es otra base ni otra fuente de verdad: WAL y
transacciones cortas permiten que el catálogo no retenga el `Mutex<TrackStore>` ni
bloquee el historial de reproducción mientras lee archivos.

### 4.4 Una sola definición de archivo compatible

La lista de extensiones ya vive en `engine/audio/formats.rs`. El buscador debe usar esa
misma definición. No se mantendrá una segunda expresión regular o lista de formatos.

También existen y deben reutilizarse:

- `audio_files_recursive()` para recorrer carpetas sin recursión de pila;
- `is_audio_path()` y `validate_audio_file()` para formatos y validación;
- `probe_duration_secs()` y la dependencia ya instalada `lofty`;
- `db::normalize_key()` para la semántica distinta de rutas en Windows y Linux;
- `TrackMeta::matches()` para decidir por tamaño y `mtime` si un dato sigue vigente;
- `TrackStore::upsert()` para refrescar datos técnicos sin pisar cue, ganancia,
  normalización ni `last_played`;
- el patrón `spawn_blocking` más progreso por lotes usado al añadir carpetas al
  reproductor.

No se copiará `audio_files_recursive()` dentro del motor. Si necesita cancelación,
progreso o resultados por lote, se evolucionará la utilidad compartida y sus
consumidores conservarán una única implementación.

La lectura de duración y etiquetas también tendrá una sola implementación compartida.
El comportamiento debe proteger una garantía que ya existe: una etiqueta antigua mal
codificada no puede impedir obtener la duración. La sonda nueva intentará propiedades
y etiquetas conjuntamente y, si falla por texto, recuperará al menos las propiedades
con `read_tags(false)`. Los errores de una pista se registrarán sin abortar la raíz.

### 4.5 Índice inicial

- Registrar una raíz debe ser instantáneo y no bloquear el diálogo.
- La indexación será una orden separada y cancelable.
- No se seguirán enlaces simbólicos ni uniones de directorio de forma predeterminada.
- Se omitirán rutas sin permiso y se contará el error; no se silenciará el resultado.
- Una raíz desconectada se marcará como no disponible. No se borrarán sus filas como
  si todos los audios hubieran desaparecido.
- Las escrituras SQLite se harán en transacciones breves por lotes.
- Nunca se mantendrá una transacción abierta mientras se consulta el disco.
- El índice anterior seguirá disponible durante una actualización.
- Solo al completar una generación se marcarán como ausentes las filas no vistas.
- La lectura de duración y etiquetas se ejecutará fuera de los hilos de UI y audio,
  con concurrencia limitada y medida para no saturar el disco.
- Los lotes descubiertos serán buscables mientras continúa el enriquecimiento.
- No habrá un límite duro de 100.000 archivos. Las pruebas cubrirán al menos 250.000.

### 4.5.1 Colecciones y unificación de raíces

Cada raíz se añade como `music` o `effects`. Esta elección es explícita porque una
duración corta no demuestra que algo sea un efecto y una duración larga no demuestra
que sea música.

Antes de guardar una raíz se normalizan y comparan todas las rutas:

- **La misma ruta ya existe:** no se añade otra fila; se informa que ya está
  indexada.
- **Se añade una subcarpeta ya cubierta por una raíz de la misma colección:** se
  informa que ya está incluida y no se crea otra raíz.
- **Se añade una raíz que contiene subcarpetas existentes de la misma colección:**
  se avisa que se unificarán; la raíz nueva sustituye esas entradas específicas sin
  volver a crear los archivos.
- **Raíz y subcarpeta pertenecen a colecciones distintas:** ambas reglas se conservan.
  La ruta más específica manda dentro de su árbol y actúa como excepción. La raíz
  general no duplica ni reclasifica esos archivos.

Ejemplo: `D:\Audio` puede ser Música y `D:\Audio\Efectos` puede ser Efectos. Los
archivos de la subcarpeta pertenecen solo a Efectos; el resto de `D:\Audio`, solo a
Música.

Si se cambia la colección de una raíz, se reclasifican sus archivos sin volver a leer
duración o etiquetas cuando tamaño y `mtime` no cambiaron.

### 4.6 Actualización incremental

Es razonable evaluar la dependencia Rust `notify`, que usa los mecanismos nativos de
Windows y Linux. No debe convertirse en la única garantía de consistencia: su propia
documentación advierte que los observadores pueden perder eventos en árboles muy
grandes o algunos sistemas de archivos.

Diseño recomendado:

1. El observador recibe crear, modificar, mover y eliminar.
2. Un debounce agrupa ráfagas generadas por una sola operación.
3. El worker actualiza únicamente las rutas afectadas mediante una transacción corta.
4. Un error o desbordamiento marca la raíz como necesitada de reconciliación.
5. La reconciliación manual o programada recupera cualquier evento perdido.

Agregar `notify` requerirá antes justificar mantenimiento, licencia, impacto de build
y comportamiento real en Windows y Linux.

### 4.7 Búsqueda difusa por etapas

No se recomienda una única comparación permisiva. El ranking debe favorecer:

1. nombre exacto;
2. palabra o nombre que empieza por la consulta;
3. subcadena contigua;
4. coincidencia aproximada con pocos errores;
5. coincidencia en carpeta relativa, con menor peso.

Normalización propuesta:

- minúsculas;
- equivalencia con y sin diacríticos;
- guiones, puntos y guiones bajos tratados como separadores;
- espacios repetidos compactados;
- comparación por palabras además del nombre completo.

Para evitar resultados «a lo loco»:

- con uno o dos caracteres se permitirá exacto o prefijo, no fuzzy amplio;
- la tolerancia crecerá con la longitud de la palabra;
- la aproximación tendrá un umbral mínimo;
- se devolverá un número acotado de resultados;
- los empates tendrán orden estable.

La SQLite incluida por `rusqlite` ya compila FTS5. Su tokenizer `trigram` permite
recuperar subcadenas rápidamente. Sin embargo, FTS5 por sí solo no corrige todos los
errores de escritura. La opción preferida para una prueba de concepto es:

1. recuperar candidatos por prefijo, subcadena y trigramas en SQLite;
2. reordenar un conjunto acotado en Rust;
3. aplicar distancia acotada, preferiblemente con transposición, sobre palabras;
4. no recorrer las 100.000 filas en cada pulsación.

`nucleo-matcher` puede evaluarse para coincidencias con huecos y ranking rápido, pero
no se aprobará solo por llamarse fuzzy: su modelo exige que los caracteres de la
consulta aparezcan en orden y no equivale necesariamente a corregir una sustitución o
una transposición. La elección final exige un benchmark y un corpus de errores reales.

### 4.8 Cancela consultas obsoletas

La UI puede aplicar un debounce corto, pero cada consulta llevará un número de
generación. Si el usuario continúa escribiendo, una respuesta antigua no podrá
reemplazar a la nueva.

El límite de resultados se aplicará en Rust. El frontend no recibirá decenas de miles
de filas para filtrarlas.

### 4.9 Uso del resultado sin cargar audio

El catálogo contiene referencias, propiedades y etiquetas; no contiene PCM. `lofty`
lee propiedades y metadatos del contenedor, pero una canción larga no se decodifica
completa ni se precarga por aparecer en resultados.

Al ejecutar una acción se reutilizarán los caminos existentes:

- preescucha: bus `Cue`;
- reproducción directa: motor de efectos;
- añadir a la cola: comandos del reproductor;
- asignar a una rejilla o al panel fijo: modelo `ButtonData` y comandos existentes;
- editor: `tracks.db` y análisis diferido.

El menú contextual queda aprobado en este orden:

1. `Reproducir al aire`.
2. `Escucha previa`.
3. `Añadir al reproductor`.
4. `Editor de pista`.

`Reproducir al aire` tendrá un reproductor compacto identificado visualmente como
`LIVE`. Reutilizará la estructura y los controles visuales de la escucha previa, pero
no su estado ni su ruta de audio. LIVE sale por Programa, obedece al máster y al Stop
general y aplica cue, ganancia y normalización. CUE permanece abajo a la derecha y
LIVE aparece abajo a la izquierda; pueden coexistir sin cubrirse ni detenerse.

Traducciones profesionales aprobadas:

- español: `Reproducir al aire`;
- inglés: `Play on air`;
- portugués de Brasil: `Reproduzir no ar`;
- portugués de Portugal: `Reproduzir no ar`.

Enter y doble clic continúan aplazados.

La lista admite clic único, `Ctrl+clic`, `Shift+clic`, flechas, `Shift+flechas`,
Page Up/Down, Escape y apertura del menú mediante la tecla Menú o `Shift+F10`. La
selección vive por ruta estable aunque la virtualización retire sus filas del DOM.
Con una selección múltiple, LIVE, CUE y editor se deshabilitan; `Añadir al
reproductor` añade todas las pistas en orden con una sola persistencia.

Al arrastrar, una pista sobre una celda se copia como botón y pide confirmación si la
celda está ocupada. Una o varias pistas sobre una pestaña se copian en orden a los
primeros espacios vacíos. Si no hay capacidad para todo el lote, Rust rechaza la
operación completa sin dejar una importación parcial.

El encabezado visible del panel fijo será también el acceso directo para alternar
`Botones fijos`, `Reproductor` y `Buscador`. Los Ajustes podrán conservar la
preferencia persistente, pero no serán el único camino para cambiar de vista.

Para anchos reducidos, cada idioma definirá una etiqueta normal y una compacta
comprensible. La interfaz elegirá la variante según el ancho disponible medido; no
truncará texto de forma arbitraria. En español, la referencia aprobada es `Botones
fijos` / `B. fijos`.

---

## 5. Auditoría de LF Automatizador v1.0

### 5.1 Alcance auditado

Se revisó `C:\LF Automatizador v1.0`.

Se excluyó completamente:

`C:\LF Automatizador v1.0\LF Automatizador 2.0`

El Automatizador contiene dos estados que no deben confundirse:

- `main`/versión publicada 0.9.17: índice SQLite y búsqueda Fuse.js cacheada dentro de
  un worker de Node.
- rama de trabajo `rust-core-migration`: migración todavía no consolidada hacia
  scanner, metadata, SQLite y búsqueda `nucleo-matcher` en Rust.

La segunda es evidencia experimental, no una arquitectura publicada que se pueda dar
por probada.

### 5.2 Lo que conviene adaptar

- Raíces persistentes con opción recursiva y estado.
- Registrar una carpeta sin escanearla inmediatamente.
- Trabajo pesado fuera de la UI y del proceso principal.
- Progreso real durante la sincronización.
- Tiempo total visible al terminar, útil para soporte remoto.
- Firma por tamaño y `mtime` para no releer archivos sin cambios.
- Estado de ausente después de completar un recorrido.
- Transacciones SQLite por lotes.
- No mantener el bloqueo de escritura mientras se leen archivos.
- `busy_timeout` y WAL para reducir errores de base ocupada.
- Priorizar título sobre artista, género, álbum y nombre de archivo cuando existen
  esos metadatos.
- Orden estable de resultados.

### 5.3 Lo que no conviene copiar

- Sí interesa abrir cada archivo para obtener etiquetas y duración. Lo que no conviene
  copiar es hacerlo de manera que la UI espere el catálogo completo o que no exista
  una fase rápida, progreso, cancelación y reutilización por tamaño/`mtime`. El
  historial del proyecto registra lotes de 500 que tardaban cerca de 50 segundos en
  disco mecánico; esa evidencia debe orientar el diseño y las expectativas.
- El scanner tiene un límite duro de 100.000 archivos, justo el tamaño mínimo que
  debe soportar la Botonera.
- No hay observador del sistema de archivos; la actualización depende de sincronizar
  manualmente.
- El diseño publicado necesitó dos conexiones y sufrió `database is locked` antes de
  acortar las transacciones.
- La migración Rust actual carga todas las filas SQLite y puntúa todo el catálogo en
  cada consulta. Eso debe medirse antes de considerarlo apto para 100.000 o 250.000.
- La migración Rust usa `nucleo-matcher`, adecuado para caracteres en orden con
  huecos, pero no demuestra por sí sola tolerancia a todas las faltas ortográficas.
- Scanner, formatos y otros consumidores mantienen listas de extensiones separadas.
- Varios módulos superan ampliamente las 200 líneas; no encajan con las reglas de la
  Botonera.
- El índice mezcla catalogación musical, tipos, géneros y metadatos. La primera fase
  de la Botonera debe ser más pequeña y medible.

### 5.4 Conclusión de la comparación

La comparación y la revisión del código propio dejan estas decisiones:

- motor Rust separado;
- catálogo persistente integrado en `tracks.db`, no una segunda base;
- UI delgada;
- trabajo asíncrono;
- lotes y progreso;
- descubrimiento rápido seguido de duración y etiquetas progresivas;
- eventos rápidos más reconciliación completa;
- ranking híbrido con umbrales.

El Automatizador refuerza la necesidad de medir el tiempo real y de no bloquear la UI.
También demuestra que leer etiquetas tiene un coste apreciable; no es razón para
eliminarlas, sino para enriquecer por lotes y comunicar el progreso con honestidad.

### 5.5 Pruebas reales con las colecciones autorizadas

Pruebas ejecutadas el 2026-07-25 en modo Release y en solo lectura:

- Música: `D:\Music` y `D:\Mis musicas`.
- Efectos: `C:\Efectos Stream`, `C:\Efectos` y
  `D:\Documentos\Efectos para Radio`.
- Total Botonera: 18.201 audios, 435 carpetas y aproximadamente 77,75 GiB.
- Formatos: 17.847 MP3, 246 WAV, 78 WMA, 26 M4A y 4 FLAC.

El recorrido Rust compartido, que no abre el contenido, obtuvo:

- primera ronda con caché fría: 4.483 ms;
- seis rondas posteriores: entre 956 y 1.050 ms;
- cero carpetas o entradas inaccesibles.

La lectura secuencial inicial de duración y etiquetas tardó 480.976 ms. Leyó 18.105
archivos; 476 necesitaron la recuperación sin etiquetas y 96 no entregaron
propiedades. La causa por formato quedó acotada: 78 WMA, 9 MP3 y 9 WAV. Los archivos
fallidos seguirán indexados por nombre y ruta, con estado de metadatos no disponibles.
No se elimina WMA de la lista compartida solo porque `lofty` no lo interprete.

Cobertura total de etiquetas:

- título: 10.638;
- artista: 9.917;
- álbum: 7.035;
- género: 8.882;
- año: 4.150;
- número de pista: 6.116.

Comparación caliente de lectura, sobre los mismos 18.201 archivos:

- 1 trabajador: 6.399 ms;
- 2 trabajadores: 3.665 ms;
- 4 trabajadores: 2.907 ms;
- 8 trabajadores: 3.118 ms;
- segunda ronda con 4 trabajadores: 2.809 ms.

Se adopta un máximo de cuatro trabajadores, reducido si el equipo ofrece menos
paralelismo. Ocho no mejora: aumenta la competencia por disco y CPU. No se atribuye
la diferencia entre 480.976 ms y 2.809 ms solo a los trabajadores, porque la primera
lectura partió con caché fría y las posteriores con cabeceras calientes.

La reconciliación sin cambios, comparando únicamente ruta normalizada, tamaño y
`mtime`, tardó 623 ms y no abrió el contenido de ningún audio.

La separación real por colección mostró:

- Música: 17.240 archivos; mediana 220,317 s; percentil 95 de 396,382 s; 87 errores
  de propiedades.
- Efectos: 961 archivos; mediana 5,016 s; percentil 95 de 85,499 s; 9 errores de
  propiedades.

Esto confirma que la categoría no debe deducirse por duración: existen efectos largos
y al menos una pista musical excepcionalmente larga.

### 5.6 Ejecución comparativa del motor real de LF Automatizador

Se ejecutó directamente
`C:\LF Automatizador v1.0\audio-engine-rust\target\release\lf-audio-engine.exe`.
Cada ronda usó una base SQLite temporal nueva, registró las cinco raíces mediante el
protocolo real y ejecutó dos `syncAllRoots`. Las bases y sus archivos WAL/SHM se
eliminaron y se verificó su ausencia al terminar.

Resultados:

- ronda A: primera indexación 12.905 ms; segunda sin cambios 11.689 ms;
- ronda B: primera indexación 11.963 ms; segunda sin cambios 10.224 ms;
- 18.123 archivos, cero errores informados y base de 15.060.992 bytes en ambas
  rondas.

El Automatizador omite los 78 WMA porque su lista de formatos no los incluye. Su
segunda sincronización vuelve a abrir metadatos cuando la pista ya existe: deja de
ingerir filas, pero conserva casi todo el coste. Por eso no se copiará esa condición.
La Botonera decidirá primero por tamaño y `mtime`; solo los archivos nuevos o
modificados pasarán a los trabajadores de metadatos.

---

## 6. Presupuestos de rendimiento que deben aprobarse

Valores propuestos para pruebas reproducibles; no son todavía compromisos publicados:

- Catálogo mínimo de prueba: 100.000 archivos.
- Catálogo de margen: 250.000 filas.
- Descubrimiento inicial de 100.000 rutas en SSD local: objetivo provisional de 10
  segundos o menos; se medirá antes de aprobarlo.
- Enriquecimiento completo de duración y etiquetas: presupuesto pendiente del
  benchmark real. Se informarán velocidad, tiempo restante y errores; no se fijará
  arbitrariamente un minuto para todo tipo de almacenamiento.
- Ninguna operación debe congelar la UI durante más de un frame perceptible.
- Consulta ya indexada: p95 del motor menor de 30 ms para 100.000 filas.
- Resultado visible, incluido IPC: p95 menor de 100 ms después del debounce.
- Evento individual reflejado en el índice: objetivo menor de 250 ms.
- Memoria y tamaño de base deben medirse con 100.000 y 250.000 filas.
- Los resultados exactos y por prefijo deben permanecer estables aunque se active la
  tolerancia difusa.

Discos mecánicos, USB, red, antivirus y unidades completas se informarán por separado.
La aplicación mostrará archivos recorridos, encontrados, omitidos, fallidos y tiempo
real, sin simular porcentajes.

---

## 7. Pruebas necesarias antes de declarar la arquitectura cerrada

### Ranking

Crear un corpus con nombres reales y errores como:

- letra omitida;
- letra añadida;
- letra sustituida;
- dos letras intercambiadas;
- acento ausente;
- palabras en orden diferente;
- nombres muy cortos;
- términos comunes que no deben inundar los resultados.

Cada caso debe fijar qué resultado gana y qué falsos positivos son inaceptables.

### Persistencia e indexación

- dos raíces independientes;
- raíz duplicada y raíz anidada;
- alta, modificación, movimiento y borrado;
- aplicación cerrada durante los cambios;
- raíz USB desconectada;
- carpeta sin permisos;
- enlace simbólico o unión circular;
- cancelación a mitad del primer índice;
- cierre inesperado sin dejar una generación parcial como vigente.

### Rendimiento

- índice sintético SQLite de 100.000 y 250.000 filas;
- árbol real en SSD;
- árbol real en disco mecánico si está disponible;
- consulta exacta, prefijo, subcadena, error y término sin resultados;
- escritura del observador mientras se consulta;
- reproducción de efectos y música durante una reconciliación.

### Plataformas

- Windows Release;
- Linux Release después de retomar la prueba física;
- rutas con espacios, acentos y caracteres no ASCII;
- sistemas sensibles e insensibles a mayúsculas.

---

## 8. Decisiones todavía abiertas

1. Acción principal de Enter y doble clic:
   - preescuchar;
   - reproducir al aire;
   - añadir a la cola.
2. Acciones secundarias que entran en la primera versión:
   - añadir al reproductor;
   - asignar a la pestaña activa;
   - asignar al panel fijo;
   - abrir editor;
   - mostrar en el explorador.
3. Las etiquetas están incluidas; queda por decidir cuáles se muestran
   en cada fila y cuáles solo participan en la búsqueda.
4. Si la vigilancia puede deshabilitarse por raíz.
5. Presupuestos definitivos de tiempo, memoria y tamaño tras las mediciones.
6. Dependencia de observación y estrategia final de fuzzy después del benchmark.

---

## 9. Orden recomendado de trabajo

1. Crear y probar el modelo de raíces, colecciones y unificación.
2. Cerrar presupuestos y corpus de ranking.
3. Hacer una prueba técnica aislada de SQLite FTS5, ranking y 250.000 filas, más un
   corpus real para medir lectura de duración y etiquetas con `lofty`.
4. Decidir dependencias con evidencia.
5. Aprobar esquema, módulos, IPC y eventos.
6. Actualizar reglas, arquitectura, glosario, libro y changelog.
7. Implementar primero persistencia e indexador, con tests.
8. Implementar consulta y ranking, con benchmarks.
9. Implementar observador y reconciliación.
10. Integrar la tercera vista del panel.
11. Conectar las acciones a los flujos de audio ya existentes.
12. Ejecutar tests, builds y prueba física Release.

La arquitectura base está aprobada. Se puede implementar persistencia, raíces,
colecciones, indexación y búsqueda. Las acciones de reproducción de la interfaz
permanecen bloqueadas hasta la conversación correspondiente.

### Avance registrado

Completado el 2026-07-25:

- migración segura de `tracks.db` del esquema 1 al 2;
- tabla `library_root` con ruta única y colección Música/Efectos;
- protección contra rebajar una base creada por una versión futura;
- regla pura y probada para detectar cobertura, unificación, excepciones y cambio de
  colección;
- prioridad de la raíz más específica en árboles con categorías alternadas.
- recorrido único compartido entre la importación existente y el indexador;
- sello de archivo centralizado, sin una segunda implementación;
- lector único de duración y etiquetas con recuperación ante tags defectuosos;
- lotes de metadatos con concurrencia acotada y orden estable;
- benchmarks reales detallados en las secciones 5.5 y 5.6.

Completado en la siguiente etapa del 2026-07-25:

- esquema 3 con catálogo persistente y FTS5 dentro del mismo `tracks.db`;
- altas, reclasificación, unificación y retirada de múltiples raíces por colección;
- descubrimiento y enriquecimiento incremental por lotes;
- búsqueda normalizada y difusa con candidatos acotados y ranking estable;
- servicio único reutilizable por panel fijo y Biblioteca;
- IPC de raíces, sincronización, estado y consulta, más progreso por evento;
- pruebas de 100.000 y 250.000 filas y prueba integral con las cinco raíces reales.

Resultados Release del índice sintético:

- 100.000 pistas: 4.178 ms para construir; búsqueda con error, 11,5 ms de media;
- 250.000 pistas: 10.984 ms para construir; búsqueda con error, 29,4 ms de media y
  34,5 ms en p95.

La primera prueba integral sobre 18.201 audios tardó 257.471 ms porque leyó duración y
etiquetas desde disco. Las dos reconciliaciones posteriores tardaron 2.805 y 2.792 ms,
con cero reaperturas de metadatos. La base fue temporal y se eliminó al terminar.

Se añadió `unicode-normalization` 0.1 para NFD y eliminación de diacríticos. Es una
dependencia pequeña, mantenida, con licencias MIT/Apache-2.0 compatibles; evita una
normalización parcial hecha a mano. No se añadió un segundo motor fuzzy ni una base
separada.

Observación incremental completada el 2026-07-25:

- `notify` 8.2 con observación recursiva nativa y debounce propio de 250 ms;
- actualización puntual para archivos creados, modificados, movidos o borrados;
- reconciliación de la raíz ante eventos de directorio o errores del observador;
- reconciliación completa en segundo plano al iniciar;
- refresco de las rutas observadas al añadir, unificar o retirar raíces;
- prueba real temporal de crear/modificar/borrar aprobada;
- actualización Release de un archivo real: 9,6 ms fría y 3,3 ms caliente.

`notify` 8.2 requiere Rust 1.77, usa licencia CC0-1.0 compatible y añade los adaptadores
nativos por plataforma. No sustituye la reconciliación porque su documentación
advierte limitaciones y pérdidas posibles en árboles grandes o ciertos sistemas de
archivos.

Carga perezosa bidireccional completada el 2026-07-25:

- esquema 4 con índices de recorrido por colección, presencia, nombre y ruta;
- comando `library_browse` con cursor estable y dirección adelante/atrás;
- bloques internos para una lista continua, sin controles de páginas;
- 100.000 pistas: 494 µs primer bloque, 438 µs siguiente y 490 µs anterior;
- 250.000 pistas: 449 µs primer bloque, 451 µs siguiente y 472 µs anterior.

Interfaz del panel implementada el 2026-07-25:

- tercera vista `search` con selector directo en el encabezado;
- nombres normales y compactos localizados según el ancho real;
- campo de búsqueda, filtro Todo/Música/Efectos y lista virtual con margen 50/50;
- selección estable con ratón y teclado y menú contextual accesible;
- adición múltiple al reproductor con una sola persistencia;
- arrastre individual a celdas y por lote a pestañas, con capacidad atómica;
- controles CUE abajo a la derecha y LIVE abajo a la izquierda;
- textos profesionales en los cuatro idiomas.

Siguiente paso: prueba funcional Release y, después, ventana independiente Biblioteca.
Enter y doble clic siguen sin acción hasta una decisión posterior.

---

## 10. Referencias técnicas

- SQLite FTS5 y tokenizer trigram:
  <https://www.sqlite.org/fts5.html>
- `notify`, observación multiplataforma y limitaciones en árboles grandes:
  <https://docs.rs/notify/latest/notify/>
- `nucleo-matcher`, comportamiento y algoritmo:
  <https://docs.rs/nucleo-matcher/latest/nucleo_matcher/>
- Implementación de referencia auditada:
  `C:\LF Automatizador v1.0\audio-engine-rust\src\library\`
- Índice de referencia:
  `C:\LF Automatizador v1.0\audio-engine-rust\src\db\library_index.rs`
