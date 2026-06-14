<div align="center">
  <h1>🎛️ LF Botonera de efectos</h1>
  <p><i>Software profesional de reproducción de efectos de sonido diseñado para radio y producción en vivo.</i></p>
  <img src="https://img.shields.io/badge/Plataforma-Windows%20%7C%20Linux-success?style=for-the-badge&logo=linux" alt="OS Compatibility" />
  <img src="https://img.shields.io/badge/Versión-1.0.0-blue?style=for-the-badge" alt="Version" />
</div>

---

## 🌟 Descripción General
**LF Botonera de efectos** es una herramienta potente y ligera creada por **Luis Fernando Velásquez**. Está optimizada para ofrecer baja latencia, superposición de audios, soporte multipantalla y un enrutamiento avanzado (pre-escucha y salida global). Es la solución perfecta para locutores, productores y streamers que requieren un control total e instantáneo sobre sus efectos de sonido.

## 🚀 Características Principales
- 📂 **Organización por Perfiles:** Soporte para múltiples perfiles de usuario y pestañas (paletas) de botones.
- 🎨 **Personalización Total:** Modifica el color de fondo, texto, volumen y atajos de teclado de cada botón de forma individual.
- 🎚️ **Modos de Reproducción:** 
  - **Simple:** Reproduce la pista de principio a fin.
  - **Bucle (Loop):** Repetición infinita del efecto.
  - **Superposición (Overlap):** Permite disparar múltiples copias del mismo audio simultáneamente (ideal para aplausos, risas o fanfarrias).
  - **Exclusivo:** Detiene todos los demás sonidos activos al momento de dispararse.
- 🎧 **Enrutamiento Avanzado:** Separa tu canal de salida principal del canal de pre-escucha (monitor).
- 🖱️ **Flujo de Trabajo Ágil:** Soporta *Drag and Drop* (arrastrar y soltar) de archivos de audio directamente sobre las casillas.
- 💾 **Respaldos Rápidos:** Exportación e importación de perfiles completos (`.bdeplf`) y botoneras individuales (`.bdelf`).

## 💻 Compatibilidad 100% Multiplataforma
Gracias a su moderna arquitectura basada en **Electron**, el software funciona de manera nativa y sin configuraciones extrañas en:
- 🪟 **Windows** (Soporte interno DirectSound / WASAPI).
- 🐧 **Linux** (Soporte interno PulseAudio / ALSA / PipeWire).

## 🛠️ Instalación y Uso
Para el usuario final, **no se requiere instalar Node.js ni dependencias adicionales**. El programa ha sido preparado para empaquetarse como ejecutable *Standalone* (instalador rápido en Windows y formato `.AppImage` para distribuciones Linux).

Si eres desarrollador y deseas correr el código fuente desde tu terminal:
```bash
# 1. Clonar el repositorio
git clone https://github.com/yosoyluisfernando/LF-Botonera-de-efectos.git

# 2. Entrar en la carpeta
cd LF-Botonera-de-efectos

# 3. Instalar las dependencias
npm install

# 4. Iniciar la aplicación
npm start
```
*(Nota: Si prefieres evitar la terminal, puedes dar doble clic sobre los archivos `Iniciar_Botonera.vbs` en Windows o `Iniciar_Botonera.sh` en Linux).*

## 🔮 Próximos Lanzamientos: El Motor de Rust
A nivel interno, el código de la interfaz ha sido completamente refactorizado hacia una arquitectura de **"Thin Client"** (Cliente Ligero), desvinculando la lógica visual de las tareas complejas de audio.

En nuestra **próxima gran actualización**, integraremos un **motor de audio programado puramente en Rust**. Esto potenciará el software a niveles de grado "Broadcast", trayendo:
- ⚡ **Latencia Cero Absoluta:** Rust hablará directamente con el hardware sin intermediarios web.
- 🧠 **Caché en RAM Inteligente:** El audio se decodificará en segundo plano, liberando a la interfaz gráfica del trabajo pesado y eliminando el riesgo de bloqueos.
- 🎛️ **Control de Hardware Nativo:** Detección y enrutamiento de tarjetas de sonido usando librerías de bajo nivel (CPAL).

---
<div align="center">
  <p>Desarrollado con ❤️ para profesionales del audio por <b>Luis Fernando Velásquez</b>.</p>
  <p><i>Este es un software gratuito, prohibida su venta.</i></p>
</div>
