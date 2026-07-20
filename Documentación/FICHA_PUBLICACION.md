# Ficha base de publicación

Borrador común para Microsoft Store, Flathub y otros catálogos. Se adapta a los
campos y límites vigentes de cada canal en el momento del envío; no se copiará a
ciegas si una tienda cambia sus requisitos.
**Producto:** LF Botonera de Efectos
**Versión base:** 1.2.1
**Licencia:** GPL-3.0-or-later
**Autor y publicador humano:** Luis Fernando Velásquez

## 1. Identidad propuesta

- Nombre público: `LF Botonera de Efectos`.
- Identificador técnico común:
  `io.github.yosoyluisfernando.LF-Botonera-de-efectos`.
- Nombre de paquete Microsoft: `LuisFernandoVelasquez.LFBotoneradeEfectos`.
- Publicador Microsoft: `CN=AD90DE58-447F-47AE-AC1A-3D369955282B`.
- Nombre visible del publicador Microsoft: `Luis Fernando Velasquez`.
- Familia del paquete: `LuisFernandoVelasquez.LFBotoneradeEfectos_5cjhmbb3mr2nr`.
- Id. de Microsoft Store: `9NJ8ST39QP7V`.
- Identificador Flatpak propuesto:
  `io.github.yosoyluisfernando.LF-Botonera-de-efectos`. Coincide con el propietario y
  el nombre reales del repositorio de GitHub, por lo que puede verificarse mediante la
  cuenta del autor. Se confirmará nuevamente durante la revisión de Flathub.

Referencias: [identificador de Tauri](https://v2.tauri.app/reference/config/#identifier),
[requisitos de ID de Flathub](https://docs.flathub.org/docs/for-app-authors/requirements#application-id)
y [verificación de autor en Flathub](https://docs.flathub.org/docs/for-app-authors/verification).

## 2. Texto en español

### Descripción breve

Botonera profesional de sonidos para radio, locución y streaming en directo.

### Descripción

LF Botonera de Efectos permite organizar y disparar efectos, cortinas,
identificaciones, locuciones y música durante una emisión o producción en directo.
Los sonidos se distribuyen en perfiles, pestañas y botones configurables para que cada
programa, cliente o proyecto conserve su propia organización.

Su motor de audio nativo ofrece mezcla simultánea, preescucha independiente, atajos de
teclado, precarga en memoria y control por pista. El editor integrado permite ajustar
puntos de inicio y fin, ganancia y normalización sin modificar el archivo original.

La aplicación incluye un panel lateral con botones fijos, un reproductor auxiliar de
música y una consola para controlar por separado efectos, panel, reproductor,
preescucha y salida principal. También puede construir locuciones de hora, temperatura
y humedad usando archivos de voz elegidos por el usuario.

LF Botonera de Efectos es software libre bajo GPL-3.0-or-later y es compatible con los
formatos de pestañas, perfiles y listas de LF Automatizador.

### Funciones destacadas

- Perfiles y pestañas ilimitados con rejillas configurables.
- Reproducción simultánea, bucle, reinicio, superposición y modo Solo.
- Preescucha mediante una salida de audio independiente.
- Atajos locales y globales del sistema operativo.
- Editor con forma de onda, cue, ganancia y normalización LUFS o pico.
- Precarga de efectos en RAM para reducir el retardo.
- Panel de botones fijos y reproductor auxiliar con listas `.LFPlay`.
- Consola de audio con fader y vúmetro por fuente.
- Locuciones configurables de hora y clima.
- Importación y exportación compatibles con LF Automatizador.
- Interfaz en español, inglés y portugués.

### Palabras clave propuestas

`botonera`, `soundboard`, `radio`, `streaming`, `efectos de sonido`, `locución`,
`audio`, `podcast`, `reproductor`, `emisión en directo`.

## 3. English copy

### Short description

Professional soundboard for live radio, voice-over, and streaming workflows.

### Description

LF Botonera de Efectos helps operators organize and trigger sound effects, music beds,
station IDs, voice clips, and music during live broadcasts and productions. Sounds are
arranged into configurable profiles, tabs, and button grids, allowing each show,
client, or project to keep its own layout.

Its native audio engine supports simultaneous playback, independent preview output,
keyboard shortcuts, memory preloading, and per-track control. The integrated track
editor adjusts start and end cue points, gain, and normalization without modifying the
original audio file.

The application includes a side panel for persistent buttons, an auxiliary music
player, and a console that controls effects, side-panel audio, music, preview, and the
main output separately. It can also assemble time, temperature, and humidity
announcements from voice files selected by the user.

LF Botonera de Efectos is free software licensed under GPL-3.0-or-later and supports
the tab, profile, and playlist formats used by LF Automatizador.

### Feature highlights

- Unlimited profiles and tabs with configurable button grids.
- Simultaneous playback, loop, restart, overlap, and Solo behavior.
- Independent audio output for preview listening.
- Local and operating-system-wide keyboard shortcuts.
- Waveform editor with cue points, gain, and LUFS or peak normalization.
- RAM preloading to reduce trigger latency.
- Persistent side-panel buttons and an auxiliary player with `.LFPlay` playlists.
- Audio console with a separate fader and meter for each source.
- Configurable time and weather announcements.
- Import and export compatibility with LF Automatizador.
- Spanish, English, Brazilian Portuguese, and European Portuguese interface.

### Proposed keywords

`soundboard`, `radio`, `streaming`, `sound effects`, `voice-over`, `audio`, `podcast`,
`music player`, `live production`, `broadcast`.

## 4. Información común de la ficha

- Sitio web y código fuente:
  `https://github.com/yosoyluisfernando/LF-Botonera-de-efectos`
- Releases:
  `https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/releases`
- Soporte:
  `https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/blob/main/SUPPORT.md`
- Privacidad:
  `https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/blob/main/PRIVACY.md`
- Licencia:
  `https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/blob/main/LICENSE`

Estas URLs serán válidas públicamente cuando los documentos estén integrados en la
rama principal. Antes del envío se abrirán desde una sesión sin autenticar.

## 5. Requisitos y declaraciones

- Windows admitido: Windows 10 y Windows 11.
- Arquitectura Windows auditada: x64.
- Linux: soporte experimental hasta completar la prueba física.
- Acceso a Internet: necesario para clima y comprobación de actualizaciones; el resto
  de las funciones principales trabaja con archivos locales.
- Cuenta de usuario: no requerida.
- Publicidad: ninguna.
- Compras dentro de la aplicación: ninguna.
- Donaciones: enlace externo opcional a PayPal.
- Telemetría: ninguna encontrada en la auditoría de la versión 1.2.1.
- Contenido generado por usuarios: no se publica ni comparte desde la aplicación.
- Archivos de audio: deben ser aportados y licenciados por el usuario.

## 6. Recursos gráficos

La biblioteca actual contiene 17 PNG maestros de Windows en español, todos de
`1440×860`. Su inventario, aptitud y restricciones están documentados en
[`../Capturas/README.md`](../Capturas/README.md). La selección pública recomendada es:

1. Rejilla principal con varias pestañas y botones representativos.
2. Panel lateral fijo y reproductor auxiliar.
3. Editor de pistas con forma de onda.
4. Consola de audio y vúmetros.
5. Ajustes de dispositivos y preescucha.
6. Tema claro o idiomas disponibles, si la tienda admite una sexta captura.

Antes de ampliar una ficha se repetirán el reproductor en reposo, Acerca de con la
versión vigente y Hora y Clima sin datos locales. Para Linux se hará una serie en el
sistema real; no se reutilizará una captura de Windows como evidencia de Linux.

## 7. Decisiones pendientes del autor

- Gratuita o de pago en Microsoft Store. Recomendación inicial: gratuita, coherente
  con el canal actual y la GPL, conservando la donación opcional.
- Países o regiones de distribución.
- Categoría y subcategoría disponibles en Partner Center.
- Visibilidad pública inicial o publicación limitada para pruebas.
- Idiomas que tendrán ficha completa desde el primer envío.
- Datos públicos de contacto que exige la cuenta individual.

No se responderá el cuestionario de clasificación por edades hasta verlo en Partner
Center. Las respuestas se basarán en funciones reales, no en una clasificación
estimada.

## 8. Afirmaciones que no deben hacerse todavía

- No anunciar certificación Microsoft hasta aprobarla.
- No afirmar compatibilidad total con Linux, Flatpak o Wayland antes de la prueba
  física.
- No afirmar accesibilidad completa con lector de pantalla sin una prueba específica.
- No llamar oficiales a AUR, PPA, COPR o un repositorio personal de OBS.
- No prometer instalación sin conexión mientras WebView2 no esté resuelto.
- No decir que un paquete está firmado si la firma no se verifica sobre el artefacto
  final.
