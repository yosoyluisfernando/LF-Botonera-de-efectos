# LF Botonera de efectos

**Versión:** 1.0.0
**Autor:** Luis Fernando Velasquez
**Licencia:** Por definir
**Descripción:** 
Software profesional de reproducción de efectos de sonido (Botonera) diseñado para entornos de radio y producción en vivo. Optimizado para ofrecer baja latencia, superposición de audios, soporte multipantalla y enrutamiento avanzado de audio (pre-escucha y salida global).

## Características Actuales
- Arquitectura basada en Electron (v29.0.0).
- Soporte para perfiles y múltiples pestañas (paletas) de botones.
- Personalización visual por botón (color, texto, volumen, atajos de teclado).
- Modos de reproducción: Simple, Bucle (Loop), Superposición (Overlap) y Exclusivo (Detener otros).
- Enrutamiento de audio configurable para Salida Principal y Pre-escucha.
- Arrastrar y soltar (Drag and Drop) archivos de audio.
- Exportación e importación de perfiles (`.bdeplf`) y botoneras (`.bdelf`).

## Compatibilidad de Sistema Operativo
- **Windows:** Soporte nativo y completo.
- **Linux:** En proceso de adaptación (Arquitectura cross-platform).

## Instalación y Ejecución (Usuario Final)
El software se distribuirá como un paquete ejecutable autónomo. El usuario final **no necesita instalar dependencias**, Node.js ni configurar nada adicional, tanto en Windows como en Linux.

## Entorno de Desarrollo
1. Clonar el repositorio.
2. Instalar las dependencias de desarrollo con `npm install`.
3. Ejecutar el programa con `npm start`.

---
*Este software está siendo preparado para ser un cliente ligero ("Thin Client") que eventualmente se comunicará con un motor de audio de alto rendimiento programado en Rust.*
