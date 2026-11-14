# Buscador de botones y acceso rápido al catálogo

## Alcance autorizado — 2026-11-13

El autor pidió conservar el funcionamiento del buscador de Danilo Wenner para
el clic izquierdo y añadir, con clic derecho, un acceso rápido al Buscador del
panel fijo existente. No se sustituye ni duplica el catálogo de la Biblioteca.

## Clic izquierdo: buscador original de Danilo

La lupa se sitúa a la izquierda de la barra de pestañas. Abre el mismo panel
flotante del fork: consulta por texto, archivo, carpeta, pestaña y posición,
ignorando tildes y mayúsculas. Las palabras se combinan con Y; se conservan
el orden de relevancia, límite de 60 resultados y ubicación de cada botón.

La búsqueda recorre las pestañas del perfil activo en Rust sobre AppConfig.
No consulta tracks.db, no indexa botones en SQLite y no accede al disco.
Al elegir un resultado cambia de pestaña si hace falta, desplaza el botón a
la vista y lo resalta temporalmente; no lo dispara. Conserva selección con
ratón, flechas y Enter, y cierre mediante Escape o clic fuera del panel.

Procedencia: Danilo Wenner, LF Botonera Radio, commit
`7c1f1ae484cdb4bd0110b4761b8a221023bdea6f` del repositorio
https://github.com/danilo-wenner-1/LF-Botonera-de-efectos, GPL-3.0-or-later.
Se importan `buttonSearch.js`, `domain/grid/search.rs`, sus pruebas y el comando
`search_buttons`, más los estilos y traducciones propios del buscador.
Las pruebas se ajustan al modelo oficial (obs en lugar del campo font_size
exclusivo del fork); el algoritmo y la interacción del clic izquierdo se conservan.

## Clic derecho: atajo al Buscador del panel fijo

El clic derecho en la misma lupa cierra el buscador flotante, evita el menú
nativo y llama a `openFixedPanelSearch()` en `fixedPanelViews.js`.

- Si el panel está oculto, lo abre mostrando Buscador.
- Si muestra Reproductor o Botones fijos, cambia a Buscador y sigue abierto.
- Si ya muestra Buscador, permanece abierto; no funciona como alternador.
- El campo de búsqueda del catálogo recibe el foco.

Se reutilizan `get_fixed_panel` y `set_fixed_panel_settings`, con view=search
y visible=true. Se conservan scope, lado, columnas, filas, límite, ancho y
posición de los modos. El resultado usa el mismo drawFixedPanel que el selector
de vistas existente. La consulta/lista del catálogo conserva su funcionamiento.
La visibilidad y vista se recuerdan mediante la persistencia estándar del panel.

Este atajo al catálogo es de escritorio: Android conserva la disponibilidad
actual de su panel, donde la vista Buscador no está habilitada. La búsqueda de
botones puede abrirse con el toque/clic normal.

## Arquitectura y verificación

Toda la búsqueda y sus reglas viven en Rust. JS solo interpreta gestos,
representa resultados y llama al IPC de navegación/panel ya existente.
El nuevo IPC `search_buttons(query)` es de solo lectura del perfil activo.
No cambia modelo persistente, exportaciones, transporte, cola ni motores de audio.
`tabs.js` proporciona la configuración actual al buscador al refrescar perfiles.

Las pruebas Rust importadas verifican todas las pestañas, archivos, tildes,
posiciones y relevancia. La prueba DOM comprueba clic izquierdo, navegación,
resaltado y los cinco estados iniciales del panel para clic derecho a 768/1366 px,
conservación de opciones y ausencia de comandos de reproducción/parada.
Las verificaciones siguen siendo locales; no se abre la botonera ni se lanzan
compilaciones en GitHub. La prueba manual se hace con DEV.bat del worktree.
