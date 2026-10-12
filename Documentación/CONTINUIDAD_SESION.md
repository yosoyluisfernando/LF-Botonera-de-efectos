# Continuidad de sesión — estado actual

**Actualizado:** 2026-10-11
**Rama:** `main`
**Versión:** 1.5.0

## Estado de publicación

Microsoft Store confirmó publicada Submission 5 (`1152921505702099923`),
con el paquete 1.5.0.0 x64 y el mensaje de actualización disponible. La firma y
las actualizaciones de ese canal las administra Microsoft Store. El autor
autorizó publicar también `v1.5.0` en GitHub con MSI, EXE, DEB, AppImage y RPM.
Los cinco paquetes proceden de GitHub Actions `38191218663`, compilado desde
`0395023f78d8755a152b993131e2b65c3a57671e`; el cierre de versión modifica
únicamente documentación y avisos de licencias, sin cambios en la aplicación.

La sección 1.5.0 de CHANGELOG está cerrada con fecha local 2026-10-11 y existe
una nueva sección `[Sin publicar]` vacía. Las cinco referencias de versión
(package.json, package-lock.json, Cargo.toml, Cargo.lock y tauri.conf.json)
coinciden en 1.5.0. Las ramas auxiliares integradas se retiraron; solo queda main.

## Paquetes y evidencia

- GitHub: `Compilados/GitHub-1.5.0/`, instaladores Windows y Linux descargados
  del run correcto. Notas para usuarios con emojis en `release-notes.md`.
- Store: `Compilados/Microsoft-Store-1.5.0/LF-Botonera-1.5.0.0-x64-unsigned.msix`.
  17.380.834 bytes; SHA-256:
  `8D3F5887B6A897D67AE4BC77700DAF8024EDB08649C0BD49EFF675B1F922D971`.
- Identidad: `LuisFernandoVelasquez.LFBotoneradeEfectos`.
  Publisher: `CN=AD90DE58-447F-47AE-AC1A-3D369955282B`.
  Windows Desktop desde 10.0.19041.0; cuatro idiomas; `runFullTrust` y `microphone`.
- Release Store, MakeAppx y generación de licencias: correctos. Se comprobó la
  ausencia de firma local y la coincidencia del ejecutable y las licencias
  dentro del MSIX. Partner Center validó el paquete y publicó la entrega.
- Última verificación de código: 349 tests Rust aprobados, 19 ignorados,
  cero fallidos; `cargo build --lib` y `npm run build` correctos. El único aviso
  MSVC corresponde a la creación de la biblioteca y el objeto de enlace.
- Los jobs Windows y Linux del run 38191218663 terminaron correctamente.

## Decisiones cerradas de 1.5.0

- Mic ofrece mantener pulsado, alternar y automático, con destinos combinables
  y nivel absoluto (60 → 20 → 60). Rust captura, mide y aplica la atenuación
  temporal conservando faders y master. Mic está entre Solo y Vol; clic derecho
  abre los ajustes. Automático requiere activación explícita; ante fallo de
  entrada restaura el nivel. No mezcla, graba ni transmite el micrófono.
- El medidor de ajustes tiene escucha temporal propia; al cerrar o cambiar a
  manual libera la entrada, salvo si automático sigue activo. Comparte captura
  con automático por nombre real de dispositivo, incluido el predeterminado.
  La resolución ocurre al elegir o abrir la entrada, sin enumeración periódica.
- El panel fijo restaura `visible`; se eliminó `show_on_start` y las dos casillas
  duplicadas. Principal conserva la opción de mostrar su botón superior.
- El volumen del reproductor se controla solo desde el botón de altavoz.
  Guardar otros ajustes ya no escribe ese volumen.
- Hora y fecha permanecen visibles; el contador de sonidos tiene su propio
  destino junto a Detener todo. Rust calcula el tiempo; no hay alternancia
  con el reloj ni temporizador de tres segundos.
- Los atajos opcionales del reproductor son por perfil y respetan `global_keys`.
  Rust valida conflictos y reutiliza su transporte. Nacen vacíos y se exportan
  como campos opcionales.
- Biblioteca y Buscador inician arrastre nativo de rutas con tauri-plugin-drag.
  fileDrop.js es el receptor único. Pestañas y reproductor admiten lotes;
  una celda o botón fijo exige una sola pista.
- Partner Center tiene novedades en los cuatro idiomas y privacidad con Mic
  en español e inglés. La página de privacidad del blog aún debe sincronizarse
  con PRIVACY.md; la política de la ficha Store se proporciona como texto.

## Pendientes

- Pruebas físicas del autor: Mic y captura compartida, atajos locales/globales,
  panel abierto/cerrado tras reiniciar y arrastre entre las dos ventanas.
- Comprobación MIDI en Linux y distintos canales Windows. No lanzar la aplicación
  desde la IA; las pruebas funcionales corresponden al autor.
- Issue #5: esperar confirmación del reportante en Debian/KDE antes de cerrarlo.
- La reorganización visual futura necesita plan y aprobación. No retomar la
  política de colores descartada ni la propuesta del issue #10 por ahora.
- No tocar `Capturas_Tienda/`: material ajeno a esta tarea.
