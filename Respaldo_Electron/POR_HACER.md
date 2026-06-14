# Plan de Trabajo - LF Botonera de efectos (Cross-Platform & Rust Ready)

## Estado del Proyecto
Actualmente, la botonera está construida en **Electron + Vanilla JS** (`main.js` y `renderer.js`). Utiliza el motor web (Web Audio API a través de la etiqueta `<audio>` de HTML5/JS) para reproducir los audios. La gestión de rutas asume en varios fragmentos que se está ejecutando sobre **Windows**, utilizando barras invertidas (`\`). 

**Objetivo Final:** 
1. Hacer el proyecto agnóstico al sistema operativo (Windows y Linux detectados dinámicamente).
2. Limpiar el código para convertirlo en un "Thin Client" (cliente ligero) listo para enviar comandos IPC a un futuro backend en Rust.

---

## 🟢 Fase 1: Preparación del Repositorio (Completado)
- [x] Auditoría completa del código (`main.js`, `renderer.js`, `package.json`).
- [x] Crear archivo `.gitignore` para omitir node_modules, configuraciones locales (.bdelf, .bdeplf) y archivos del sistema.
- [x] Crear archivo `README.md` documentando la versión v1.0.0 y las características.
- [x] Inicializar este archivo `POR_HACER.md` como hoja de ruta.
- [ ] **Definir Licencia:** Seleccionar y agregar el archivo de licencia oficial para el proyecto.

## 🟢 Fase 2: Compatibilidad Multiplataforma (Completado)
- [x] **Manejo Dinámico de Rutas:** Modificar la lógica de arrastrar/soltar en `renderer.js` para usar `path.basename` y evitar fallos por barras invertidas o inclinadas (`/` vs `\`).
- [x] **Formato de Rutas en Audio:** Reemplazar los reemplazos forzados por Regex (`.replace(/\\/g, '/')`) con un gestor inteligente o utilizar `file://${ruta_correcta}` de forma estandarizada.
- [x] **Detector Inteligente del SO:** Implementar lógica utilizando `process.platform` para asegurar que el sistema sabe si corre en Linux ('linux') o Windows ('win32').
- [x] **Limpieza de Archivos Nativos:** Analizar alternativas para la ejecución si existen scripts (`Iniciar_Botonera.vbs`) y crear equivalentes bash (`.sh`) para Linux.

## 🟢 Fase 3: Refactorización hacia Arquitectura "Thin Client" (Completado)
- [x] **Separación del Lógico de Audio:** Mover la creación dinámica de `new Audio(...)` en `renderer.js` a un gestor de audio encapsulado (preparando el terreno).
- [x] **Limpieza de UI vs Lógica:** Separar en módulos más pequeños si es posible, desvinculando la renderización visual de la ejecución del audio.
- [x] **Optimización de Memoria (Clones/Overlap):** Asegurar que las instancias viejas se destruyan correctamente (Garbage Collection) para evitar fugas de memoria en la interfaz web.

## 🔴 Fase 4: Integración del Motor Rust y Arquitectura "Thin Client" Definitiva (En Espera)
*Nota: Esta fase transformará a Electron en un simple "control remoto" (UI), delegando todo el trabajo pesado de audio, memoria y hardware al backend en Rust.*

- [ ] **Desactivar el Audio HTML5:** Eliminar la dependencia de la API de `<audio>` web en `audioEngine.js` para que deje de decodificar archivos MP3/WAV en el hilo principal de Node/Chromium.
- [ ] **Migrar la Detección de Dispositivos (Hardware):** Eliminar `navigator.mediaDevices` en JS. Será Rust quien escanee las tarjetas de sonido (usando librerías como CPAL) y le envíe la lista de dispositivos disponibles a la interfaz.
- [ ] **Gestión de Memoria y Caché en Rust:** El motor Rust se encargará de leer los archivos del disco duro, decodificarlos y cargarlos en RAM. La UI solo enviará la ruta del archivo (`file_path`).
- [ ] **Puente de Comunicación Bidireccional (IPC/Sockets):**
  - **Comandos (UI -> Rust):** Enviar peticiones como `PLAY(id)`, `STOP(id)`, `SET_VOLUME(id, vol)`, `SET_OUTPUT(device_id)`.
  - **Telemetría (Rust -> UI):** Rust debe emitir eventos constantes (ej. 60 FPS) informando el progreso de la pista (`currentTime`, `duration`, `estado`) para que la barra de progreso visual avance fluidamente sin que Javascript tenga que calcular el tiempo.
- [ ] **Transición del Gestor `audioEngine.js`:** Modificar los métodos actuales del archivo para que en lugar de ejecutar lógicas locales, empaqueten los comandos y los envíen a través del puente de Rust.

## 🟢 Fase 5: Distribución y Empaquetado (Completado)
- [x] Configurar `electron-builder` o `electron-packager` en el `package.json`.
- [x] Crear scripts de compilación para Windows (`.exe` portable o instalador) y Linux (`AppImage` o `.deb`).
- [x] Asegurar que todas las dependencias nativas queden incluidas dentro del ejecutable final para que el usuario no necesite instalar Node.js ni bibliotecas adicionales.
