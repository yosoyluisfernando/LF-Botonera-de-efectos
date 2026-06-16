# ⚖️ REGLAS INMUTABLES DEL PROYECTO

Estas reglas guían todo el desarrollo de la "LF Botonera de Efectos" y su migración a Tauri. Cualquier desarrollador o agente de IA que contribuya al código **debe leer y acatar estrictamente** estas normas antes de escribir una sola línea de código. 

### 1. No Copiar y Pegar Ciegamente (Analizar y Traducir)
Si hay dudas sobre cómo hace algo el "LF Automatizador", se investiga a fondo su código fuente, se entiende el *porqué* y el *cómo*, y **se traduce/adapta** a las necesidades específicas de la botonera. Está rotundamente prohibido copiar y pegar código ciegamente sin entenderlo y optimizarlo para este entorno.

### 2. Cero "Parches sobre Parches" (Soluciones desde la Raíz)
Si se descubre un error o se necesita una mejora, se soluciona desde la raíz lógica del problema. Si algo requiere dividir un archivo o reestructurar una lógica, se hace. Está terminantemente prohibido apilar código espagueti para ocultar errores.

### 3. Límite Estricto de Líneas (Modularidad Extrema)
**Un archivo no puede exceder las 150 - 200 líneas de código.** 
Cada archivo y módulo debe tener una única responsabilidad (Single Responsibility Principle). Si se debe añadir una nueva función o característica, se crea un archivo nuevo. Todo se debe separar en pequeños bloques legibles.

### 4. La Interfaz Gráfica es un "Humilde Control Remoto"
El Frontend (HTML/JS/CSS) no piensa, no gestiona bases de datos, ni procesa audio. Su único trabajo es dibujar botones bonitos y ser un puente que envía órdenes (por ejemplo: "El usuario hizo clic en el botón 3") hacia el motor de Rust. Toda la lógica dura, enrutamiento, temporizadores y cargas, ocurre en Rust (Tauri Backend).

### 5. Compatibilidad Bidireccional de Formatos
Todo lo que se exporte (pestañas o perfiles) debe ser 100% compatible para abrirse en el *LF Automatizador* sin errores, y viceversa. Tauri servirá como un traductor/adaptador (rellenando campos faltantes) para que los archivos legados `.bdelf` sigan funcionando pero escalen a la nueva estructura de datos.

### 6. Internacionalización (i18n) Inmediata
No se permite escribir texto estático o "quemado" en el código (hardcoded strings) en la interfaz gráfica visible. Todos los textos deben venir de archivos de traducción (ej. `en.json`, `es.json`). 

### 7. Soporte para Tema Claro y Oscuro Nativo
Todo el diseño debe soportar Tema Claro y Oscuro dinámicamente mediante el uso de variables CSS. No deben existir parpadeos blancos al abrir la aplicación.

### 8. Documentación Exhaustiva para Código Abierto
Este proyecto es Software Libre. Todo archivo, función importante, evento de red o canal IPC debe estar ampliamente documentado (usando `///` en Rust o `/** */` en JS) explicando *qué hace, por qué lo hace y qué recibe/devuelve*, para que cualquier nuevo desarrollador entienda el código fácilmente.

### 9. Limpieza Estricta del Espacio de Trabajo (Cero Basura)
Todo archivo temporal generado para compilar el motor Rust, logs de pruebas o compilaciones antiguas debe ser **eliminado** inmediatamente después de que deje de ser útil. Si se genera una nueva compilación de prueba, la vieja se borra. El espacio de trabajo debe permanecer inmaculado en todo momento.

### 10. Rust Primero, JavaScript Solo Cuando Sea Necesario
Todo comportamiento que pueda resolverse en Rust debe enviarse al backend de Tauri. JavaScript debe limitarse a dibujar la interfaz, capturar interacciones del usuario y llamar comandos IPC; solo puede contener lógica propia cuando sea estrictamente necesario para la experiencia visual inmediata o para integrar APIs del navegador que no existan en Rust.

---
**⚠️ CLÁUSULA DE LECTURA FORZADA PARA IA:**
Como Inteligencia Artificial, en CADA NUEVO TURNO antes de hacer una sola modificación al código, tengo la **obligación inquebrantable** de recordar conscientemente estas 10 reglas. Específicamente, debo preguntarme: *"¿Estoy poniendo lógica pesada en el Frontend?"* y *"¿Esto podría vivir de forma más segura en Rust?"*. Si la respuesta exige mover lógica al backend, debo detener el proceso inmediatamente y solucionarlo desde la raíz.
