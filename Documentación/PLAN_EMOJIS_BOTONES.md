# Identificadores visuales para los botones

**Estado:** implementación terminada; pendiente de prueba funcional del autor.

**Última actualización:** 2026-07-30.

Este documento conserva las decisiones estables de la función y explica cómo ampliar
los catálogos sin reconstruir el diseño desde una conversación.

## 1. Objetivo

Mejorar la identificación de sonidos a distancia mediante una referencia visual
grande y reconocible, sin obligar a leer siempre el nombre completo del botón.

Cada botón puede presentarse como:

- solo texto;
- emoji o icono monocromático junto al texto;
- solo emoji o icono grande.

Un botón tiene como máximo un identificador visual elegido. El visual no cambia su
tipo, audio, volumen, cue, normalización, atajo, color ni reglas de reproducción.

## 2. Colecciones incorporadas

### 2.1 Emojis

`Emojis` es la primera sección y la opción principal por su color y reconocimiento
rápido. Incluye los 3.953 valores de Unicode Emoji 17.0 y nombres o palabras clave de
CLDR 48.2 en español, inglés, portugués de Brasil y portugués de Portugal.

La fuente Noto Emoji COLRv1 `v2.051` viaja con la aplicación. No depende del repertorio
de Windows o Linux ni realiza descargas durante el uso.

Los 2.035 valores que solo añaden un modificador de tono de piel se ocultan al
principio para reducir repeticiones. Una casilla permite mostrarlos; sin ellos quedan
1.918 valores.

### 2.2 Básicos

`Básicos` reúne todo lo monocromático. El color se hereda del texto del botón y se
adapta al tema:

- 24 Material Symbols Rounded esenciales, cuyos identificadores históricos se
  conservan sin prefijo;
- 4.231 Tabler Icons `v3.46.0`, sin marcas ni variantes terminadas en `-off`;
- 4.133 Game Icons, sin la carpeta auxiliar `badges` y con atribución CC BY 3.0.

El total es de 8.388 conceptos. Los valores nuevos son estables y llevan colección,
categoría e identificador, por ejemplo `tabler:animals:dog` o
`game-icons:weather:thunder-struck`.

Las 134 etiquetas oficiales de Game Icons se conservan en una instantánea local. Se
usan para distribuir los diseños en categorías útiles en vez de amontonarlos en una
categoría genérica.

## 3. Funcionamiento completamente local

Elegir, buscar y dibujar un identificador visual no realiza peticiones de red. Los
recursos, nombres, categorías, palabras clave, fuentes y licencias viajan con la
aplicación.

Los SVG se dividen por categoría y ninguno puede superar 1.250.000 bytes. Así el
navegador solicita solo las categorías que llegan a la ventana virtual y conserva
las ya usadas en su caché local.

## 4. Búsqueda multilingüe

Rust normaliza mayúsculas y acentos, pagina los resultados y valida los valores. Los
catálogos conservan:

- el nombre original en inglés;
- nombres y palabras clave locales;
- términos conceptuales por categoría;
- correcciones manuales para vocabulario importante de radio, sonido, animales,
  oficina, naturaleza, objetos, alertas y transporte.

Se generaron traducciones auxiliares locales para 5.149 palabras. Las correcciones
manuales tienen prioridad. Por eso búsquedas como `perro`, `animales`, `oficina`,
`naturaleza`, `sonido`, `trueno` o `explosión` no requieren saber inglés.

Buscar el nombre singular o plural de una categoría encuentra el conjunto completo.
En `Básicos`, por ejemplo, hay 486 animales, 600 elementos de naturaleza, 449 de
oficina, 412 objetos y 247 conceptos de audio.

## 5. Selector y rendimiento

El selector muestra:

- buscador;
- pestañas `Emojis` y `Básicos`;
- filtro y separadores de categoría;
- nombre de la categoría actual;
- opción de tonos de piel solo para emojis;
- contador de posición y total.

No existe un botón `Mostrar más`. La rueda y la barra recorren la altura completa,
pero el DOM conserva como máximo 300 resultados y un colchón prudente de 100
elementos. Al acercarse a un borde pide otra ventana local y descarta los nodos
lejanos.

Favoritos y recientes quedan para una ampliación posterior: no son necesarios para
la primera entrega y requieren una política propia de persistencia.

## 6. Modelo y compatibilidad

`ButtonData.visual` usa `#[serde(default)]` y se omite al serializar si conserva el
valor histórico:

```text
ButtonVisual {
  kind:  "auto" | "emoji" | "basic"
  value: identificador estable del recurso
  mode:  "text" | "visual_text" | "visual"
}
```

`auto + visual_text` reproduce la presentación anterior. En la interfaz se llama
`Restaurar icono original` y recupera el símbolo propio de audio, carpeta, hora o
clima. El modo `text` se presenta como `Solo texto`.

El nombre del sonido nunca se sustituye por el dibujo. El visual se conserva en
rejillas, botones fijos, reproductor, configuración, respaldo y formatos compartidos
que reutilizan `ButtonData`. Los archivos antiguos siguen abriendo sin migración
obligatoria.

Rust valida que `kind`, `mode` y `value` correspondan a un recurso incorporado antes
de guardar. JavaScript solo presenta el selector, mide su cuadrícula y dibuja el
resultado validado.

## 7. Accesibilidad

El emoji o SVG es decorativo y se marca con `aria-hidden="true"`. El botón completo
conserva como nombre accesible el texto del sonido; el selector da a cada resultado
su nombre localizado.

Esto evita que un lector anuncie primero el nombre Unicode del dibujo y luego el
nombre útil del sonido. La auditoría integral con lector de pantalla pertenece a una
etapa posterior, pero esta función no agrega una lectura redundante.

## 8. Recursos, versiones y licencias

- Unicode Emoji 17.0 y CLDR 48.2:
  <https://unicode.org/emoji/charts/emoji-counts.html>.
- Noto Emoji `v2.051`, commit
  `8998f5dd683424a73e2314a8c1f1e359c19e8742`.
- Material Symbols: Apache License 2.0.
- Tabler Icons `v3.46.0`, commit
  `8ac7d81b72ece11072ef25ea9fd92e80c6f3c9fc`, licencia MIT.
- Game Icons, commit
  `82d948812bfe3f269ef8f731dcdb07b08160edc4`, CC BY 3.0 o CC0 según autor.

Las versiones, repositorios y reglas de selección viven en
`scripts/visual-packs.json`. Las atribuciones redistribuidas se documentan en
`THIRD_PARTY_NOTICES.md` y se empaquetan bajo `legal/`.

## 9. Generación y verificación

Los recursos derivados se reproducen con:

```powershell
npm run visuals:generate-emojis
npm run visuals:generate-basics
npm run visuals:generate-packs
npm run visuals:verify
```

`visuals:verify` comprueba hashes, cantidades, orden entre idiomas, valores únicos,
correspondencia entre catálogo y símbolos SVG, seguridad de las etiquetas, tamaño
máximo por sprite, cobertura mínima de las categorías útiles y las 134 etiquetas de
Game Icons.

La verificación de cierre incluye además:

```powershell
cd src-tauri
cargo test --lib
cargo build --lib
cd ..
npm run build
npm run tauri build -- --no-bundle
```

El renderizado externo de Tabler y Game Icons se comprobó en una página local
Chromium. La prueba real corresponde al autor en el ejecutable Release y WebKitGTK
continúa como validación física posterior de Linux.
