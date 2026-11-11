# Apariencia de Windows

## Alcance aprobado — 2026-11-11

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
