# Plan: teclado visual para asignar atajos al cargar sonidos

## Objetivo

Agregar un flujo opcional para que, al cargar un sonido en un boton, la app muestre
un teclado en pantalla y permita asignar el atajo inmediatamente.

El usuario podria elegir la tecla de dos formas:

1. Haciendo clic en una tecla del teclado visual.
2. Presionando una tecla del teclado fisico.

La opcion debe poder activarse o desactivarse desde ajustes con un texto como:
"Añadir atajo del teclado al cargar un sonido".

## Contexto actual observado

- Los atajos locales se capturan en `src/js/ui/shortcuts.js`, pero Rust decide la
  accion con `handle_local_shortcut`.
- La normalizacion de teclas vive en `src/js/util/keyInputs.js`.
- La resolucion real de conflictos vive en Rust, en
  `src-tauri/src/engine/input/rules.rs`.
- El guardado de botones pasa por `update_button_data`, que ya acepta
  `replaceShortcut`.
- La carga directa de audio pasa por `assign_file_to_button`, que crea o reemplaza
  el boton, asigna color aleatorio y guarda inmediatamente.
- Los dos flujos principales de carga actuales son:
  - menu contextual / editor, mediante `editModal.js`;
  - arrastrar archivo a la grilla, mediante `gridDnd.js`.
- Tambien existe doble clic en celda vacia desde `grid.js`, que usa el mismo
  comando `assign_file_to_button`.

## Decision de arquitectura recomendada

Mantener el teclado visual como UI pura: dibuja teclas, muestra colores y captura
clics o eventos fisicos. La validacion y persistencia deben seguir en Rust.

Para evitar que el teclado visual tenga que llamar a `update_button_data` pasando
otra vez nombre, color, ruta y volumen, conviene agregar un comando IPC nuevo:

```text
set_button_shortcut(index, shortcut, replace_shortcut?) -> GridState
```

Ese comando reutilizaria `shortcut_rules::apply_button_shortcut`, guardaria solo
`btn.shortcut`, sincronizaria atajos globales y devolveria la grilla actualizada.

Tambien conviene agregar un comando de solo lectura para pintar el teclado sin
duplicar reglas de negocio en JavaScript:

```text
get_shortcut_keyboard_state(target_index?) -> ShortcutKeyboardState
```

Ese estado podria incluir:

- teclas usadas por botones de la paleta activa, con `index`, `label`,
  `color_bg`, `color_text` y `shortcut`;
- teclas usadas por pestañas del perfil;
- teclas globales del perfil: detener todo, pestaña siguiente y pestaña anterior;
- teclas reservadas del sistema de la app: `Ctrl+Z`, `Ctrl+Alt+Z`;
- la paleta activa y el boton objetivo.

El frontend solo interpretaria ese estado para aplicar clases visuales y textos
traducidos.

## Modelo de datos propuesto

Agregar campos globales a `AppConfig`, siempre con `#[serde(default)]`:

```rust
pub shortcut_prompt_on_audio_load: bool,
pub keyboard_layout: String,
```

Valores por defecto:

- `shortcut_prompt_on_audio_load = false`, para no cambiar el flujo actual al
  actualizar.
- `keyboard_layout = "auto"`, para elegir un layout inicial segun idioma/app/OS
  cuando sea posible.

Estos campos no necesitan viajar en `.bdelf` ni `.bdeplf`, porque son preferencias
de la app, no datos de botones o perfiles compartidos con LF Automatizador.

## Sobre "el teclado que tiene cada usuario"

Hay una limitacion importante: WebView/JavaScript no puede conocer de forma fiable
el modelo fisico exacto del teclado del usuario ni todas sus leyendas por privacidad
y diferencias del sistema operativo.

Propuesta realista:

1. Para tecla fisica, usar `KeyboardEvent.key`, como ahora. Eso respeta la
   distribucion activa del usuario al presionar una tecla real.
2. Para teclado visual, ofrecer layouts seleccionables:
   - Auto
   - Español Latinoamerica
   - Español España
   - Ingles US
   - Portugues Brasil
   - Portugues Portugal
3. En modo `Auto`, elegir el layout por idioma de la app como primer paso.
4. Opcional avanzado posterior: en Windows, consultar el layout activo desde Rust
   y usarlo como sugerencia. Esto debe tratarse como mejora progresiva, no como
   dependencia central, para no romper Linux/macOS.

## Flujo de usuario recomendado

### Flujo A: arrastrar archivo a boton

1. Usuario arrastra un audio sobre una celda.
2. `gridDnd.js` llama a `assign_file_to_button`.
3. Rust crea el boton, asigna color, valida audio y guarda.
4. Si `shortcut_prompt_on_audio_load` esta activo:
   - la UI localiza el boton resultante por `index`;
   - abre el modal de teclado con ese boton como objetivo;
   - pinta cada tecla ocupada con el color del boton que ya usa ese atajo;
   - el usuario hace clic en una tecla o presiona una tecla fisica;
   - JS llama a `set_button_shortcut`.
5. Si Rust reporta conflicto reemplazable, se usa el mismo patron de
   `shortcutSave.js`: confirmar y reintentar con `replaceShortcut = true`.
6. Se refresca la grilla.

### Flujo B: doble clic en celda vacia

Mismo flujo que arrastrar, porque tambien termina en `assign_file_to_button`.

### Flujo C: editor de boton

1. Usuario abre el editor desde menu contextual.
2. Pulsa `...` para seleccionar archivo.
3. `editModal.js` llama a `assign_file_to_button`, como hoy.
4. Si la opcion esta activa, se abre el teclado sobre el editor.
5. Al asignar la tecla, se actualiza el campo `#edit-shortcut` y tambien se guarda
   el atajo en Rust.

Punto a cuidar: si el usuario edito nombre/color/volumen en el modal antes de abrir
el selector de archivo, no conviene que el teclado vuelva a guardar todos esos
campos. Por eso se recomienda `set_button_shortcut`, que toca solo el atajo.

## Comportamiento visual del teclado

Cada tecla puede tener estados:

- Libre: estilo neutro.
- Ocupada por boton de la paleta activa: fondo con `color_bg` del boton y texto
  adaptado con `color_text`.
- Ocupada por pestaña: estilo de pestaña o borde especial.
- Reservada por atajo global del perfil: estilo bloqueado.
- Reservada por la app: estilo bloqueado.
- Tecla actualmente elegida: resaltado adicional.

Para tu idea de "si un atajo ya existe, toma el color que ya tiene el boton", la
regla mas clara seria:

- Si la tecla pertenece a un boton de la paleta activa, usar exactamente el color
  de ese boton.
- Si pertenece a una pestaña o accion global, no usar color de boton; mostrarla
  como reservada con tooltip o texto breve.

## Archivos nuevos sugeridos

Manteniendo el limite de 200 lineas:

- `src/js/ui/shortcutKeyboard.js`
  - abre/cierra el modal;
  - recibe objetivo;
  - captura clics y teclas fisicas;
  - llama a Rust para guardar.
- `src/js/ui/shortcutKeyboardLayout.js`
  - define layouts visuales y normaliza la tecla clicada al formato persistido.
- `src/css/shortcutKeyboard.css`
  - estilos del teclado visual.
- `src-tauri/src/ipc/cmd_shortcut_keyboard.rs`
  - `get_shortcut_keyboard_state`;
  - `set_button_shortcut`;
  - setters de preferencias si no quedan en `cmd_config.rs`.

## i18n necesaria

Agregar claves en `es.json`, `en.json`, `pt-BR.json` y `pt-PT.json` para:

- opcion de ajuste: "Añadir atajo del teclado al cargar un sonido";
- titulo del modal de teclado;
- botones: guardar, saltar, limpiar, cancelar;
- leyendas/ayudas: tecla libre, tecla ocupada, reservada, reemplazar atajo;
- selector de layout de teclado.

## Dudas que conviene cerrar antes de implementar

1. Al detectar una tecla ocupada por otro boton de la misma pestaña, ¿quieres que
   un clic la reemplace con confirmacion, como ahora, o que se bloquee y obligue a
   elegir una tecla libre?
2. Si el mismo atajo existe en otra pestaña, ¿debe mostrarse como ocupado? Hoy Rust
   permite repetir atajos de boton entre pestañas porque solo dispara en la paleta
   activa.
3. ¿La preferencia "Añadir atajo al cargar un sonido" debe ser global para toda la
   app o por perfil?
4. ¿El teclado visual debe mostrar solo teclas simples al inicio, o tambien combos
   con `Ctrl`, `Alt` y `Shift` mediante modificadores activables?

## Fases de implementacion

### Fase 1: base segura

- Agregar preferencia `shortcut_prompt_on_audio_load`.
- Agregar comando `set_button_shortcut`.
- Reutilizar `shortcutSave.js` para conflictos.
- Abrir el teclado despues de `assign_file_to_button` en drag/drop, doble clic y
  selector del editor.

### Fase 2: teclado visual completo

- Crear modal y CSS del teclado.
- Pintar ocupacion de teclas con estado entregado por Rust.
- Soportar clic en tecla y tecla fisica.
- Agregar selector de layout.

### Fase 3: mejoras de layout

- Modo `Auto` por idioma.
- Deteccion opcional desde Rust en Windows si vale la pena.
- Ajustes finos para teclas especiales, teclado numerico y layouts ISO/ANSI.

## Verificacion

- `cd src-tauri && cargo test --lib`
- `npm run build`
- Contar lineas de archivos nuevos o modificados con `wc -l`.

La prueba visual final la hara el usuario en su equipo, como indica la regla del
proyecto.
