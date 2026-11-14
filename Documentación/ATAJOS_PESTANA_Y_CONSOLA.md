# Atajos de pestaña y recuperación de la consola — 2026-11-13

## Alcance autorizado

El autor pidió recuperar el foco de la consola flotante al pulsar de nuevo su
botón, e importar el acceso de Danilo a la captura de atajos desde el menú de
pestaña. El teclado conserva la interacción existente; MIDI va plegado y depende
de su disponibilidad real. Trabajo en variante-danilo, sin abrir la aplicación,
compilar en GitHub, publicar una versión ni modificar main.

## Consola flotante

consoleWindow.js consulta WebviewWindow.getByLabel('audio-console') antes de
crear. Si existe, espera unminimize(), show() y setFocus() en ese orden. Si se
cerró, crea la ventana habitual. openPreferred y popOut esperan el resultado.
Se conserva el modo modal, el fallback y la exclusión de Android. El botón no
cierra ni alterna la ventana; el usuario la cierra con su X.
No se altera consola, audio, buses, ruteo, medidores ni preferencias guardadas.
Se reutilizan APIs y permisos ya presentes, como en trackEditorWindow.js.

## Acceso de Danilo y teclado

Procedencia: Danilo Wenner, danilo-wenner-1/LF-Botonera-de-efectos, GPL-3.0-or-later,
commit 7c1f1ae484cdb4bd0110b4761b8a221023bdea6f. Listener original de tabs.js,
entrada de menú y traducciones importados en 1d1f16c con su autoría; la ampliación
MIDI se registra aparte. Créditos en README, CHANGELOG y THIRD_PARTY_NOTICES.

Clic derecho → Asignar atajo de teclado… utiliza captureTab con la pestaña
pulsada y su perfil, incluso si no es la activa. No cambia la pestaña al abrir
el editor de atajos. La combinación se muestra con las reglas anteriores:
keyInputs.js captura/limpia; invokeShortcutSave confirma conflictos y Rust
valida/persiste mediante update_paleta_meta. Al usar después el atajo, el
mecanismo de teclado/MIDI ya existente activa esa pestaña.

## MIDI opcional

capture-midi-section es un details plegado inicialmente. Solo se muestra cuando
get_shortcut_editor_state informa midi_available: MIDI habilitado y al menos
una entrada conectada. No hay IPC, modelo ni campos nuevos; se reutilizan los
bindings de PaletaData y las reglas/captura MIDI existentes.

midi-devices-changed y el foco de ventana refrescan disponibilidad mientras el
modal está abierto. Desconectar oculta y pliega, deshabilita captura y cancela
la espera. Reconectar muestra la sección plegada. Cerrar el desplegable o el
modal cancela la captura pendiente. No se borra una asignación por estar
oculta o desconectada. El guardado se bloquea mientras hay captura pendiente.
El modal acota su altura a la ventana y permite desplazar el cuerpo cuando
se despliega MIDI en una pantalla baja.

## Verificación

Pruebas de ventana con API nativa simulada: recuperación repetida de foco,
restauración, crear tras cierre, modo modal, fallback y exclusión Android.
Prueba DOM con módulos reales y backend simulado: pestaña contextual no activa,
captura F8, confirmación de conflictos, preservación de MIDI oculto, capturar,
desconectar/reconectar, cancelar, guardar y ausencia de reproducción/navegación.
Se comprueba acceso al pie del modal en ventanas de 1366×900 y 640×360.
La prueba funcional de foco en Windows y del controlador MIDI real la hace el
autor con DEV.bat del worktree. No se ha abierto la aplicación desde herramientas.

Validación local: 453 pruebas Rust aprobadas, 22 omitidas; cargo build --lib
correcto. Cinco pruebas de consola y dos pruebas DOM de pestaña/MIDI aprobadas;
npm run build correcto. El foco nativo y el dispositivo MIDI real quedan para
la prueba funcional del autor. No se han ejecutado compilaciones en GitHub.
