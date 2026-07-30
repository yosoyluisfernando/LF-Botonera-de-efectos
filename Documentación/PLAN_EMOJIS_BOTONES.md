# Identificadores visuales para los botones

**Estado:** diseño funcional aprobado; implementación no iniciada.

**Fecha de la decisión:** 2026-07-30.

Este documento guía la incorporación de emojis e iconos a los botones de LF
Botonera. Conserva las decisiones estables, separa las validaciones pendientes y
evita reconstruir el diseño a partir de una conversación compactada.

## 1. Objetivo

Mejorar la identificación de sonidos a distancia mediante una referencia visual
grande y reconocible, sin obligar a leer siempre el nombre completo del botón.

Cada botón podrá presentarse como:

- solo texto;
- emoji y texto;
- icono básico y texto;
- solo emoji grande;
- solo icono básico grande.

Un botón tendrá como máximo un identificador visual elegido por el usuario: emoji o
icono básico. No se combinarán emoji e icono en la misma celda porque competirían
por el espacio y reducirían el reconocimiento.

La función no cambia el tipo del botón, su audio, volumen, cue, normalización,
atajo, color ni reglas de reproducción.

## 2. Decisiones cerradas

### 2.1 Emojis como catálogo principal

`Emojis` será la primera sección del selector y la opción visual principal. Su color
y variedad favorecen el reconocimiento rápido en una rejilla usada durante una
emisión.

El catálogo funcionará sin Internet y tendrá una apariencia incorporada en la
aplicación. No dependerá de que Windows o Linux incluyan una versión concreta de
Unicode Emoji.

La base visual recomendada es Noto Emoji, fijada a una versión concreta y
redistribuida con sus avisos de licencia. Antes de incorporarla se debe validar qué
formato local ofrece mejor equilibrio entre fidelidad, tamaño del instalador y
rendimiento en WebView2 y WebKitGTK.

### 2.2 Material Symbols como catálogo «Básicos»

`Básicos` será la segunda sección. Usará una selección cuidada de Material Symbols
Rounded, gruesos y preferiblemente rellenos, alojados localmente.

No se expondrán inicialmente miles de nombres técnicos en inglés. La primera
colección contendrá aproximadamente entre 300 y 500 símbolos comprensibles para
radio, streaming y uso general. Podrá ampliarse en versiones posteriores sin cambiar
el formato guardado en los botones.

### 2.3 Funcionamiento completamente local

Elegir, buscar y dibujar un identificador visual no realizará peticiones de red.
Los recursos, nombres, categorías y palabras clave viajarán con la aplicación.

Una actualización futura del catálogo será parte de una nueva versión de LF
Botonera, nunca una descarga silenciosa durante el uso.

### 2.4 Búsqueda en los cuatro idiomas

Los emojis usarán nombres cortos y palabras clave de Unicode CLDR para:

- español;
- inglés;
- portugués de Brasil;
- portugués de Portugal.

La búsqueda ignorará diferencias de mayúsculas y acentos. Además podrá incorporar
sinónimos propios útiles para el contexto de radio, por ejemplo `aplausos`,
`carcajada`, `sirena`, `gol`, `cortina` o `publicidad`.

Material Symbols no ofrece un índice multilingüe equivalente a CLDR. La colección
`Básicos` tendrá un manifiesto local propio con nombre, categoría y palabras clave en
los cuatro idiomas. El usuario nunca necesitará conocer identificadores internos
como `sports_soccer` o `phone_in_talk`.

### 2.5 Nombre textual separado

El emoji o icono no se guardará dentro de `name` ni `label`. El botón conservará un
nombre textual aunque se muestre visualmente solo el dibujo.

Esta separación permite:

- cambiar entre solo texto, visual y texto, o solo visual;
- buscar y editar el botón por un nombre comprensible;
- mantener una base correcta para la futura compatibilidad con lector de pantalla;
- evitar que el lector anuncie el nombre Unicode del emoji además del nombre del
  sonido.

El elemento visual se marcará como decorativo (`aria-hidden="true"`). No se ocultará
el control completo. La auditoría integral de lector de pantalla pertenece a una
etapa posterior, pero esta función no debe crear una barrera nueva.

### 2.6 Comportamiento de los iconos actuales

Hoy la interfaz muestra automáticamente un pequeño icono de tipo para audio, carpeta
aleatoria, hora, temperatura y humedad.

- Un botón antiguo o sin identificador elegido conservará la presentación actual.
- Al elegir un emoji o icono básico, ese visual sustituirá al icono automático de
  tipo para no mostrar dos símbolos en competencia.
- Quitar el identificador elegido restaurará el comportamiento automático.

## 3. Experiencia del selector

El editor del botón incorporará una sección `Identificador visual`. El selector debe
ser operable sin conocer nombres en inglés y presentará:

- buscador;
- `Emojis` como sección inicial;
- `Básicos` como segunda sección;
- categorías;
- usados recientemente;
- favoritos;
- opción automática según el tipo;
- opción sin identificador visual.

Las categorías de emojis seguirán el orden de Unicode cuando sea útil. Además, la
presentación o los términos de búsqueda deben facilitar usos frecuentes en radio:

- música y producción;
- aplausos, público y reacciones;
- humor;
- alarmas y emergencias;
- teléfonos y comunicaciones;
- noticias;
- deportes;
- clima;
- animales;
- vehículos;
- transiciones y separadores;
- publicidad y comercio;
- personas y voces.

El selector no debe insertar todos los elementos en el DOM simultáneamente. Usará
carga perezosa o lista virtual y cargará únicamente las miniaturas visibles y un
margen pequeño.

## 4. Presentación en los botones

Los tres modos persistentes serán conceptualmente:

- `text`: solo texto;
- `visual_text`: visual y texto;
- `visual`: solo visual.

Los nombres técnicos finales se confirmarán antes de modificar el modelo.

El tamaño debe derivarse del espacio real de la celda, no de una medida fija:

- con texto, el visual ocupa una parte principal sin tapar el nombre;
- sin texto visible, el visual crece y se centra;
- índice, atajo, temporizador y barra de progreso conservan zonas propias;
- la presentación debe funcionar en la rejilla principal y en botones fijos de
  diferentes dimensiones.

El visual elegido también debe conservarse cuando el mismo `ButtonData` viaja a la
cola del reproductor. La presentación concreta en una fila estrecha puede ser más
pequeña, pero no debe usar otra fuente de verdad.

## 5. Arquitectura propuesta

La decisión permanente es separar el visual del nombre. La forma exacta del modelo
debe aprobarse antes de programar porque afecta `ButtonData`, IPC y formatos
compartidos.

La base recomendada es un valor opcional con campos de cadena compatibles hacia
delante:

```text
ButtonVisual {
  kind:  "auto" | "emoji" | "basic" | "none"
  value: identificador estable del recurso
  mode:  "text" | "visual_text" | "visual"
}
```

`ButtonData` lo recibiría mediante `#[serde(default)]`. El valor predeterminado debe
reproducir la apariencia actual de los botones antiguos.

Rust será responsable de:

- validar `kind`, `value` y `mode`;
- resolver el catálogo según el idioma;
- normalizar y buscar nombres, acentos y palabras clave;
- guardar la elección;
- devolver contratos de vista válidos;
- importar y exportar el dato.

JavaScript será responsable únicamente de:

- mostrar el selector y sus resultados;
- solicitar búsquedas;
- dibujar el recurso validado;
- medir el espacio disponible y aplicar la distribución visual;
- enviar la elección mediante IPC.

No se creará una base SQLite para un catálogo estático de unos pocos miles de
elementos. El catálogo será un recurso versionado generado durante el desarrollo y
leído localmente.

## 6. Recursos y licencias

Fuentes oficiales consultadas para el diseño:

- Unicode Emoji 17.0 registra 3.953 emojis contando variantes y secuencias:
  <https://unicode.org/emoji/charts/emoji-counts.html>.
- Unicode CLDR mantiene nombres y palabras clave por idioma, destinados entre otros
  usos a selectores de caracteres:
  <https://www.unicode.org/cldr/charts/latest/annotations/index.html>.
- Noto Emoji ofrece fuentes de color, SVG y PNG. Las fuentes usan SIL Open Font
  License 1.1; las herramientas y la mayoría de recursos gráficos, Apache 2.0:
  <https://github.com/googlefonts/noto-emoji>.
- Material Symbols se distribuye bajo Apache License 2.0 y ofrece variantes Rounded,
  ejes de grosor y relleno:
  <https://github.com/google/material-design-icons>.

Antes de añadir archivos al repositorio se debe registrar:

- versión o commit exacto;
- archivos incorporados y proceso de generación;
- licencia y avisos que deben redistribuirse;
- tamaño añadido al repositorio y a los instaladores;
- resultado en Windows y Linux;
- ausencia de dependencias de red.

No se incorporará un paquete NPM no mantenido oficialmente si los recursos originales
pueden procesarse durante el desarrollo y almacenarse como archivos locales estables.

## 7. Compatibilidad y portabilidad

El visual forma parte del contenido del botón y debe acompañarlo en:

- `botonera_config.json`;
- rejillas y botones fijos;
- cola y listas `.LFPlay`;
- exportación e importación `.bdelf`;
- exportación e importación `.bdeplf`;
- respaldo `.lfbackup`.

Los catálogos serán comunes a todas las instalaciones, por lo que se guardará un
identificador estable, no una ruta absoluta a una imagen.

Los campos exportados deben ser opcionales. LF Automatizador podrá ignorarlos sin
romper archivos antiguos o nuevos, pero para conservar el visual después de un viaje
Botonera → Automatizador → Botonera será necesario coordinar el cambio en ambas
aplicaciones.

Las imágenes personales o arbitrarias quedan fuera de la primera versión. Requerirían
administración de archivos, empaquetado portátil y reglas de seguridad diferentes.

## 8. Etapas de implementación

### Etapa 1: prueba técnica de recursos

- comparar fuente, SVG, PNG o atlas para Noto Emoji;
- medir tamaño real y tiempo de carga;
- probar WebView2 y WebKitGTK;
- fijar versión y licencias;
- definir la selección inicial de `Básicos`.

No se modifica `ButtonData` durante esta prueba.

### Etapa 2: catálogo y búsqueda

- generar el catálogo local de emojis desde Unicode y CLDR;
- generar el manifiesto multilingüe de `Básicos`;
- implementar normalización y búsqueda en Rust;
- probar acentos, sinónimos, idiomas y resultados estables;
- medir consultas y consumo de memoria.

### Etapa 3: modelo e IPC

- aprobar la forma definitiva de `ButtonVisual`;
- añadir `#[serde(default)]`;
- validar valores en Rust;
- extender actualización de botones y contratos de vista;
- probar configuraciones antiguas y valores desconocidos.

### Etapa 4: selector y presentación

- construir el selector virtualizado;
- incorporar favoritos y recientes;
- aplicar los tres modos visuales;
- compartir el pintor entre rejilla, panel fijo y reproductor;
- conservar índice, atajo, tiempo y progreso.

### Etapa 5: formatos compartidos y documentación

- extender `.bdelf`, `.bdeplf` y `.LFPlay`;
- coordinar LF Automatizador si debe preservar los campos;
- comprobar `.lfbackup`;
- actualizar arquitectura, libro, glosario y changelog.

### Etapa 6: verificación

- `cargo test --lib`;
- `cargo build --lib`;
- `npm run build`;
- `npm run tauri build -- --no-bundle`;
- auditoría de archivos de más de 200 líneas;
- prueba funcional Release con distintas rejillas, temas y escalas;
- prueba posterior con lector de pantalla cuando comience esa etapa.

## 9. Decisiones pendientes antes de programar

Aunque el diseño funcional está aprobado, todavía se debe cerrar:

1. formato exacto para empaquetar Noto Emoji después de medirlo;
2. contenido y cantidad final de la primera colección `Básicos`;
3. forma definitiva de `ButtonVisual` y nombres de sus valores;
4. ubicación de favoritos y recientes: configuración persistente o estado local;
5. comportamiento exacto de exportación al pasar por LF Automatizador;
6. tamaños y distribución definitivos después de una maqueta con rejillas reales.

Estas decisiones no autorizan cambios de código hasta que el autor apruebe la
arquitectura resultante.
