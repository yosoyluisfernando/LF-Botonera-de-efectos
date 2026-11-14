# Editor unificado de colores — integración autorizada

## Solicitud del autor — 2026-11-14

Una sola lista de relleno en el editor del botón, con Predefinidos primero y
seleccionado para botones nuevos o antiguos sin estilo avanzado. Después:
Color sólido, Degradado horizontal, Degradado vertical, Degradado diagonal ↘
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
el estilo como argumento opcional; no hacen falta nuevos comandos ni eventos IPC.
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
