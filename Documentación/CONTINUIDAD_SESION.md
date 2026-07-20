# Continuidad de sesión — distribución en tiendas

Este documento contiene únicamente el estado del trabajo iniciado el 2026-07-20
para publicar **LF Botonera de Efectos 1.2.0** en Microsoft Store y preparar su
distribución en Linux.

No usar aquí planes históricos de funciones ya terminadas. Para retomar esta tarea,
leer solamente:

1. [`REGLAS_PROYECTO.md`](REGLAS_PROYECTO.md).
2. Este documento.
3. [`PLAN_DISTRIBUCION_TIENDAS.md`](PLAN_DISTRIBUCION_TIENDAS.md).
4. [`CHECKLIST_PREPUBLICACION_LOCAL.md`](CHECKLIST_PREPUBLICACION_LOCAL.md).
5. [`FICHA_PUBLICACION.md`](FICHA_PUBLICACION.md).

---

## 1. Estado actual

- **Rama:** `codex/distribucion-tiendas`.
- **Punto de partida:** commit `00cbc17`, `Release 1.2.0`.
- **Prioridad:** Microsoft Store.
- **Linux:** preparar ahora lo reutilizable; probar físicamente antes de solicitar
  entrada en Flathub o en repositorios oficiales.
- **Cuenta Microsoft:** todavía no se crea. Primero se termina la preparación local.
- **Identificador técnico:**
  `io.github.yosoyluisfernando.LF-Botonera-de-efectos`; se eligió para que corresponda
  al repositorio verificable y pueda reutilizarse en Tauri y Flatpak.
- **Cambios sin commit:** documentación de distribución y correcciones de metadatos.

---

## 2. Orden de trabajo acordado

1. Ejecutar y documentar todas las verificaciones que puedan hacerse localmente.
2. Corregir textos dañados, versiones antiguas y metadatos inconsistentes.
3. Preparar privacidad, soporte, licencia, código fuente y avisos de terceros.
4. Preparar textos y recursos que se reutilizarán en Microsoft Store y Linux.
5. Definir y probar el paquete de Windows sin publicar nada todavía.
6. Crear y verificar la cuenta individual de Microsoft.
7. Reservar el nombre y registrar la identidad asignada por Partner Center.
8. Certificar y enviar a Microsoft Store.
9. Probar la aplicación físicamente en Linux.
10. Preparar Flathub y, después, evaluar repositorios oficiales de distribuciones.

No abrir varios procesos de publicación a la vez.

---

## 3. Evidencia ya comprobada

- La versión es 1.2.0 en `package.json`, `Cargo.toml` y `tauri.conf.json`.
- `cargo test --lib`: 206 pruebas aprobadas y 4 ignoradas deliberadamente.
- `cargo build --lib`: correcto.
- `npm run build`: correcto.
- `npm run tauri build`: correcto; genera MSI y NSIS Release.
- Los cuatro archivos i18n tienen 427 claves.
- `npm audit`: ninguna vulnerabilidad conocida.
- El MSI 1.2.0 se genera, pero no está firmado y su instalación actual puede
  descargar WebView2. No es todavía un candidato válido para Store.
- Tauri no genera MSIX directamente. Hay que construir una prueba de concepto antes
  de elegir definitivamente MSIX o MSI/EXE firmado.
- No se encontró telemetría ni publicidad en el código.
- La aplicación consulta GitHub Releases y, cuando se habilita el clima, Open-Meteo.
  PayPal solo se abre si el usuario acepta o pulsa el enlace de donación.

---

## 4. Correcciones locales aplicadas

Se corrigieron los textos dañados encontrados durante la auditoría:

- descripción y autor en `src-tauri/Cargo.toml`;
- descripción larga en `src-tauri/tauri.conf.json`;
- dos textos de respaldo en `src/index.html`;
- un mensaje de cancelación en `src-tauri/src/ipc/cmd_locutions.rs`.

`Âmbito`, en portugués de Portugal, es una palabra correcta y no debe modificarse.

---

También se declaró explícitamente el publicador `Luis Fernando Velásquez`, el sitio,
copyright, licencia y categoría. El MSI regenerado ya muestra el fabricante correcto
e incorpora la página de licencia GPL.

## 5. Documentación preparada

- `PRIVACY.md`: política de privacidad en español e inglés.
- `SUPPORT.md`: soporte, reporte de fallos y canales de contacto.
- `THIRD_PARTY_NOTICES.md`: inventario y avisos de dependencias incluidos.
- `CHECKLIST_PREPUBLICACION_LOCAL.md`: verificaciones locales y evidencia.
- `FICHA_PUBLICACION.md`: textos comunes y decisiones pendientes de la ficha.
- `PLAN_DISTRIBUCION_TIENDAS.md`: estrategia, fases y decisiones.
- `LICENSE`: texto completo de GPL-3.0, ya existente.
- `README.md`: enlaces visibles a privacidad, soporte, licencia y código fuente.

Las URLs públicas definitivas se obtendrán desde GitHub cuando estos archivos estén
confirmados y publicados. Partner Center puede exigir además datos privados del
titular; esos datos no se guardarán en el repositorio.

---

## 6. Siguiente punto de reanudación

1. Diseñar la prueba MSIX local y resolver WebView2 sin crear todavía la cuenta.
2. Pasar a Partner Center únicamente después de cerrar esa preparación local.

Privacidad, soporte y ficha base fueron aprobados por el autor el 2026-07-20.
Los informes completos de licencias Rust y Node ya se generan con `npm run licenses`.
MSI y NSIS ya incorporan privacidad, soporte y los tres archivos de avisos dentro de
su carpeta `legal/`.
