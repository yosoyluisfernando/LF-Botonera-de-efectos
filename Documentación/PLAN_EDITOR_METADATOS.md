# Editor de metadatos y tags de Biblioteca

Estado: implementado y verificado técnicamente el 2026-07-30.

## Objetivo

Permitir que Música y Efectos tengan datos editoriales propios y palabras clave
buscables desde las dos superficies de Biblioteca, sin duplicar el catálogo, sin
destruir los tags originales de los audios y sin poner reglas de negocio en
JavaScript.

## Experiencia de uso

El menú contextual del Buscador fijo y de la ventana Biblioteca presenta
`Editar metadatos…` inmediatamente antes de `Editor de pista`.

Para una pista de Música se muestran nombre de archivo, título, artista, álbum,
artista del álbum, género, año, número de pista, compositor, comentario y tags.
No se incluye BPM.

Para un Efecto se muestran nombre de archivo, nombre descriptivo, categoría,
descripción y tags. El nombre descriptivo permite mejorar la presentación sin
renombrar el archivo físico.

En una selección múltiple de una sola colección, cada campo colectivo tiene una
casilla `Aplicar a todos`. Lo no marcado conserva el valor de cada pista. En una
selección mixta de Música y Efectos solo se modifican tags. El editor muestra los
tags comunes; añadir o quitar uno lo aplica a toda la selección sin borrar tags
particulares no visibles.

## Persistencia y búsqueda

No existe una segunda base. El esquema 6 de `tracks.db` incorpora:

- `track_user_metadata`, una fila opcional por pista para campos elegidos por el
  usuario;
- `track_keyword`, cualquier cantidad de tags ordenados por pista;
- un índice de sugerencias sobre la clave normalizada.

La forma visible del tag se conserva. Para comparar y deduplicar se eliminan
diferencias de mayúsculas, acentos y puntuación equivalente. Guardar un lote ocurre
en una transacción SQLite: si una pista dejó de existir, ninguna se modifica.

El valor efectivo es único para obtención, tabla y búsqueda. Un campo propio manda
sobre lo leído durante la indexación. El FTS5 se reconstruye con nombre, ruta,
metadatos efectivos y tags. Reconciliar o volver a leer el archivo no elimina las
decisiones del usuario.

## Renombrado físico

Renombrar nunca es automático y solo se ofrece para una pista a la vez. Se conserva
la extensión y se rechazan nombres vacíos, reservados, con caracteres no portables,
colisiones, enlaces simbólicos y pistas en reproducción.

La operación coordina:

1. un diario duradero con configuración anterior y nueva;
2. el nombre real del archivo, incluido el cambio solo de mayúsculas en Windows;
3. la clave de `track`, el catálogo, metadatos propios, tags y FTS;
4. las referencias en rejillas, botones fijos globales y por perfil y cola del
   reproductor;
5. el guardado atómico de `botonera_config.json`;
6. la invalidación de cachés de precarga, análisis y forma de onda.

Las etiquetas visibles de botones no cambian. Cue, ganancia, normalización, análisis
e historial siguen unidos a la pista. Si el proceso se interrumpe, el arranque
completa la operación o restaura el estado anterior usando el diario.

## Escritura opcional dentro del audio

Los datos propios se guardan siempre en SQLite. Adicionalmente, una pista individual
de Música puede solicitar `Escribir también en el archivo`. Esa opción nunca aparece
en lotes ni en Efectos.

El backend:

1. rechaza WMA, un formato no reconocido o una pista en reproducción;
2. crea una copia hermana con la misma extensión;
3. escribe solo los campos compatibles solicitados;
4. sincroniza, vuelve a abrir y comprueba duración y valores;
5. registra un diario y sustituye el original conservando un respaldo temporal;
6. confirma SQLite;
7. marca la instalación como confirmada y limpia el respaldo.

Una interrupción no confirmada restaura el original. Una interrupción posterior a la
confirmación termina la limpieza sin deshacer el archivo válido. Los tags propios de
Biblioteca no se escriben en el audio: siguen siendo una herramienta interna.

## Respaldo

`.lfbackup` usa la copia online completa de SQLite, por lo que incluye
`track_user_metadata` y `track_keyword` sin una ruta especial ni dos archivos
adicionales. La restauración valida la versión de esquema antes de instalarla.

## Evidencia automatizada

- lote atómico y rechazo total si falta una pista;
- anulaciones y tags visibles en obtención, recorrido y búsqueda;
- persistencia después de reindexar y eliminación por tag normalizado;
- renombrado con conservación de cue, tags y todas las referencias;
- recuperación después de interrumpir un renombrado físico;
- escritura verificada sin cambiar la duración;
- rollback byte por byte;
- recuperación de escritura no confirmada y confirmada;
- respaldo `.lfbackup` con campos propios y tags;
- copia online desechable de la base real de 38.576.128 bytes;
- búsquedas sintéticas con 100.000 y 250.000 pistas.

La validación visual y accesible del modal se realiza sobre la compilación Release.
