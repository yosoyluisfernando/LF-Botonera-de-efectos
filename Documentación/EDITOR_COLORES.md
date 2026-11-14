# Editor unificado de colores — integración autorizada

## Solicitud del autor — 2026-11-14

Una sola lista de relleno en el editor del botón, con Predefinidos primero y
seleccionado para botones nuevos o antiguos sin estilo avanzado. Después:
Favoritos, Color sólido, Degradado horizontal, Degradado vertical, Degradado diagonal ↘
y Degradado diagonal ↗. Sustituye los dos bloques superpuestos del fork.

## Comportamiento propuesto

- Predefinidos muestra los controles de la paleta oficial existente y conserva
  su adaptación al tema. Oculta HEX, segundos colores y opciones avanzadas.
- Color sólido muestra selectores libres de fondo/texto y entradas HEX.
- Los cuatro degradados añaden un segundo color de fondo. Se conserva la
  opción de texto en degradado de Danilo, mostrando su segundo color al activarla.
- Color exacto conserva la intención de Danilo: los colores personalizados
  se muestran tal como se eligen; los predefinidos siguen adaptándose al tema.
- Una única vista previa permanece visible en todos los modos y se actualiza
  al elegir la paleta, mover selectores, escribir un HEX válido, cambiar modo,
  dirección o texto. Los HEX incompletos no se guardan y muestran validación.
- Cambiar el borrador solo actualiza la vista previa; Aceptar persiste y
  Cancelar descarta los cambios de estilo pendientes.
- Al reabrir un botón se recupera el modo que corresponde a su estilo guardado.
- No se incorpora el tamaño de fuente por botón del fork: no fue seleccionado.

## Cambio de datos y módulos autorizado (§2.9 de AGENTS.md)

Añadir a ButtonData un estilo opcional, con serde(default) y omisión cuando
sea predeterminado. Adaptar ButtonStyle de Danilo conservando los nombres
style.gradient, style.bg2, style.text2 y style.exact; excluir font_size.
color_bg y color_text conservan su función y sus valores en botones anteriores.
No se añade un catálogo ni dependencias. Predefinidos equivale al estilo vacío.

Rust valida las direcciones y los colores antes de guardar mediante los comandos
existentes de edición de botones de rejilla y panel fijo. Estos comandos reciben
el estilo como argumento opcional. La ampliación posterior de favoritos añade
los tres comandos descritos abajo; no hay eventos nuevos.
La importación/exportación incluye el estilo opcional. Una versión anterior del
Automatizador puede ignorarlo y mostrar el primer color; no se promete conservar
el degradado en una ida y vuelta por una aplicación que desconozca ese campo.

Separar edición visual y representación en módulos menores de 200 líneas.
La vista previa y los botones reales usan la misma función para pintar el estilo.
JavaScript solo sincroniza controles y dibuja el borrador; Rust valida/persiste.
Adaptar también la representación en rejilla y botones fijos, respetando estados
de reproducción, selección, temas y operaciones de pintura múltiple existentes.

## Causas verificadas en el fork

radio.css aplica --btn-gradient a .grid-item.style-exact y
.fixed-panel-item.style-exact, pero no a .edit-style-preview.style-exact.
El selector de paleta asigna los valores sin emitir un cambio que llegue al
editor. Ambas rutas deben converger en el repintado del borrador; compartir la
representación evita diferencias entre vista previa y resultado guardado.

## Procedencia y verificación prevista

Base: Danilo Wenner, danilo-wenner-1/LF-Botonera-de-efectos, commit
7c1f1ae484cdb4bd0110b4761b8a221023bdea6f, GPL-3.0-or-later.
Atribución en historial, CHANGELOG, README y avisos, separando adaptación propia.

Pruebas de serialización antigua/nueva, validación de colores/direcciones,
edición en rejilla/panel fijo e importación/exportación. Prueba DOM de modos,
visibilidad, actualización inmediata y equivalencia entre vista previa y botón.
Traducciones en es/en/pt-BR/pt-PT. cargo test --lib, cargo build --lib y
npm run build locales. No abrir la app ni compilar en GitHub.

Estado: autorizado por el autor e implementado para su prueba funcional.
El autor podrá solicitar cambios adicionales al probar esta primera versión.

## Resultado y comprobaciones

Predefinidos equivale a estilo vacío; sólido personalizado conserva exact=true.
No hay una segunda casilla de color exacto: elegir personalizado ya expresa esa
intención. Los degradados usan horizontal/vertical/diagonal/diagonal_inv.
Campos HEX aceptan #RGB o #RRGGBB y guardan seis dígitos; los incompletos o
inválidos impiden aceptar. El borrador no dispara comandos de escritura de estilo.
Al pintar una selección con la paleta se borra su estilo avanzado para que la
pintura conserve el significado anterior; el resto de botones no cambia.
La edición OBS comparte el mismo formulario y también conserva style.

Archivos originales button_style.rs, buttonStyle.js y editStyle.js importados
en 75096df con autoría de Danilo; la adaptación se registra aparte. No se
incorpora el control de tamaño de fuente del fork.

445 pruebas Rust aprobadas, 22 omitidas. Se comprueba JSON antiguo, estilos
inválidos, configuración con rejilla/fijos/reproductor, ida y vuelta del formato
compartido y pintura múltiple. cargo build --lib y npm run build correctos.
Dos pruebas DOM aprobadas: claro/oscuro, controles condicionales, paleta,
cuatro direcciones, color exacto, texto, estado playing, HEX y equivalencia
con botones reales; Aceptar/Cancelar en rejilla/fijos mediante los editores
reales con IPC simulado. No se ejecutó la app ni se compiló en GitHub.

## Ajustes posteriores solicitados y autorizados

### Modal adaptable

El contenedor del editor tiene ancho limitado por la ventana y altura máxima del
área disponible. Cabecera y pie no se encogen ni salen de la ventana; solo el
cuerpo desplaza verticalmente. En ventanas estrechas las filas se apilan y las
acciones se distribuyen en varias líneas. La vista previa permanece en el cuerpo.

### Favoritos globales con nombre

El autor aprobó explícitamente que fueran globales y persistentes. Favoritos es
la segunda opción de Relleno. Permite elegir una combinación y eliminarla; el
borrador muestra el resultado al instante. Para editar sus colores después de
aplicarla, seleccionar sólido o la dirección de degradado correspondiente.

Debajo de la vista previa, «Guardar combinación en favoritos» despliega nombre
y acción de guardar. Este guardado es independiente de Aceptar/Cancelar del botón:
guardar un favorito es una acción explícita de persistencia. Cancelar el botón
no borra un favorito guardado. Eliminar un favorito pide confirmación y no
modifica botones ni el borrador donde ya se aplicó.

AppConfig.color_favorites es una lista opcional omitida cuando está vacía:
ColorFavorite {name, color_bg, color_text, style}. No contiene ruta, nombre de
sonido ni identificador visual. Se incluye en el respaldo completo normal y no
en exportaciones de un perfil/pestaña. Una configuración antigua empieza vacía.
Rust valida los colores y nombres de 1–80 caracteres, recorta espacios y rechaza
nombres repetidos sin distinguir mayúsculas. No sobrescribe favoritos existentes.

IPC: get_color_favorites(), save_color_favorite(favorite),
remove_color_favorite(name). Los dos últimos guardan la configuración antes de
publicar el cambio en RAM. colorFavorites.js solo presenta la lista y envía
acciones; editStyle.js comparte el borrador y la vista previa.

### Conservar colores al cambiar el sonido

En el editor, elegir archivo usa pick_audio_file y actualiza únicamente el
borrador; no llama a assign_file_to_button ni assign_file_to_fixed_button.
Aceptar valida y guarda la nueva ruta con los colores del borrador; Cancelar
conserva el sonido original. El tipo puede cambiar a carpeta aleatoria, hora,
temperatura o humedad sin restaurar ni sortear colores.

Las asignaciones directas de archivo/carpeta fuera del editor también conservan
color_bg, color_text y style al reemplazar una celda ocupada. La construcción
new_button_preserving_colors centraliza la regla: sortear únicamente sin botón
anterior. from_audio_file_preserving decodifica la nueva pista con esos colores.
La duración y la ruta se actualizan; los estilos no se transfieren desde otro
sonido. La copia explícita de un botón del reproductor mantiene su semántica
anterior de copiar el botón y sus colores.

Estas mejoras son adaptaciones del proyecto oficial; la base de colores y
su atribución a Danilo se conserva separada en el historial.

Verificación de esta ampliación: 453 pruebas Rust aprobadas, 22 omitidas;
cargo build --lib y npm run build correctos. Diez pruebas DOM del editor
aprobadas (cinco tamaños en claro/oscuro) y dos de regresión del buscador.
Los controles siguen accesibles mediante scroll y el pie/cabecera permanecen
dentro del viewport. Favoritos se guardan/reabren/aplican/eliminan con IPC
simulado; Rust comprueba validación y persistencia. WAV real verifica cambio
de medio/duración conservando los colores. Sin abrir la aplicación.
