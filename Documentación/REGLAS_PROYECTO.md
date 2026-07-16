# Principios de desarrollo — LF Botonera de Efectos

Estas directrices definen los estándares de calidad del proyecto. Están pensadas para orientar a cualquier desarrollador o colaborador de IA antes de realizar cambios en el código.

---

### 1. Adaptar, no transcribir

Cuando se toma inspiración del LF Automatizador u otro proyecto externo, se analiza la lógica, se comprende el propósito y se reimplementa adaptada al contexto de la Botonera. Incorporar código de otras bases sin entenderlo y ajustarlo introduce deuda técnica difícil de detectar después.

### 2. Soluciones desde la raíz

Cuando aparece un bug o una limitación, la solución correcta ataca la causa real del problema. Añadir condiciones defensivas, capturar errores para silenciarlos, o añadir código adicional alrededor de lógica rota desplaza el problema en lugar de resolverlo y acumula complejidad innecesaria con el tiempo.

### 3. Archivos pequeños con responsabilidad única

Ningún archivo de código supera las 200 líneas. Cada módulo tiene una responsabilidad clara y acotada. Si una función crece o una nueva característica requiere más espacio, se crea un módulo separado. Medir con `wc -l` (POSIX); PowerShell `Measure-Object -Line` puede descontar la última línea sin salto y dar un resultado menor al real.

### 4. La interfaz gráfica no toma decisiones

El frontend (HTML/CSS/JS) se encarga de dibujar la interfaz y de traducir la interacción del usuario en comandos IPC hacia Rust. No contiene lógica de audio, temporizadores críticos, validaciones de datos ni cálculos de estado. Toda esa responsabilidad vive en el backend Rust, que comunica los resultados al frontend a través de eventos o respuestas IPC.

### 5. JavaScript requiere justificación

Antes de escribir lógica en JavaScript, la pregunta de referencia es: *¿puede esto resolverse en Rust?* JavaScript es el lenguaje de la capa de presentación; Rust es el lenguaje de la lógica. Elegir JS porque es el camino más rápido no es un argumento técnico válido. JS está justificado cuando se trata de una interacción visual inmediata o cuando depende de una API del navegador que no tiene equivalente en el backend.

### 6. Compatibilidad bidireccional con LF Automatizador

Los formatos `.bdelf` y `.bdeplf` son compartidos con el LF Automatizador. Todo campo nuevo añadido al modelo de datos debe llevar `#[serde(default)]` para que la otra aplicación pueda leer el archivo ignorando ese campo sin errores. No se añaden campos obligatorios al esquema de exportación sin coordinar el cambio en ambos proyectos.

### 7. Sin texto hardcodeado en la interfaz

Ningún texto visible en la UI se escribe directamente en el código. Todo pasa por el sistema i18n (`t(key)` en JS, clave en los cuatro archivos de traducción). El idioma de referencia es `es.json`; el mismo cambio debe reflejarse en `en.json`, `pt-BR.json` y `pt-PT.json`.

### 8. Tema claro y oscuro sin parpadeo

Los colores de la interfaz se definen exclusivamente mediante CSS custom properties. No se cambian clases en la carga inicial de manera que provoquen un flash de pantalla blanca. Los colores asignados por el usuario se adaptan para garantizar contraste en ambos temas mediante `colorAdapter.js`.

### 9. Proponer antes de reestructurar

Cuando un cambio afecta la estructura de módulos, el esquema de datos, el flujo IPC o la compatibilidad con LFA, se presenta el plan primero y se espera confirmación del autor del proyecto antes de escribir código. La implementación sin alineación previa en puntos arquitectónicos puede generar trabajo redundante o conflictos difíciles de deshacer.

### 10. Verificación sin lanzar la aplicación

El método estándar de verificación es `cargo test --lib` **y `cargo build --lib`** para el backend, y `npm run build` para el frontend. Hacen falta las dos comprobaciones de Rust: `cargo test` compila el código *junto con* las pruebas, y un `use super::*` de un fichero de pruebas puede mantener vivo un import que el módulo ya no usa; quien ejecuta la aplicación compila sin las pruebas y sí ve ese aviso. Una verificación que no reproduce cómo se compila de verdad no es una verificación. La prueba visual y funcional la realiza el usuario en su equipo. No se controla la pantalla del usuario ni se utilizan herramientas de automatización de escritorio como parte del flujo de verificación.

### 11. Solo personas reales como colaboradores

El historial de git, los commits, las descripciones de pull request y los comentarios de código reflejan únicamente el trabajo de usuarios humanos reales con cuenta de GitHub. Las herramientas de IA que asisten en el desarrollo no aparecen como colaboradoras, coautoras ni firmantes. No se añaden trailers `Co-Authored-By` de asistentes de IA ni atribuciones de ningún tipo a modelos de lenguaje en el registro del proyecto. El reconocimiento al uso de herramientas de IA durante el desarrollo está documentado en la sección "Créditos de desarrollo" del `README.md` público.

### 12. Seguridad y dependencias con criterio

Toda dependencia nueva debe justificarse antes de añadirse. La evaluación mínima incluye: necesidad real, mantenimiento activo, licencia compatible con GPL-3.0-or-later, superficie de seguridad, tamaño/impacto en el build y si existe una alternativa ya presente en el proyecto o en la biblioteca estándar. No se incorporan paquetes por comodidad si el coste de mantenimiento o riesgo supera el beneficio.

### 13. Documentación junto al cambio

Todo cambio que modifique arquitectura, estructura de módulos, comandos IPC, modelo de datos, formatos de exportación, reglas de negocio o flujos relevantes debe actualizar la documentación correspondiente en el mismo cambio. Según el alcance, revisar `ARCHITECTURE.md`, `GLOSARIO.md`, `LIBRO_PROYECTO.md`, `AGENTS.md`, `CLAUDE.md`, `CHANGELOG.md` y las guías de `Documentación/Reorganización/` si aplica. La documentación no es una tarea posterior: forma parte de la definición de terminado.

### 14. Espacio de trabajo limpio

Los archivos temporales, planes de implementación completados y artefactos de compilación que han dejado de ser necesarios se eliminan del repositorio. El repo contiene código, documentación permanente y configuración; no ficheros de trabajo en curso.

---

*Para las instrucciones específicas de cada herramienta de IA, ver [`CLAUDE.md`](../CLAUDE.md) y [`AGENTS.md`](../AGENTS.md).*
