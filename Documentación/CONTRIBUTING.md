# Guía de Contribución / Contributing Guidelines

¡Gracias por tu interés en contribuir a la **LF Botonera de Efectos**! Este proyecto es software libre y comunitario, por lo que toda ayuda es bienvenida.

## 📚 Documentación Principal (Lectura Obligatoria)

Para no duplicar información, todas las reglas de desarrollo y la arquitectura del proyecto se encuentran en la carpeta `Documentación/`. Antes de enviar un Pull Request (PR), **debes leer y comprender** los siguientes archivos:

1. **[Reglas del Proyecto](REGLAS_PROYECTO.md):** Las 11 reglas inmutables del proyecto. Todo PR que incumpla estas reglas será rechazado (ej: no añadir librerías JS sin justificación profunda, mantener el límite de 200 líneas por archivo).
2. **[Arquitectura](ARCHITECTURE.md):** Explicación del modelo "Núcleo + Motores" (Backend en 5 capas, Frontend en 3 capas).
3. **[Guía de Compilación](../COMPILAR.md):** Pasos detallados para configurar tu entorno (Node.js, Rust, Vite) y compilar la aplicación.

## 🛠️ Cómo Empezar

1. Haz un **fork** de este repositorio.
2. Clona tu fork localmente: `git clone https://github.com/TU_USUARIO/LF-Botonera-de-efectos.git`
3. Crea una rama para tu feature o fix: `git checkout -b feature/mi-nueva-funcionalidad` o `git checkout -b fix/error-audio`
4. Realiza tus cambios, asegurándote de seguir el estándar de 200 líneas y la arquitectura de motores.
5. Haz commit explicando claramente qué cambia y por qué.
6. Envía tu código (Push) a tu fork y abre un **Pull Request** hacia la rama `main` del repositorio original.

## 🐛 Reportar Bugs y Solicitar Funciones

Por favor, utiliza las **Plantillas de Issues** proporcionadas al crear un Issue en GitHub. Esto nos asegurará tener la información necesaria (como tu sistema operativo y la versión del programa) para reproducir el error.

## 🤖 Uso de Inteligencia Artificial

Las IAs no son colaboradoras del proyecto. Los Commits, PRs y cualquier contribución registrada van **únicamente a nombre de usuarios humanos reales** con cuenta de GitHub. Está terminantemente prohibido incluir firmas de asistente, "Co-Authored-By: IA", o menciones a la IA en el historial de Git o en los comentarios del código fuente. Si usas IA para generar código, asegúrate de revisarlo, entenderlo y hacerte responsable de él. (Regla 11 del proyecto).

¡Esperamos con gusto tus contribuciones!
