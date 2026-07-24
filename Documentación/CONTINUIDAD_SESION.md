# Continuidad de sesión — buscador interno y distribución

Este documento es el punto de entrada para retomar el trabajo después de completar la
publicación de **LF Botonera de Efectos 1.2.1** en Microsoft Store y fusionar la rama
de distribución con `main`.

No usar aquí planes históricos de funciones ya completadas.

## 0. Etapa activa — buscador interno

- **Rama:** `codex/buscador-interno`.
- **Base inicial:** `771adf7`, `Integra distribución en tiendas y prepara la etapa
  Linux (#6)`.
- **Inicio:** 2026-07-24.
- **Estado:** planificación y documentación; no hay código funcional autorizado.
- **Documento rector:** [`PLAN_BUSCADOR_INTERNO.md`](PLAN_BUSCADOR_INTERNO.md).

El objetivo es añadir al panel fijo una búsqueda rápida sobre una o varias carpetas
de audio elegidas por el usuario. Debe admitir más de 100.000 canciones además de
efectos, búsqueda difusa, índice persistente y actualización incremental sin bloquear
la interfaz ni los motores de audio.

La propuesta pendiente de aprobación es una tercera presentación
`fixed_panel.view = "search"` con un motor Rust propio integrado en el `tracks.db`
existente. No se creará otra base que duplique rutas, tamaño, `mtime` o duración.
Primero se descubrirán las rutas para hacerlas buscables y después se enriquecerán en
segundo plano con duración y etiquetas. LUFS, onda y PCM seguirán siendo análisis
explícitos y no parte de la catalogación.

Se auditó `C:\LF Automatizador v1.0` como referencia, excluyendo completamente
`C:\LF Automatizador v1.0\LF Automatizador 2.0`. Los hallazgos útiles y los límites que
no deben copiarse quedaron registrados en `PLAN_BUSCADOR_INTERNO.md`.

Antes de escribir código:

1. Leer completo `PLAN_BUSCADOR_INTERNO.md`.
2. Acordar con el autor la experiencia de uso, especialmente Enter y doble clic.
3. Aprobar presupuestos de rendimiento y corpus de errores de escritura.
4. Ejecutar una prueba aislada de ranking e índice con 100.000 y 250.000 filas, y
   medir `lofty` con duración y etiquetas en archivos reales.
5. Aprobar dependencias, esquema SQLite, módulos, comandos IPC y eventos.
6. Solo después actualizar las reglas y documentos arquitectónicos definitivos.

La prueba física en Linux y Flathub siguen pendientes, pero quedan pausados mientras
se diseña esta actualización. La documentación histórica de distribución permanece
vigente más abajo.

## 1. Lectura inicial obligatoria

Antes de proponer o modificar código:

1. Leer `AGENTS.md` en la raíz del repositorio.
2. Leer [`REGLAS_PROYECTO.md`](REGLAS_PROYECTO.md).
3. Leer este documento completo.
4. Para la etapa activa, leer
   [`PLAN_BUSCADOR_INTERNO.md`](PLAN_BUSCADOR_INTERNO.md).
5. Para retomar Linux, leer
   [`PLAN_DISTRIBUCION_TIENDAS.md`](PLAN_DISTRIBUCION_TIENDAS.md) y
   [`CHECKLIST_PREPUBLICACION_LOCAL.md`](CHECKLIST_PREPUBLICACION_LOCAL.md).
6. Auditar el código y la configuración actuales antes de decidir arquitectura.

Los documentos `MSIX_LOCAL.md`, `WACK_MSIX.md` y
`ACTUALIZAR_MICROSOFT_STORE.md` son referencia histórica o para futuras
actualizaciones de Microsoft Store. No forman parte de la lectura inicial de Linux.

---

## 2. Estado actual

- **Proyecto:** LF Botonera de Efectos.
- **Versión en Microsoft Store:** 1.2.1.
- **Última versión pública en GitHub Releases:** 1.2.0. La publicación 1.2.1 en
  GitHub sigue pendiente hasta que el autor decida unificar los canales.
- **Stack:** Tauri v2, backend Rust, frontend Vanilla JS con Vite.
- **Licencia:** GPL-3.0-or-later.
- **Repositorio local:** `C:\OVERLAY\BOTONERA`.
- **Rama de trabajo:** `codex/buscador-interno`.
- **Base de la rama:** `771adf7`,
  `Integra distribución en tiendas y prepara la etapa Linux (#6)`.
- **Identificador técnico común:**
  `io.github.yosoyluisfernando.LF-Botonera-de-efectos`.
- **Prioridad activa:** planificar el buscador interno sin tocar código funcional.
- **Prioridad pausada:** prueba física en Linux y después Flathub.
- **Primer destino previsto:** Flathub, después de una prueba física real en Linux.
- **Destinos posteriores:** evaluar repositorios oficiales de Debian, Fedora u otras
  distribuciones únicamente cuando el paquete y su mantenimiento sean sostenibles.
- **Cambios de distribución:** ya fusionados en `main` mediante el PR #6.
- **Última verificación local:** 209 pruebas Rust aprobadas, 4 pruebas manuales
  ignoradas, `cargo build --lib`, `npm run build` y aplicación Windows Release
  completados correctamente.

Antes de editar, ejecutar `git status` y conservar cualquier cambio ajeno a la tarea.
No limpiar, descartar ni sobrescribir el árbol de trabajo.

---

## 3. Microsoft Store — hito completamente cerrado

No hay trabajo pendiente de publicación en Microsoft Store para esta etapa.

- La aplicación está aprobada y disponible públicamente:
  `https://apps.microsoft.com/detail/9NJ8ST39QP7V`.
- El identificador de Store es `9NJ8ST39QP7V`.
- La versión pública es 1.2.1.
- La actualización de metadatos
  `1152921505701463690` fue aprobada.
- La ficha pública ya muestra la descripción definitiva, los enlaces directos al
  sitio oficial y al soporte, y las siete capturas de pantalla acordadas.
- La primera publicación y la actualización posterior fueron verificadas por el
  autor.
- El canal `store` deshabilita la consulta de actualizaciones en GitHub y deja que
  Microsoft Store administre las actualizaciones.
- La rutina de futuras versiones está documentada en
  [`ACTUALIZAR_MICROSOFT_STORE.md`](ACTUALIZAR_MICROSOFT_STORE.md).

La próxima sesión no debe entrar en Partner Center, modificar la ficha ni reconstruir
el MSIX, salvo que el autor abra expresamente una nueva tarea de Microsoft Store.

---

## 4. Decisión cerrada: cada canal administra sus actualizaciones

La experiencia de Microsoft Store deja una regla obligatoria para Linux:
**una instalación nunca debe quedar administrada a la vez por GitHub y por una
tienda o repositorio**.

Comportamiento requerido:

- **Microsoft Store:** Microsoft Store busca, descarga e instala actualizaciones. El
  comprobador de GitHub permanece deshabilitado.
- **Descarga directa desde GitHub Releases:** puede conservar el comprobador de
  GitHub y dirigir al usuario al canal directo correspondiente.
- **Flathub o Flatpak administrado por un repositorio:** Flatpak administra las
  actualizaciones. La aplicación no debe ofrecer ni iniciar actualizaciones desde
  GitHub.
- **Paquete instalado desde un repositorio DEB o RPM:** APT, DNF o el gestor propio
  de la distribución administra las actualizaciones. La aplicación no debe ofrecer
  ni iniciar actualizaciones desde GitHub.
- **DEB, RPM o AppImage descargado directamente desde GitHub:** debe tratarse como
  canal directo, salvo que la auditoría técnica demuestre que un formato necesita
  otra política explícita.

El canal de distribución debe quedar definido de forma explícita y reproducible
durante la compilación o el empaquetado. No se debe depender únicamente de
heurísticas frágiles en tiempo de ejecución ni mantener dos fuentes de verdad.

Para cada futuro canal administrado de Linux, antes de programar:

1. Auditar el pipeline real del destino y quién administra sus actualizaciones.
2. Revisar todos los puntos de interfaz y backend afectados por ese canal.
3. Extender la arquitectura existente sin crear otra fuente de verdad.
4. Explicar qué se decide en compilación, qué metadatos recibe cada paquete y cómo se
   prueba que dos gestores de actualización no puedan mezclarse.
5. Si la solución cambia estructura, flujo IPC o arquitectura, presentar el plan y
   esperar aprobación antes de implementarla.

Los nombres definitivos de Flatpak, APT, DNF u otros canales todavía no están
cerrados. Deben decidirse al diseñar cada pipeline, no inventarse por adelantado.

### 4.1 Estado real de la implementación

La base `direct`/`store` ya está implementada y no debe volver a duplicarse:

- `src-tauri/src/domain/distribution.rs` decide canal, plataforma y administrador.
- `get_distribution_info` entrega esos datos a «Acerca de».
- `cmd_updates.rs` consulta la misma fuente antes de acceder a GitHub.
- `scripts/build-store-msix.ps1` fija `LF_DISTRIBUTION_CHANNEL=store`.
- Los builds actuales de GitHub para Windows y Linux son `direct`; la ausencia de la
  variable conserva ese valor por compatibilidad.

Todavía **no** existen canales administrados para Flatpak, APT o DNF. Se añadirán a
la misma fuente cuando se prepare cada pipeline, nunca mediante detección de rutas,
extensiones o archivos instalados.

Decisión fundamental: plataforma, formato y canal son ejes independientes. Un DEB
de GitHub es directo; un futuro DEB de un repositorio APT será administrado. Linux no
significa automáticamente Flathub y Windows no significa automáticamente Store.

### 4.2 Cómo mejorar Linux sin dañar Windows

- Investigar primero si la causa es común o realmente exclusiva de Linux.
- Mantener la lógica de negocio y audio común siempre que sea posible.
- Aislar código nativo con `cfg(target_os)` y una API común solo cuando haya evidencia.
- No decidir comportamiento de plataforma en JavaScript.
- Ejecutar la matriz común y ambos jobs CI después de cada cambio.
- Probar audio y funciones del sistema en una compilación Release física.

La política completa y sus límites están en
[`ARCHITECTURE.md`](ARCHITECTURE.md#política-para-cambios-específicos-de-windows-o-linux);
los comandos y marcas de canal están en
[`COMPILACION_Y_VERSIONES.md`](COMPILACION_Y_VERSIONES.md#21-plataforma-formato-y-canal-no-son-lo-mismo).

---

## 5. Orden de trabajo para Linux

No iniciar Flathub, Debian y Fedora al mismo tiempo. Trabajar en etapas pequeñas y
verificables:

1. **Completado:** auditar soporte Linux, paquetes Tauri, dependencias, rutas,
   permisos y actualizador.
2. **Completado para los canales actuales:** implementar una fuente común para
   `direct` y `store`. Los canales administrados de Linux se añaden al preparar cada
   pipeline.
3. Verificar la compilación local común:
   - `cargo test --lib`;
   - `cargo build --lib`;
   - `npm run build`.
4. Preparar una compilación **Release** en Linux y generar los formatos que realmente
   soporte el proyecto. No evaluar audio con una compilación debug.
5. Probar físicamente en Linux antes de anunciar soporte estable:
   - inicio, cierre y persistencia;
   - reproducción de efectos y reproductor auxiliar;
   - salida principal, preescucha y selección de dispositivos;
   - archivos y carpetas externos;
   - arrastre, diálogos y rutas con espacios o caracteres no ASCII;
   - editor de pistas, onda, cue y normalización;
   - clima y acceso de red;
   - atajos locales y globales en X11 y Wayland;
   - importación y exportación de `.bdelf`, `.bdeplf` y `.LFPlay`;
   - PipeWire, PulseAudio o ALSA según el sistema de prueba.
6. Corregir las causas reales encontradas y repetir la matriz Release.
7. Preparar los metadatos Linux reutilizables: archivo `.desktop`, AppStream,
   iconos, capturas, licencia, privacidad, soporte y descripción en los idiomas que
   exija el destino.
8. Preparar Flatpak para Flathub:
   - compilación desde fuentes y sin descargar dependencias durante el build;
   - permisos mínimos y portales cuando correspondan;
   - acceso correcto a audio y archivos elegidos por el usuario;
   - identificador y metadatos coherentes;
   - actualización administrada exclusivamente por Flatpak.
9. Validar el manifiesto y el paquete, realizar prueba física del Flatpak y solo
   entonces iniciar la solicitud a Flathub.
10. Después de Flathub, evaluar por separado los requisitos de repositorios oficiales
    de Debian y Fedora, incluyendo disponibilidad de dependencias, políticas de
    empaquetado, firma, mantenimiento y canal de actualizaciones.

---

## 6. Evidencia y decisiones reutilizables

- El código está planteado como multiplataforma y Tauri tiene previstos paquetes
  DEB, RPM y AppImage, pero esto no equivale a soporte Linux verificado.
- Las rutas de datos usan la abstracción del proyecto y SQLite se integra con la
  aplicación; aun así, deben comprobarse en un sistema Linux real.
- El acceso a archivos absolutos, el audio, los dispositivos y los atajos globales
  son los riesgos principales dentro del sandbox de Flatpak.
- El proyecto es software libre bajo GPL-3.0-or-later y ya dispone de política de
  privacidad, soporte, avisos de terceros, capturas y sitio oficial.
- GitHub Releases seguirá siendo un canal válido para descargas directas. Publicar en
  Flathub o en un repositorio oficial no debe eliminar ese canal; debe separarlo.
- No hay telemetría ni publicidad. Las conexiones conocidas son GitHub Releases para
  el canal directo, Open-Meteo cuando el usuario usa el clima y PayPal únicamente
  mediante una acción consciente del usuario.
- Las pruebas automáticas no sustituyen la prueba física ni auditiva. El agente
  realiza compilaciones, tests y auditoría técnica; Luis Fernando valida la experiencia
  real y el audio.
- Si Luis Fernando oye cortes, clics, saturación o un ruteo incorrecto aunque los
  tests pasen, la prueba se considera fallida y se investiga la causa.

---

## 7. Primer punto de reanudación

La auditoría inicial y la base `direct`/`store` ya están completadas. Una sesión nueva
debe:

1. Confirmar rama, versión y árbol de trabajo; conservar todos los cambios locales.
2. No volver a Partner Center ni reconstruir MSIX salvo petición expresa del autor.
3. Revisar que los cambios de canal y preescucha continúan presentes y verificados.
4. Preparar la prueba física Release en Linux. La primera distribución recomendada
   es Ubuntu Desktop 26.04 LTS; comenzar con DEB y AppImage.
5. Registrar distribución, versión, Wayland/X11, PipeWire/PulseAudio/ALSA, formato
   instalado, una o dos tarjetas y resultados de la matriz funcional.
6. Corregir cada hallazgo en el núcleo común o en un adaptador Linux según su causa,
   repitiendo siempre la verificación Windows.
7. Solo después de esa prueba diseñar el canal administrado `flatpak` y los permisos
   de sandbox. Si exige cambios estructurales o IPC, presentar el plan al autor.

El objetivo inmediato no es publicar, sino obtener un paquete Linux Release
reproducible y físicamente probado sin alterar el comportamiento estable de Windows.
