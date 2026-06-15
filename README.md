<div align="center">
  <img src="icono_circular.png" alt="Logo de LF Botonera de Efectos" width="96" height="96">

  <h1>LF Botonera de Efectos</h1>

  <p><strong>Botonera profesional de efectos de sonido para radio, locución y streaming.</strong></p>

  <p>Reproduce efectos, cortinas, identificaciones, locuciones y audios de apoyo desde una interfaz rápida, clara y controlable por teclado.</p>

  [![Windows](https://img.shields.io/badge/Windows-10%2F11-0078D4?logo=windows)](#descarga)
  [![Linux](https://img.shields.io/badge/Linux-DEB%20%7C%20AppImage-FCC624?logo=linux&logoColor=black)](#descarga)
  [![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
  [![Rust](https://img.shields.io/badge/Rust-audio%20engine-B7410E?logo=rust)](https://www.rust-lang.org/)
  [![Licencia](https://img.shields.io/badge/Licencia-GPL--3.0--or--later-green)](LICENSE)
  [![Release](https://img.shields.io/github/v/release/yosoyluisfernando/LF-Botonera-de-efectos?label=version)](https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/releases/latest)
  [![Donar](https://img.shields.io/badge/Donar-PayPal-00457C?logo=paypal)](https://www.paypal.com/donate/?hosted_button_id=3JJVFFBVR4MQQ)
</div>

---

## ¿Qué es?

**LF Botonera de Efectos** es una aplicación de escritorio para disparar sonidos en vivo con precisión y velocidad. Está pensada para locutores, emisoras de radio, operadores, productores, creadores de contenido y streamers que necesitan una botonera confiable durante una transmisión real.

El proyecto está construido con **Tauri 2 + Rust**: la interfaz es ligera y el motor de audio es nativo. Eso permite menor consumo de recursos, mejor respuesta y una base más sólida que una aplicación web empaquetada tradicional.

También es parte del ecosistema de [LF Automatizador](https://github.com/yosoyluisfernando/lf-automatizador). Ambos proyectos comparten formatos de archivo para que pestañas y perfiles puedan convivir sin romper flujos de trabajo existentes.

## Descarga

La última versión estable está disponible en la página de releases:

**[Descargar LF Botonera de Efectos](https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/releases/latest)**

| Sistema | Archivo recomendado | Uso |
|---|---|---|
| Windows 10/11 | `LF.Botonera.de.Efectos_*_x64-setup.exe` | Instalador con asistente para la mayoría de usuarios |
| Windows 10/11 | `LF.Botonera.de.Efectos_*_x64_en-US.msi` | Instalador MSI para entornos más controlados |
| Linux Ubuntu/Debian | `LF.Botonera.de.Efectos_*_amd64.deb` | Paquete para distribuciones basadas en Debian |
| Linux universal | `LF.Botonera.de.Efectos_*_amd64.AppImage` | Ejecutable portable sin instalación |

> En Windows se requiere Windows 10 20H2 o posterior con WebView2 Runtime. En Windows 11 y Windows 10 actualizado normalmente ya viene incluido.

## Para qué sirve

- Disparar efectos de sonido, separadores, intros, pisadores, cortinas y locuciones.
- Organizar audios por perfiles, pestañas y cuadrículas personalizadas.
- Controlar botones desde teclado sin depender del ratón.
- Reproducir varios sonidos a la vez o aplicar reglas globales de reproducción.
- Preparar una botonera compatible con flujos de radio, cabina, streaming y producción.

## Características principales

- **Motor de audio en Rust** con reproducción simultánea y respuesta rápida.
- **Cuadrícula configurable** por pestaña: filas, columnas, colores, nombres y salida de audio.
- **Perfiles y pestañas ilimitadas** para separar programas, bloques, clientes o estilos de trabajo.
- **Modos de reproducción global**: Normal, Loop, MULTI, Restart, Solo y Stop All.
- **Atajos de teclado** por botón, por pestaña y globales, con modo de mapeo visual.
- **Pre-escucha** con panel flotante, volumen independiente y progreso.
- **Vúmetro estéreo L/R** en tiempo real.
- **Reloj, fecha y contador regresivo** en la barra inferior.
- **Tema claro, oscuro y automático**, con adaptación de colores para mantener contraste.
- **Arrastrar y soltar** archivos desde el explorador.
- **Reordenamiento de botones** con Alt + arrastre.
- **Exportación e importación** de pestañas (`.bdelf`) y perfiles (`.bdeplf`).
- **Búsqueda de actualizaciones** desde GitHub Releases.
- **Español e inglés** mediante archivos de traducción.

## Formatos de audio compatibles

MP3 · WAV · FLAC · OGG/Vorbis · OGG/Opus · AAC · M4A · AIFF

La decodificación se apoya en el ecosistema de audio usado por Rust y Tauri, sin depender de componentes propietarios de pago dentro del proyecto.

## Compatibilidad con LF Automatizador

LF Botonera de Efectos conserva compatibilidad con los formatos del LF Automatizador:

| Formato | Uso |
|---|---|
| `.bdelf` | Pestañas de botones |
| `.bdeplf` | Perfiles completos |

La idea es que una emisora o creador pueda usar la botonera para efectos rápidos y el automatizador para programación más amplia, manteniendo una estructura compatible entre ambas herramientas.

## Filosofía del proyecto

Este proyecto sigue reglas estrictas de arquitectura:

- La interfaz no procesa audio ni inventa estado crítico.
- La UI funciona como un control remoto: dibuja, envía acciones y espera respuestas.
- La lógica fuerte vive en Rust.
- Cada archivo debe mantenerse pequeño y con responsabilidad clara.
- No se aceptan soluciones improvisadas ni parches acumulados.
- El diseño debe funcionar correctamente en modo claro y oscuro.
- Todo texto visible debe pasar por i18n.

Las reglas completas están en [Documentación/REGLAS_PROYECTO.md](Documentación/REGLAS_PROYECTO.md).

## Compilar desde el código fuente

Requisitos:

- [Node.js](https://nodejs.org/)
- [Rust](https://www.rust-lang.org/tools/install)
- Dependencias del sistema requeridas por [Tauri 2](https://tauri.app/start/prerequisites/)

### Windows

```powershell
npm install
npm run tauri build
```

### Linux Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libasound2-dev \
  patchelf \
  squashfs-tools

npm install
npm run tauri build
```

Los instaladores quedan en:

```text
src-tauri/target/release/bundle/
```

Para desarrollo:

```bash
npm run tauri dev
```

## Publicación automática

El repositorio incluye un workflow de GitHub Actions. Al publicar un tag con formato `v*`, por ejemplo `v1.0.1`, se compila automáticamente para Windows y Linux y se publica un release con los instaladores generados.

La versión debe mantenerse sincronizada en:

| Archivo | Campo |
|---|---|
| `package.json` | `"version"` |
| `package-lock.json` | `"version"` |
| `src-tauri/Cargo.toml` | `version` |
| `src-tauri/Cargo.lock` | paquete local `tauri-app` |
| `src-tauri/tauri.conf.json` | `"version"` |

## Estructura del repositorio

| Ruta | Descripción |
|---|---|
| `src/` | Interfaz HTML, CSS y JavaScript |
| `src-tauri/src/` | Motor Rust, audio, configuración, comandos IPC y empaquetado |
| `src/public/i18n/` | Traducciones de la interfaz |
| `Documentación/` | Reglas, planificación y documentación interna del proyecto |
| `Respaldo_Electron/` | Maqueta original usada como referencia visual y funcional |
| `.github/workflows/` | Compilación y publicación automática |

## Firma de Windows

Actualmente los instaladores de Windows no están firmados con certificado de desarrollador. Windows puede mostrar una advertencia al abrir el instalador por primera vez.

El código fuente está disponible públicamente y el proyecto se distribuye bajo licencia libre. A futuro se contempla preparar la aplicación para una distribución más formal, incluyendo Microsoft Store.

## Apoyar el desarrollo

Si esta herramienta te resulta útil en radio, producción o streaming, puedes apoyar su mantenimiento:

**[Donar vía PayPal](https://www.paypal.com/donate/?hosted_button_id=3JJVFFBVR4MQQ)**

## Licencia

Copyright (C) 2026 **Luis Fernando Velásquez**.

Este programa es **software libre**: puedes redistribuirlo y/o modificarlo bajo los términos de la **Licencia Pública General de GNU GPL-3.0-or-later**.

Este programa se distribuye con la esperanza de que sea útil, pero **sin garantía alguna**. Consulta el archivo [LICENSE](LICENSE) para el texto completo.
