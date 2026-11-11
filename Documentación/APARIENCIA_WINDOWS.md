# Apariencia de escritorio

## Alcance vigente beta.4 — 2026-11-11

El autor amplió el alcance a Windows y Linux. Ambos incluyen los mismos ajustes
visuales de Moderno y el selector de Principal con guardado inmediato. Android
conserva su interfaz específica. Las restricciones anteriores a Windows que se
narran a continuación son históricas y quedan sustituidas por esta decisión.

Vite habilita __LF_DESKTOP_THEMES__ y la hoja de apariencia en Windows y Linux.
Rust permite los dos sistemas para getter/setter y persistencia de desktop_style.
Las compilaciones móviles siguen sin esta capa. La clave de caché y los valores
default/modern se conservan; no se introduce un formato nuevo.

Los textos de peligro de los menús contextuales usan el color semántico de error,
con una variante legible en claro, en lugar del texto blanco para botones rojos.
Esto cubre pestañas, perfiles y los demás menús con la clase danger.

Se retiraron las transformaciones de hover y pulsación en rejilla/panel fijo.
El borde permanece bajo el puntero y se conservan sombras/bordes como feedback.
La prueba con entrada real de mouse comprueba el borde inferior en claro/oscuro,
con cursor quieto y con botón pulsado. Se verificó que detecta el efecto anterior.

## Alcance inicial — 2026-11-11

La rama `codex/desarrollo` incorpora exclusivamente Predeterminado y Moderno
del experimento local de `C:/OVERLAY/BOTONERA`. El experimento original se
conserva intacto. Linux, macOS, Android e iOS mantienen su presentación existente.

El selector es una lista en **Ajustes → Principal → Apariencia**. Cambia y
guarda al seleccionar, sin reinicio ni botón adicional. El modo de color
(claro, oscuro o sistema) es una preferencia independiente. Principal agrupa
Apariencia, botones de la barra y salidas de audio, conservando los controles
originales, sus identificadores y sus eventos. No existe una pestaña de temas.

## Persistencia y sincronización

- `model/appearance.rs`: `DesktopStyle` admite Default y Modern. Las preferencias
  antiguas sin campo siguen usando Default. Los valores históricos studio y
  essential se leen como Default para conservar el resto de la configuración;
  no se ofrecen ni aceptan al cambiar el diseño.
- `AppConfig.desktop_style` tiene default; se omite al serializar Default y se
  ignora completamente fuera de Windows.
- `get_desktop_appearance` devuelve disponibilidad y diseño guardado.
- `set_desktop_appearance(style)` valida en Rust, persiste y revierte el estado
  en memoria ante un fallo de escritura. Después emite
  `desktop-appearance-changed` para todas las ventanas.
- `desktopAppearance.js` aplica el atributo raíz y la caché de primer pintado.
  Al volver a Predeterminado retira ambos. La caché no sustituye la preferencia
  de Rust. El selector restaura el valor anterior si falla el guardado.
- Se inicializa en la ventana principal, bienvenida, editor independiente,
  consola y Biblioteca. El evento DOM `lf-theme-change` con el modo resuelto
  redibuja la onda inmediatamente después de cambiar el diseño.

## Capa visual

`desktopThemes.css` importa las cinco hojas Moderno y `settingsMain.css`.
Los estilos optativos están acotados al atributo Moderno. La organización de
Principal se aplica a ambos diseños en Windows. Vite retira el enlace CSS y
desactiva la lógica de caché/selector en las compilaciones de otras plataformas.
Rust rechaza además cualquier intento de activar el diseño fuera de Windows.

Se conservan colores personales, estados de reproducción y señalización OBS.
No se cambian las alturas de filas de listas virtualizadas. El contador del
editor usa `--action-accent` con texto blanco; la onda conserva su acento claro.
Los contenedores generales de modal no ocultan los menús desbordados.

## Verificación

- `node --test scripts/*test.mjs`: gestos táctiles, traducciones y cambio de
  diseño, fallo de guardado, sincronización y exclusión de plataformas.
- `cargo test --lib` y `cargo build --lib` desde `src-tauri`.
- `npm run build` para Windows, Linux, Android e iOS; inspección del resultado
  para confirmar que CSS y caché de apariencia solo figuran en Windows.
- La comprobación visual y funcional en la aplicación instalada corresponde
  al autor. No se lanza la aplicación durante esta verificación.

## Corrección del montaje — 2026-11-11

La capacidad real de escritorio es `platform: "desktop"`, tanto en Windows
como en Linux. La primera integración la comparaba con "windows" e impedía
montar el selector, incluso en el instalador correcto. La inicialización usa
la capacidad desktop; Vite y Rust mantienen la exclusión por sistema operativo.
Principal usa también el atributo desktop en sus reglas CSS.

Se añadió una prueba de navegador sin interfaz con el HTML y módulos reales:
comprueba montaje de los dos diseños, conservación de controles, navegación,
cambio inmediato, retorno a Predeterminado y contraste en claro/oscuro.
La prueba aislada usa el contrato real de Rust, sin inventar la plataforma.
La compilación corregida es 1.6.0-beta.2; Windows Installer usa 1.6.0.2.
No se publica un release ni se modifica main.

## Ajustes visuales beta.3 — 2026-11-11

La copia actual para continuar el desarrollo está en
`D:/BOTONERA/Desarrollo-Unificado` (rama codex/desarrollo). Las copias anteriores
de la raíz y app/ conservan sus cambios locales sin sobrescribirlos.

Moderno usa el acento morado en el botón de perfil y la pestaña activa; las
pestañas inactivas usan el fondo de controles del tema. La capa CSS prevalece
sobre los colores inline de esos controles, sin alterar preferencias guardadas.
Predeterminado recupera esos colores y tab-playing conserva su señal verde.

Los nombres de perfiles usan texto del tema en claro y oscuro. La selección
se distingue con fondo suave y borde de acento; la acción de borrar conserva
la señal de peligro. El dock inferior calcula su altura por contenido, con
mínimo de 102 px y 8 px entre el vúmetro y la fila de controles.

La prueba de DOM usa el HTML y módulos reales de perfiles, pestañas y colores:
comprueba acentos, contraste, geometría, señal de reproducción y restauración
de los colores personales. No se modifica lógica de audio ni IPC.
