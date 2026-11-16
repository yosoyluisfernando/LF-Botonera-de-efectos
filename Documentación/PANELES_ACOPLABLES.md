# Paneles acoplables en escritorio

El espacio de trabajo conserva la rejilla en el centro. Los paneles auxiliares
pueden ocupar un lateral o abrirse en una ventana independiente.

## Primera etapa · Consola de audio

Implementación de escritorio en `codex/desarrollo`. La prueba de ventana
nativa en Linux queda pendiente.
El clic derecho en el botón de la consola ofrece ambos laterales, ventana
independiente y modal. En modo modal también hay un selector en su cabecera.
El clic izquierdo muestra u oculta el lateral, o enfoca la ventana independiente
si ya está abierta.
El separador permite ajustar el ancho con ratón o teclado. Al reabrir la consola
desde su botón se recuperan la última ubicación elegida y el ancho guardado.
El ancho inicial es de 320 px. Las cinco tiras se ajustan al ancho disponible
en el lateral; el máster queda visible al reducir el panel.

La ventana principal reutiliza la misma vista dentro del lateral y del modal.
La ventana independiente sigue usando `WebviewWindow`. En ambos casos,
`get_console_view`, `meter-tick` y `set_bus_fader` siguen sirviendo la misma
consola de Rust. La preferencia existente `console_mode` admite también
`left` y `right`, y el ancho está en `console.dock_width` con un valor
predeterminado para configuraciones anteriores. `set_console_mode` acepta un
ancho opcional para persistir un redimensionamiento del lateral activo.

Para probar con `DEV.bat`, abrir la consola, alternar los cuatro modos con clic derecho,
redimensionar con la ventana principal maximizada y comprobar el Panel fijo en
ambos lados. Con dos monitores, mover la ventana independiente y volver a
acoplarla. Durante la prueba, reproducir efectos y música, comprobar medidores
y faders, y cerrar y reabrir para verificar la ubicación y el ancho.

## Segunda etapa · Panel fijo

Evaluar una ventana independiente para el Panel fijo completo, con sus vistas
de botones fijos, reproductor y buscador. La ventana debe conservar la vista
seleccionada y poder volver al lateral sin reiniciar la reproducción ni perder
la cola.

## Última etapa · Biblioteca

Preparar una vista que dé prioridad a búsqueda y lista de pistas en un lateral
estrecho. Revisar el acceso a acciones de pista, metadatos y procesamiento desde
la Biblioteca acoplada y desde su ventana independiente.

El arrastre de paneles entre zonas, las pestañas de paneles y el bloqueo de la
distribución se revisarán después de la primera etapa.
