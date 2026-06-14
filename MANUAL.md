# Manual de Usuario — LF Botonera de Efectos

**Versión:** consulta Ajustes → Acerca de  
**Autor:** Luis Fernando Velásquez  
**Licencia:** GPL-3.0

---

## 1. Primeros pasos

Al iniciar la aplicación por primera vez aparece un asistente que pregunta:

1. **¿Activar el módulo de Locuciones Dinámicas?**  
   Solo para emisoras de radio que necesiten anuncios de hora y clima. Si solo usas efectos de sonido, deja esta opción desactivada.

2. **¿Tienes LF Automatizador instalado?**  
   Activa la conexión por red (WebSockets) para usar la Botonera como segunda pantalla remota del Automatizador.

Puedes cambiar ambas opciones más adelante desde **Configuración Global**.

---

## 2. La cuadrícula de botones

Cada **pestaña** contiene una cuadrícula de botones (filas × columnas configurables).

### Asignar un archivo de audio

- **Arrastra y suelta** un archivo directamente sobre un botón vacío.
- O haz **clic derecho** sobre un botón vacío y elige **Editar...** para buscarlo manualmente.

Formatos compatibles: MP3 · WAV · FLAC · OGG/Vorbis · OGG/Opus · AAC · M4A · AIFF

### Reproducir

- **Clic izquierdo** sobre un botón con audio → reproduce.
- El botón muestra una barra de progreso y cuenta regresiva mientras suena.
- La pestaña se resalta en verde si algún botón está sonando en ese momento.

### Comportamientos individuales por botón

Haz **clic derecho** sobre un botón con archivo para cambiar su comportamiento:

| Opción | Efecto |
|---|---|
| **Bucle** | El audio se repite indefinidamente. |
| **Superposición** | Cada clic agrega una capa nueva sin detener la anterior. |
| **Reiniciar al hacer clic** | Si el audio ya suena, lo reinicia desde el inicio. |
| **Detener otros** | Al activarse, detiene todos los demás botones. |

### Editar un botón

Clic derecho → **Editar...** permite cambiar:
- Archivo de audio y tipo de efecto
- Nombre visible y colores
- Volumen individual
- Tecla de atajo directa

### Limpiar un botón

Clic derecho → **Limpiar** elimina el audio asignado. El botón queda vacío y disponible.

### Reordenar botones

Mantén **Alt** pulsado y arrastra un botón a otra posición. Si el destino está vacío, el botón se mueve; si tiene contenido, los botones se intercambian.

---

## 3. Pestañas

Cada pestaña es una botonera independiente con su propia cuadrícula, nombre y salida de audio.

- **Nueva pestaña**: clic en el botón **+** junto a las pestañas.
- **Clic derecho sobre una pestaña** para editar nombre/tamaño, exportar, importar o eliminar.
- El **tamaño de la cuadrícula** (filas y columnas) se configura al crear o editar la pestaña.
- **Salida de audio por pestaña**: cada pestaña puede usar una tarjeta de sonido diferente.

### Atajos de teclado para pestañas

En **Modo Mapeo** puedes asignar una tecla para saltar directamente a cualquier pestaña.

---

## 4. Perfiles

Un **perfil** agrupa todas tus pestañas y configuración de atajos. Puedes tener perfiles distintos para diferentes programas o estudios.

- El selector de perfil está en la barra superior (icono de engranaje junto al nombre del perfil).
- Puedes **importar y exportar** perfiles completos (`.bdeplf`), compatibles con LF Automatizador.

---

## 5. Modos de reproducción global

La **barra inferior** contiene seis botones que afectan a TODOS los botones de la botonera:

| Modo | Icono | Comportamiento |
|---|---|---|
| **Normal** | ▶ | Cada botón usa su configuración individual. |
| **Loop** | ↻ | Todos los botones repiten indefinidamente. |
| **Stack** | ⊕ | Los botones se acumulan (superposición global). |
| **Restart** | ↺ | Clic en un botón que ya suena → lo reinicia. |
| **Solo** | ⊙ | Al activar un botón, silencia todos los demás. |
| **Stop All** | ■ | Detiene todos los sonidos inmediatamente. |

El modo activo se muestra resaltado. **Stop All** no es un estado persistente, es una acción inmediata.

---

## 6. Barra de estado inferior

### Reloj / Contador

- Muestra la **hora actual** en formato digital.
- Cuando hay audio sonando, muestra el **tiempo restante** del sonido más largo activo.
- **Clic derecho** sobre el reloj para cambiar entre formato 12 h y 24 h.

### Vúmetro estéreo (L / R)

- Muestra el nivel de audio combinado de todos los botones activos.
- **Verde** (0–80 %): nivel normal.
- **Naranja** (80–90 %): nivel alto, cuidado con la ganancia.
- **Rojo** (90–100 %): saturación inminente.
- Decae automáticamente cuando el audio baja o se detiene.

---

## 7. Atajos de teclado

### Teclas reservadas del sistema

| Tecla | Acción |
|---|---|
| **ESC** | Cancela o cierra cualquier modal abierto. |
| **ENTER** | Guarda o acepta el modal activo. |

Estas dos teclas **no pueden asignarse** como atajos de botones o pestañas.

### Atajos globales

En **Configuración Global → Atajos del Teclado** puedes asignar:

- **Detener TODOS los sonidos**: una tecla que para todo en cualquier momento.
- **Pestaña siguiente / anterior**: navegar entre pestañas desde el teclado.

Haz clic en el cuadro de la tecla y pulsa la combinación deseada. Backspace o Supr limpia el atajo.

### Atajos por botón y pestaña (Modo Mapeo)

1. Ve a **Configuración Global → Atajos del Teclado** y pulsa **✏️ Asignar atajos**.
2. Aparece un banner naranja: la aplicación entra en Modo Mapeo.
3. Haz clic sobre cualquier **botón** o **pestaña** para asignarle una tecla.
4. En el cuadro que aparece, pulsa la tecla deseada y haz clic en **Guardar Atajo**.
5. Para borrar un atajo: abre el cuadro de ese botón/pestaña, pulsa **Backspace** y guarda.
6. Pulsa **ESC** en cualquier momento para salir del Modo Mapeo.

### Detección de atajos huérfanos

Si un botón tenía un atajo y su archivo de audio fue eliminado o vaciado,
el panel **Atajos del Teclado** mostrará una lista de atajos inactivos con un
botón **Limpiar** para eliminarlos.

---

## 8. Pre-escucha (Prelisten)

La pre-escucha permite escuchar un archivo en una **salida de audio diferente** (auriculares del operador) antes de reproducirlo en antena.

- Clic derecho sobre un botón → **Escucha previa**.
- O desde el menú **Editar...**, botón **Pre-escuchar**.
- En el panel de pre-escucha puedes ver el progreso y detenerlo con el botón **Stop**.

La salida de pre-escucha se configura en **Configuración Global → Principal** (campo "Salida de Pre-escucha").

---

## 9. Configuración Global

Abre el engranaje ⚙ en la esquina superior derecha.

### Pestaña Principal

- **Tema**: Oscuro / Claro / Predeterminado del sistema (se adapta automáticamente).
- **Idioma**: Español / English.
- **Salida Principal**: tarjeta de sonido para las botoneras.
- **Salida Pre-escucha**: tarjeta separada para el operador.

### Pestaña Atajos del Teclado

- Atajos globales (detener todo, siguiente/anterior).
- Modo Mapeo para asignar atajos a botones y pestañas.
- Sección de atajos inactivos (huérfanos) con opción de limpiarlos.

### Pestaña Hora y Clima (módulo opcional)

Solo disponible si activaste el módulo en el asistente inicial.

- Configura carpetas de locuciones de hora (`HRS00`–`HRS23`, `MIN00`–`MIN59`).
- Configura ciudad y unidad para el clima (temperatura y humedad).
- Botones de prueba para verificar que los archivos están correctamente asignados.

---

## 10. Exportación e importación

### Pestañas

Clic derecho sobre una pestaña → **Exportar (.bdelf)** / **Importar (.bdelf)**.

Los archivos `.bdelf` son compatibles con el **LF Automatizador v1.0**.

### Perfiles completos

Menú de perfil (icono junto al nombre del perfil) → **Exportar Perfil (.bdeplf)** / **Importar Perfil (.bdeplf)**.

---

## 11. Compatibilidad con LF Automatizador

Si el LF Automatizador está instalado en la misma máquina, la Botonera puede actuar como **segunda pantalla remota** a través de la red local. La conexión se configura en el asistente inicial.

Los formatos de archivo `.bdelf` y `.bdeplf` son los mismos en ambas aplicaciones.

---

## 12. Preguntas frecuentes

**¿Por qué no suena nada?**  
Verifica la salida de audio en Configuración Global → Principal. Asegúrate de que la tarjeta seleccionada es la correcta.

**El botón muestra el nombre del archivo en mayúsculas. ¿Puedo cambiarlo?**  
Sí: clic derecho → Editar → campo Nombre.

**¿Puedo usar la misma tecla como atajo en dos botones?**  
No. Si asignas una tecla que ya está en uso en otro botón, el comportamiento es indefinido. El sistema usa el primero que encuentre.

**¿Cómo elimino un perfil?**  
Menú de perfil → Eliminar Perfil actual. No se puede eliminar el único perfil existente.

**¿Dónde se guarda la configuración?**  
En la carpeta de datos del usuario del sistema operativo:
- **Windows**: `%APPDATA%\lf-botonera\config.json`
- **Linux**: `~/.local/share/lf-botonera/config.json`
