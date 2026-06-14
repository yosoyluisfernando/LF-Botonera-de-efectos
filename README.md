# LF Botonera de Efectos

Botonera de efectos de sonido para locutores y emisoras de radio, construida
con **Tauri 2 + Rust** (motor de audio nativo) y una interfaz ligera en
HTML/CSS/JavaScript puro. Es la compañera del
[LF Automatizador](https://github.com/yosoyluisfernando/lf-automatizador): comparte sus formatos de archivo y
está preparada para enlazarse con él por red.

Creada por **Luis Fernando Velásquez**.

![Windows](https://img.shields.io/badge/Windows-10%2F11-0078D4?logo=windows)
![Linux](https://img.shields.io/badge/Linux-DEB%20%7C%20AppImage-FCC624?logo=linux&logoColor=black)
![Licencia](https://img.shields.io/badge/Licencia-GPL--3.0-green)
[![Donar](https://img.shields.io/badge/Donar-PayPal-00457C?logo=paypal)](https://www.paypal.com/donate/?hosted_button_id=3JJVFFBVR4MQQ)

> Si este proyecto te es útil, puedes apoyar su desarrollo:
> **[☕ Donar vía PayPal](https://www.paypal.com/donate/?hosted_button_id=3JJVFFBVR4MQQ)**

## Características

- **Cuadrícula de botones** configurable (filas × columnas) por pestaña,
  con perfiles ilimitados y pestañas ilimitadas por perfil.
- **Motor de audio en Rust** (rodio/CPAL): bucle, superposición,
  reinicio al pulsar, detener otros, y reproducción simultánea sin cortes.
- **Modos de reproducción global**: Normal, Loop, Stack (superposición),
  Restart, Solo y Stop All — seleccionables desde la barra inferior.
- **Cuenta regresiva y barra de progreso** en cada botón; pestaña en verde
  cuando suena audio en segundo plano.
- **Barra de estado inferior** con:
  - Reloj/contador en vivo (clic derecho para alternar formato 12/24 h).
  - Vúmetro estéreo L/R en tiempo real con detección de pico y decaimiento
    natural (verde → naranja → rojo).
  - Botones de modo de reproducción global con etiqueta e icono.
- **Atajos de teclado** por botón y por pestaña, atajos globales
  (detener todo, pestaña siguiente/anterior) y Modo Mapeo visual. ESC cierra
  cualquier modal; ENTER guarda el modal activo.
- **Detección de atajos huérfanos**: el panel de Atajos avisa cuando un atajo
  está asignado a un botón sin archivo de audio y permite limpiarlo.
- **Pre-escucha** con panel flotante, volumen en vivo y progreso.
- **Locuciones dinámicas** (módulo opcional): anuncios hablados de hora,
  temperatura y humedad con archivos `HRS`/`MIN`/`TMP`/`HUM`, clima vía
  Open-Meteo con refresco automático cada 15 minutos.
- **Exportación e importación** de pestañas (`.bdelf`) y perfiles
  (`.bdeplf`), 100% compatibles con el LF Automatizador v1.0.
- **Tema claro, oscuro y automático** (sigue el modo del sistema operativo),
  español e inglés, arrastrar y soltar desde el explorador, y reordenamiento
  de botones con Alt + arrastre.
- **Compatible con Windows y Linux** (EXE/MSI y DEB/AppImage).

## Formatos de audio compatibles

MP3 · WAV · FLAC · OGG/Vorbis · OGG/Opus · AAC · M4A · AIFF

Todos decodificados por [Symphonia](https://github.com/pdeljanov/Symphonia) vía rodio.
Ninguno requiere licencias de pago para uso en software libre GPL-3.0.

## Compatibilidad

| Sistema | Versión mínima | Instaladores disponibles |
|---|---|---|
| Windows | Windows 10 (20H2 o posterior) | `.exe` instalador · `.exe` portable · `.msi` |
| Linux | Ubuntu 22.04 / Debian 12 o equivalente con glibc 2.31+ | `.deb` · `.AppImage` |

> Windows 7, 8 y 8.1 no son compatibles: Tauri requiere WebView2 (Edge nativo),
> cuyo soporte en esas versiones finalizó en enero de 2023.

## Compilar desde el código fuente

Requisitos: [Node.js](https://nodejs.org/) y
[Rust](https://www.rust-lang.org/tools/install) (con los requisitos de
[Tauri 2](https://tauri.app/start/prerequisites/) para tu sistema).

**Windows:**
```
npm install
npm run tauri build
```

**Linux (Ubuntu/Debian):** instalar primero las dependencias del sistema:
```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev libasound2-dev
npm install
npm run tauri build
```

El ejecutable y los instaladores quedan en `src-tauri/target/release/bundle/`.  
Para desarrollo con recarga en vivo: `npm run tauri dev`.

### Compilación automatizada (GitHub Actions)

Al hacer push de un tag `v*` (p. ej. `v1.0.1`), el workflow
`.github/workflows/build.yml` compila automáticamente en Windows y Linux y
sube los artefactos (.msi, .exe, .deb, .AppImage).

### Actualizar la versión

La versión se declara en **tres lugares que deben coincidir**:

| Archivo | Campo |
|---|---|
| `src-tauri/tauri.conf.json` | `"version"` |
| `src-tauri/Cargo.toml` | `version` en `[package]` |
| `package.json` | `"version"` |

## Estructura del proyecto

- `src/` — interfaz (HTML/CSS/JS puro). La interfaz **no contiene lógica de
  negocio**: solo dibuja y delega al motor (ver las reglas del proyecto).
- `src-tauri/src/` — motor en Rust: audio, persistencia, exportación,
  locuciones, clima y comandos IPC.
- `Documentación/` — reglas inmutables del proyecto, plan maestro y planes
  de fase. **Léelas antes de contribuir**: el límite de 200 líneas por
  archivo y la regla "la interfaz es un control remoto tonto" no se negocian.
- `Respaldo_Electron/` — maqueta original en Electron, referencia visual y
  funcional de la experiencia de usuario.

## Licencia

Copyright (C) 2026 Luis Fernando Velásquez.

Este programa es **software libre**: puedes redistribuirlo y/o modificarlo
bajo los términos de la **Licencia Pública General de GNU (GPL) versión 3**,
publicada por la Free Software Foundation, o (a tu elección) cualquier
versión posterior.

Esto significa que **cualquier software derivado de este proyecto debe ser
también software libre** bajo la misma licencia (copyleft).

Este programa se distribuye con la esperanza de que sea útil, pero **SIN
GARANTÍA ALGUNA**; ni siquiera la garantía implícita de comerciabilidad o
idoneidad para un propósito particular. Consulta el archivo
[LICENSE](LICENSE) para el texto completo.
