# 🗺️ PLAN MAESTRO: LF Botonera de Efectos (Tauri + Rust)
*Última actualización: 2026-06-12 — basado en auditoría doble de Respaldo_Electron y LF Automatizador v1.0*

---

## Visión General
Construir LF Botonera de Efectos en Tauri + Rust partiendo de la **maqueta Electron** como referencia visual y funcional, e incorporando las **mejoras que el LF Automatizador ha acumulado** sobre esa misma base. La interfaz será un "humilde control remoto" que dibuja botones y envía órdenes; todo el pensamiento y el audio viven en Rust.

---

## 📊 Estado Actual del Proyecto (Post-Auditoría)

### Lo que YA TENEMOS funcionando:
| Componente | Estado |
|---|---|
| Base Tauri v2 + Vite | ✅ Compilando |
| Motor de audio Rust (rodio) | ✅ Básico (play/stop) |
| API wrapper JS seguro (`api.js`) | ✅ |
| Cuadrícula 5×5 estática | ✅ |
| Asistente de primer arranque | ✅ |
| Persistencia config + grid (APPDATA) | ✅ |
| Temas claro/oscuro (CSS vars) | ✅ |
| i18n básico (es.json) | ✅ |
| `clear_button`, `update_button_data` IPC | ✅ |

### Lo que FALTA (priorizado por impacto):
| Función | Fuente | Prioridad |
|---|---|---|
| Sistema de Perfiles completo | Maqueta | 🔴 Crítico |
| Pestañas múltiples por perfil | Maqueta | 🔴 Crítico |
| Grid configurable por pestaña | Maqueta | 🔴 Crítico |
| Menú contextual completo (loop/overlap/stopOther) | Maqueta | 🔴 Crítico |
| Modal edición completo (volumen, atajo, prelisten) | Maqueta | 🔴 Crítico |
| Indicadores visuales (verde = reproduciendo) | Maqueta | 🔴 Crítico |
| Progress bar en tiempo real (eventos Rust→JS) | Maqueta | 🔴 Crítico |
| Exportar/Importar .bdelf / .bdeplf | Maqueta + LFA | 🔴 Crítico |
| Sistema de atajos de teclado | Maqueta | 🟠 Alto |
| Swap D&D entre botones | Maqueta | 🟠 Alto |
| Prelisten (pre-escucha) | Maqueta | 🟠 Alto |
| Enrutamiento dual audio (main + prelisten) | Maqueta | 🟠 Alto |
| Botón "Detener Todo" global | Maqueta | 🟠 Alto |
| Flag `restart` (reiniciar si está sonando) | LF Automatizador | 🟡 Medio |
| Campo `type` (audio/time/temperatura/humedad) | LF Automatizador | 🟡 Medio |
| Duración pre-calculada y guardada en disco | LF Automatizador | 🟡 Medio |
| Locuciones dinámicas (hora/clima) | LF Automatizador | 🟢 Opcional |
| WebSocket bridge con LF Automatizador | Plan Original | 🟢 Opcional |

---

## 🗂️ Esquema de Datos Definitivo

Este es el formato **universal** que usarán `.bdelf`, `.bdeplf` y el archivo interno `botonera_config.json`. Es 100% compatible bidirección con LF Automatizador.

### botonera_config.json (archivo principal)
```json
{
  "activeProfileId": "string",
  "profiles": [ /* ver Profile */ ]
}
```

### Estructura Profile
```json
{
  "id": "timestamp_random",
  "name": "Principal",
  "bg": "#008c3a",
  "text": "#ffffff",
  "config": {
    "outMain": "default",
    "outPre":  "default",
    "keys": {
      "stopAll": "Ctrl+Space",
      "next":    "Ctrl+Right",
      "prev":    "Ctrl+Left"
    }
  },
  "paletas": [ /* ver Paleta */ ]
}
```

### Estructura Paleta (Tab/Botonera)
```json
{
  "nombre":  "BOTONERA 1",
  "rows":    6,
  "cols":    5,
  "audioOut": "global",
  "shortcut": "",
  "tabBg":   "#3a3f44",
  "tabText": "#cccccc",
  "botones": [ /* ver Botón */ ]
}
```

### Estructura Botón (COMPLETA — incluye campos LF Automatizador)
```json
{
  "id":        1,
  "label":     "1",

  "type":      "audio",
  "file":      "C:/ruta/al/archivo.mp3",
  "folder":    "",
  "name":      "APLAUSOS",
  "bg":        "#FF6B35",
  "text":      "#FFFFFF",
  "vol":       0.85,
  "duration":  30.5,

  "loop":      false,
  "stopOther": false,
  "overlap":   false,
  "restart":   false,

  "shortcut":  "Ctrl+A"
}
```

### Archivos de exportación
- **`.bdelf`** — Exportación de una sola pestaña (contiene estructura Paleta completa)
- **`.bdeplf`** — Exportación de un perfil completo (contiene estructura Profile completa)

### Regla de compatibilidad (Regla 5)
El lector siempre rellena campos faltantes con valores por defecto. Esto garantiza que:
- Un `.bdelf` creado en la botonera se abre en LF Automatizador (ignora `type`/`folder`, los rellena como "audio"/"")
- Un `.bdelf` creado en LF Automatizador se abre en la botonera (los campos `type`/`folder` se guardan aunque no se usen)

---

## 🛠️ Fases de Implementación

### Fase 0: Base — COMPLETADA ✅
- Proyecto Tauri + Vite inicializado
- Motor Rust con `play_audio`, `stop_audio`, `clear_button`, `update_button_data`
- Wizard de primer arranque
- Persistencia de config y grid en `%APPDATA%\LF Botonera\`
- Sistema de temas y i18n

---

### Fase 1: Modelo de Datos Completo (Rust)
*Objetivo: El motor Rust entiende el esquema completo ANTES de que exista la UI.*

1. **`config.rs`** — Reemplazar `ButtonData` y `GridState` por el esquema completo:
   - `ButtonData`: todos los campos del esquema definitivo
   - `PaletaData`: `nombre`, `rows`, `cols`, `audioOut`, `shortcut`, `tabBg`, `tabText`, `botones`
   - `ProfileData`: `id`, `name`, `bg`, `text`, `config`, `paletas`
   - `AppConfig`: `activeProfileId`, `profiles`

2. **Normalizer en Rust**: Función que carga un JSON y rellena campos faltantes con defaults. Esto asegura la compatibilidad con archivos viejos y con los del LF Automatizador.

3. **Nuevos comandos IPC**:
   - `get_profiles` → devuelve todos los perfiles
   - `set_active_profile(id)` → cambia el perfil activo
   - `create_profile(name)` → crea perfil con pestaña vacía por defecto
   - `delete_profile(id)` → elimina (mínimo 1 debe quedar)
   - `update_profile_meta(id, name, bg, text)` → cambia nombre y color
   - `create_tab(profile_id, nombre, rows, cols)` → añade pestaña
   - `delete_tab(profile_id, tab_index)` → elimina pestaña (mínimo 1)
   - `update_tab_meta(profile_id, tab_index, ...)` → renombra/redimensiona
   - `reorder_buttons(profile_id, tab_index, from_id, to_id)` → swap de botones
   - `export_tab(profile_id, tab_index)` → retorna JSON de la paleta para guardar en .bdelf
   - `import_tab(profile_id, json)` → importa una paleta desde JSON
   - `export_profile(profile_id)` → retorna JSON del perfil para guardar en .bdeplf
   - `import_profile(json)` → importa un perfil desde JSON
   - `get_audio_devices` → lista dispositivos de salida del SO
   - `probe_duration(path)` → calcula la duración de un archivo de audio

4. **Motor de audio mejorado** (`audio.rs`):
   - Soporte para `loop` (bucle infinito)
   - Soporte para `overlap` (múltiples instancias simultáneas del mismo ID)
   - Soporte para `stop_other` (al reproducir, para todos los demás del mismo tab)
   - Soporte para `restart` (si ya suena, reinicia desde 0)
   - **Eventos de progreso**: emitir `cartwall-progress {id, current_time, duration}` al frontend vía Tauri events
   - Soporte para prelisten (dispositivo de salida separado)
   - Enrutamiento por dispositivo (outMain / outPre)
   - `stop_all` — para todo
   - `stop_tab(tab_index)` — para todo un tab

---

### Fase 2: Sistema de Perfiles y Pestañas (Frontend)
*Ley: La maqueta Electron es la referencia visual. Sin inventar nada nuevo.*

Nuevos archivos JS (ninguno supera 200 líneas):

- **`src/js/profiles.js`** — Lógica de UI para el botón `👤 Perfil`:
  - Menú desplegable: lista de perfiles + opciones (nuevo, editar, eliminar, exportar, importar)
  - Refleja en el botón el color y nombre del perfil activo
  
- **`src/js/tabs.js`** — Lógica de UI para las pestañas:
  - Renderizar, crear, cambiar, indicador visual "tab verde" cuando hay audio
  - Menú contextual de pestaña (editar, exportar, importar, eliminar)

- **`src/js/profileModal.js`** — Modal para crear/editar perfil (nombre + colores)

- **`src/js/tabModal.js`** — Modal para crear/editar pestaña (nombre, filas, columnas, colores)

- **`src/js/importer.js`** — Lógica de importación/exportación .bdelf y .bdeplf

---

### Fase 3: Funciones Completas por Botón (Frontend)
*La maqueta es la ley. Cada opción del menú contextual debe existir.*

Archivos a completar/reemplazar:

- **`src/js/contextMenu.js`** — Menú contextual completo:
  - `Editar...` → abre editModal
  - `Limpiar` → llama `clear_button`
  - `─────────`
  - `Bucle` (✓ si activo) → toggle `loop`
  - `Reproducción superpuesta` (✓ si activo) → toggle `overlap`
  - `Reiniciar al pulsar` (✓ si activo) → toggle `restart`
  - `Detener al pulsar otro` (✓ si activo) → toggle `stop_other`
  - `─────────`
  - `Escucha previa` → abre panel prelisten

- **`src/js/editModal.js`** — Modal completo:
  - Campo: Ruta del archivo (read-only + botón `...`)
  - Campo: Nombre
  - Campo: Volumen (slider 0-100%)
  - Campo: Color de fondo + Color de texto
  - Campo: Tecla de atajo (input especial que captura la combinación)
  - Botón: Escucha previa
  - Botón: Aceptar / Cancelar

- **`src/js/prelisten.js`** — Panel flotante de pre-escucha:
  - Nombre del audio
  - Barra de progreso en tiempo real
  - Timer `00:15 / 00:30`
  - Botón Stop
  - Volumen independiente

- **`src/js/shortcuts.js`** — Sistema de atajos de teclado:
  - Listener global `keydown`
  - Busca en TODOS los tabs/botones del perfil activo
  - Prioridad: parar todo > siguiente tab > anterior tab > tab específico > botón específico
  - Captura de combinación para modal de asignación

---

### Fase 4: Motor de Audio Avanzado + Progress Bar
*Rust emite eventos, el frontend los escucha y actualiza barras de progreso.*

- Tauri events desde Rust hacia el frontend:
  - `audio://playing` → `{id, tab_index, duration}`
  - `audio://progress` → `{id, current_time, duration}` (cada ~100ms)
  - `audio://stopped` → `{id}`

- Frontend en `grid.js`:
  - Escuchar `audio://playing` → añadir clase `playing` (verde) al botón
  - Escuchar `audio://progress` → actualizar barra de progreso y timer
  - Escuchar `audio://stopped` → quitar clase `playing`, resetear barra

- Frontend en `tabs.js`:
  - Escuchar `audio://playing` → marcar tab como activa (verde)
  - Escuchar `audio://stopped` → desmarcar si no hay más audio en ese tab

---

### Fase 5: Configuración / Ajustes
*Panel de ajustes accesible desde el botón ⚙️.*

Nuevos archivos:
- **`src/js/settingsModal.js`** — Modal de ajustes con tres secciones:
  1. **Salidas de Audio**: dropdown "Salida principal" + dropdown "Salida pre-escucha"
  2. **Atajos Globales**: campo para "Detener todo", "Siguiente pestaña", "Pestaña anterior"
  3. **Acerca de**: nombre, versión, autor, licencia

---

### Fase 6: Locuciones Dinámicas *(Módulo Opcional)*
*Se activa/desactiva desde el Asistente de primer arranque Y desde Ajustes.*

- Nuevo tipo de botón: `type = "time"` / `"temperature"` / `"humidity"`
- En UI: botón con icono especial (🕐 / 🌡️ / 💧)
- Al editar: selector de carpeta en vez de archivo
- Motor Rust lee hora/valor del SO y busca archivo en la carpeta configurada
- Formato de archivo esperado: `14.mp3` (hora), `23.mp3` (valor), etc.
- **SIN API externa de clima** (usa datos del SO)

---

### Fase 7: WebSocket Bridge con LF Automatizador *(Módulo Opcional)*
*Solo activo si el usuario activó la opción en el asistente/ajustes.*

- La botonera levanta un cliente WebSocket (no servidor — el servidor es el LF Automatizador)
- Si el LF Automatizador está corriendo, la botonera le envía los comandos de reproducción en lugar de reproducir localmente
- Protocolo: JSON sobre WebSocket, mismo formato que usa el LF Automatizador v2

---

### Fase 8: Pruebas y Limpieza Final
1. Probar con 10+ pestañas × 30 botones × reproducción simultánea
2. Verificar compatibilidad de archivos: exportar desde botonera → importar en LF Automatizador y viceversa
3. Probar drag & drop desde Explorer en Windows 10, 11 y Linux
4. Verificar que atajos funcionan con ventana minimizada
5. Eliminar todos los archivos temporales de compilación de prueba (Regla 9)
6. Auditar que ningún archivo supera 200 líneas (Regla 3)
7. Auditar que no hay strings hardcodeados visibles en la UI (Regla 6)

---

## 🔄 Compatibilidad con LF Automatizador

| Escenario | Comportamiento |
|---|---|
| Botonera standalone (sin LFA) | Reproduce audio localmente con motor Rust |
| Botonera + LFA instalado, bridge OFF | Ambos son independientes, comparten formato de archivos |
| Botonera + LFA instalado, bridge ON | La botonera actúa como "segunda pantalla" — envía comandos al LFA |
| Exportar pestaña desde botonera → abrir en LFA | 100% compatible (LFA ignora campos que no conoce) |
| Exportar pestaña desde LFA → abrir en botonera | 100% compatible (botonera guarda campos aunque no los use) |

---

## 📁 Estructura de Archivos Final Objetivo

```
src/
├── index.html
├── css/
│   ├── theme.css        (variables claro/oscuro)
│   ├── main.css         (estructura, titlebar)
│   ├── grid.css         (cuadrícula de botones)
│   ├── tabs.css         (sistema de pestañas)
│   ├── contextMenu.css  (menú contextual)
│   └── modal.css        (modales comunes)
├── js/
│   ├── api.js           (wrapper seguro de Tauri IPC)
│   ├── i18n.js          (motor de traducción)
│   ├── theme.js         (init de tema sin parpadeo)
│   ├── main.js          (orquestador: wizard vs app)
│   ├── wizard.js        (asistente primer arranque)
│   ├── grid.js          (cuadrícula de botones)
│   ├── tabs.js          (sistema de pestañas)
│   ├── profiles.js      (sistema de perfiles)
│   ├── shortcuts.js     (atajos de teclado globales)
│   ├── contextMenu.js   (menú clic derecho)
│   ├── editModal.js     (modal edición de botón)
│   ├── prelisten.js     (panel de pre-escucha)
│   ├── profileModal.js  (modal crear/editar perfil)
│   ├── tabModal.js      (modal crear/editar pestaña)
│   ├── settingsModal.js (panel de ajustes)
│   └── importer.js      (exportar/importar archivos)
└── i18n/
    ├── es.json
    └── en.json

src-tauri/src/
├── main.rs              (entry point — no tocar)
├── lib.rs               (setup, comandos IPC, run)
├── config.rs            (structs de datos + persistencia)
└── audio.rs             (motor de audio + eventos de progreso)
```

---

## ⚠️ Decisiones de Diseño Tomadas

1. **Drag & Drop de archivos**: Se usa el evento nativo de Tauri `tauri://file-drop`, no el del navegador. Rust valida la extensión y devuelve el nuevo estado. La UI no decide si el archivo es válido.

2. **Drag & Drop de botones (swap)**: Se detecta con Alt+clic+arrastre (igual que la maqueta). Se envía `reorder_buttons(from_id, to_id)` a Rust, que intercambia posiciones y devuelve el nuevo estado.

3. **Duración del audio**: Se calcula en Rust (no en JS) usando la misma librería symphonia que ya decodifica el audio. Más preciso, sin timeouts, sin uso del hilo principal.

4. **Atajos de teclado**: Se registran como listeners JS en la ventana de la app. En una futura versión se pueden registrar como atajos globales del SO via Tauri (plugin `global-shortcut`). Por ahora, funcionan dentro de la ventana (igual que la maqueta).

5. **Formato de guardado**: Un solo archivo `botonera_config.json` con todos los perfiles. Mismo esquema que la maqueta para máxima compatibilidad.
