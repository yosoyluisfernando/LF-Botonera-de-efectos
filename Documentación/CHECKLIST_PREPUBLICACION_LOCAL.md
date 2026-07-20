# Checklist local previo a crear cuentas de tienda

Esta lista reúne todo lo que debe quedar comprobado en el equipo de desarrollo antes
de crear la cuenta de Microsoft o iniciar solicitudes en catálogos Linux.

**Versión auditada:** 1.2.0.

**Rama:** `codex/distribucion-tiendas`.
**Inicio:** 2026-07-20.

---

## 1. Identidad y metadatos

- [x] Versión 1.2.0 sincronizada en `package.json`, `Cargo.toml` y
  `tauri.conf.json`.
- [x] Identificador Tauri y Flatpak común:
  `io.github.yosoyluisfernando.LF-Botonera-de-efectos`.
- [x] `upgradeCode` MSI estable y documentado.
- [x] Autor real declarado: Luis Fernando Velásquez.
- [x] Declarar explícitamente publicador, sitio, copyright, licencia y categoría para
  impedir que Tauri deduzca `luis-fernando` como fabricante.
- [x] Corregir mojibake en autor y descripciones públicas.
- [x] Corregir mojibake encontrado en textos de interfaz y mensajes Rust.
- [x] Preparar descripción corta, descripción larga y palabras clave en español e
  inglés en `FICHA_PUBLICACION.md`.
- [x] Textos de ficha revisados y aprobados por el autor el 2026-07-20.
- [ ] Definir la identidad asignada por Microsoft. Este punto espera la cuenta y la
  reserva del nombre.

## 2. Documentación pública y legal

- [x] Licencia GPL-3.0-or-later completa en `LICENSE`.
- [x] Configurar `licenseFile` y comprobar que Tauri incorpora la GPL en las páginas
  de licencia de MSI y NSIS.
- [x] Política de privacidad bilingüe en `PRIVACY.md`.
- [x] Página bilingüe de soporte en `SUPPORT.md`.
- [x] Inventario inicial de licencias en `THIRD_PARTY_NOTICES.md`.
- [x] Enlace público al código fuente y a los releases.
- [x] Generar en modo offline y estricto los textos Rust con `cargo-about` 0.9.1.
- [x] Generar el inventario y los avisos Node con un script local sin dependencias.
- [x] Configurar privacidad, soporte y avisos como recursos de los paquetes Tauri.
- [x] Confirmar en las tablas MSI y el script NSIS que se instalan los cinco recursos
  legales dentro de `legal/`.
- [x] Privacidad y soporte revisados y aprobados por el autor el 2026-07-20.
- [ ] Obtener las URLs públicas definitivas después de integrar estos archivos en la
  rama publicada.

## 3. Privacidad y conexiones

- [x] Confirmar que no existe telemetría ni publicidad.
- [x] Documentar la consulta de actualizaciones a GitHub Releases.
- [x] Documentar búsquedas y consultas de clima a Open-Meteo.
- [x] Documentar que PayPal se abre solo por acción o aceptación del usuario.
- [x] Documentar archivos, rutas y metadatos guardados localmente.
- [ ] Diseñar el canal `microsoft-store` para que Store gestione las actualizaciones.
- [ ] Volver a auditar conexiones justo antes de cada publicación.

## 4. Calidad del repositorio

- [x] `cargo test --lib`: 206 aprobadas, 4 ignoradas.
- [x] `cargo build --lib`: correcto.
- [x] `npm run build`: correcto.
- [x] `npm audit`: 0 vulnerabilidades conocidas.
- [x] Los cuatro idiomas tienen 427 claves.
- [x] `git diff --check`: sin errores.
- [x] Repetir estas verificaciones después del saneamiento documental y de metadatos.
- [x] `npm run tauri build`: compilación Release completa y generación de MSI y NSIS.
- [ ] Repetir todas las verificaciones después de los futuros cambios de empaquetado
  MSIX o del canal Microsoft Store.
- [ ] Ejecutar las cuatro pruebas ignoradas con audio, hardware y red reales cuando
  corresponda.
- [ ] Hacer prueba auditiva usando exclusivamente una compilación Release.

## 5. Windows local

- [x] Generar MSI 1.2.0.
- [x] Confirmar nombre del producto y versión dentro del MSI.
- [x] Registrar los SHA-256 de los paquetes generados con los metadatos saneados:
  - MSI: `A66E6F51C65CD7939388184A51855026D8B578A562C80FD76302B11857C5208C`
  - NSIS: `BD35E87960FE822670DD0441B9486EF8CD4C8E2541C3A0F60F3BC69444F5DFD9`
  - Ejecutable: `F4B5FC53551CD44C1F8437B566F2FEF8BA1DB6E7D0FED2224EB0B1E0DFE4480C`
- [x] Confirmar que el MSI y el ejecutable actuales no están firmados.
- [x] Confirmar dentro del MSI: producto `LF Botonera de Efectos`, versión `1.2.0`,
  fabricante `Luis Fernando Velásquez`, idioma de instalador 1033 e instalación por
  equipo (`ALLUSERS=1`).
- [x] Confirmar que el cambio de identificador conserva el `upgradeCode` MSI
  `{43888972-C5A4-5E8D-A996-CA913F3B6D8E}`.
- [x] Detectar que el instalador actual puede descargar WebView2 y por ello no es
  aceptable todavía para Store como MSI/EXE.
- [x] Confirmar disponibilidad local de Windows App Certification Kit, `makeappx` y
  `signtool` mediante Windows SDK.
- [x] Confirmar que MSIX Packaging Tool no está instalado actualmente.
- [ ] Definir el contenido autónomo de WebView2 y medir el tamaño resultante.
- [ ] Generar una prueba MSIX repetible.
- [ ] Probar archivos, carpetas, arrastrar y soltar, audio, preescucha, atajos globales,
  ventanas, red y persistencia dentro de MSIX.
- [ ] Comparar MSIX con MSI/EXE autónomo y firmado.
- [ ] Aprobar una sola ruta de publicación.
- [ ] Pasar Windows App Certification Kit al candidato final.
- [ ] Probar instalación silenciosa si se elige MSI o EXE.
- [ ] Probar instalación, actualización, desinstalación y conservación de datos en una
  cuenta limpia de Windows.
- [ ] Escanear exactamente el artefacto final descargado desde el canal publicado.

## 6. Linux local

- [x] Configuración Tauri presente para DEB, RPM y AppImage.
- [x] AppImage configurado con `bundleMediaFramework: true`.
- [ ] Compilar Release en una base Linux suficientemente antigua.
- [ ] Probar DEB y AppImage en una máquina Linux real.
- [ ] Probar PipeWire/PulseAudio, dispositivos, preescucha y cambio en caliente.
- [ ] Probar atajos globales en X11 y Wayland.
- [ ] Probar diálogos, carpetas, arrastrar y soltar, editor, red y persistencia.
- [ ] Crear y validar `metainfo.xml`, archivo `.desktop` e iconos.
- [ ] Construir Flatpak completamente desde fuentes y sin red durante el build.
- [ ] Definir permisos mínimos del sandbox y probar rutas persistentes.
- [ ] Medir disponibilidad de crates y dependencias Node en Debian y Fedora antes de
  prometer entrada en sus repositorios oficiales.

## 7. Puerta para crear la cuenta Microsoft

Se pasa a la cuenta solamente cuando:

- privacidad y soporte hayan sido aprobados por el autor;
- los textos públicos básicos estén preparados;
- los errores de codificación estén corregidos;
- el repositorio compile y las pruebas automáticas estén verdes;
- se haya decidido qué datos del titular pueden mostrarse públicamente;
- estén separadas las tareas que dependen de la identidad asignada por Partner Center.

No es necesario terminar MSIX antes de crear la cuenta, porque la identidad de Store
es una entrada del paquete. Sí es necesario llegar a la cuenta con la base documental y
el repositorio saneados.
