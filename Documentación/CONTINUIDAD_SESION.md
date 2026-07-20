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
6. [`MSIX_LOCAL.md`](MSIX_LOCAL.md).
7. [`WACK_MSIX.md`](WACK_MSIX.md).

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
- **Punto de restauración:** commit `c72848a`, `Prepara distribución en tiendas`.
- **Cambios posteriores:** prueba MSIX reproducible y su documentación.

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
- Tauri no genera MSIX directamente. La prueba personalizada ya produce un paquete
  válido para `MakeAppx`; falta firmarlo e instalarlo para la matriz funcional.
- `scripts/build-msix.ps1` generó un MSIX provisional de 8.365.091 bytes, versión
  `1.2.0.0`, con SHA-256
  `E6480763EC1A60E68E4D8752AD169EE56B6951EB0CABC67376E1243A72FA566D`.
- Al extraer el MSIX se recuperaron los 12 archivos previstos y el ejecutable incluido
  conservó exactamente el hash del ejecutable Release original.
- MSIX virtualiza los archivos nuevos de `%APPDATA%`. En Windows 10 2004 o posterior
  debería leer y modificar los archivos ya existentes de la instalación MSI. No se
  añadió `unvirtualizedResources`: su necesidad se decidirá con la prueba instalada.
- La prueba se firmó con un certificado local autorizado y Windows la instaló como
  `LF.Botonera.Efectos.Local_1.2.0.0_x64__b7gt2fsps2vdj`, estado `Ok`.
- El artefacto firmado tiene SHA-256
  `7782803F64DD2642DF51B18999AACB0585F9E3D2E4A20F94163E979392184EA9`.
- La confianza temporal activa está en `LocalMachine\TrustedPeople`, huella
  `618B5F1B2283598D9FC4C6E590531D44ADD5C3BE`. Debe eliminarse al acabar las pruebas.
- El autor aprobó el primer uso real: inicio, cierre, datos existentes, reproducción,
  salida principal y preescucha funcionaron correctamente.
- La comparación comprobó que `botonera_config.json` se leyó y actualizó directamente
  en `%APPDATA%\LF Botonera`; no se duplicaron la configuración ni `tracks.db` en la
  zona virtual. Solo se virtualizó `.window-state.json` de Tauri.
- WACK ejecutó 24 pruebas completas: 22 pasaron, `Blocked executables` falló como prueba
  opcional y DPI produjo una advertencia; resultado global `WARNING`.
- Se añadió un manifiesto Win32 con identidad y `PerMonitorV2`. `mt.exe` confirmó que
  está incrustado, pero WACK continuó indicando que no podía procesar el binario Rust.
- Las cadenas `cmd.exe` proceden de la biblioteca estándar de Rust; otras coincidencias
  son bytes accidentales. `ShellExecute` pertenece al abridor de URLs necesario.
- El permiso Tauri se redujo a apertura de URLs, eliminando el permiso no usado para
  revelar archivos.
- La revisión local actual es `1.2.0.2`, estado `Ok`, SHA-256 firmado
  `D0FF484FCDD0A54BAA93102E3804036E2937C411EAFB6A277CEB582524BE53C7`.
- El autor revalidó la revisión `1.2.0.2`: audio, salida principal, preescucha, URLs e
  interfaz funcionan. También cerró y abrió una segunda vez sin perder el estado.
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
- `MSIX_LOCAL.md`: manifiesto, script reproducible, identidad provisional y límites de
  la prueba sin firma.
- `LICENSE`: texto completo de GPL-3.0, ya existente.
- `README.md`: enlaces visibles a privacidad, soporte, licencia y código fuente.

Las URLs públicas definitivas se obtendrán desde GitHub cuando estos archivos estén
confirmados y publicados. Partner Center puede exigir además datos privados del
titular; esos datos no se guardarán en el repositorio.

---

## 6. Siguiente punto de reanudación

1. Completar la matriz MSIX: archivos externos, arrastre, atajos globales, editor,
   ventana separada, clima y persistencia tras reinicio.
2. Probar actualización, desinstalación y el caso de un usuario sin datos previos.
3. Resolver la cobertura de WebView2 en un Windows 10 limpio.
4. Retirar paquete y certificado locales cuando termine la prueba.
5. Pasar a Partner Center únicamente después de cerrar esa preparación local.

Privacidad, soporte y ficha base fueron aprobados por el autor el 2026-07-20.
Los informes completos de licencias Rust y Node ya se generan con `npm run licenses`.
MSI y NSIS ya incorporan privacidad, soporte y los tres archivos de avisos dentro de
su carpeta `legal/`.
