# Plan de distribución en tiendas y repositorios

Documento rector para publicar **LF Botonera de Efectos** fuera de GitHub Releases.
Microsoft Store quedó completada con la versión 1.2.1. La prioridad activa es la
prueba física en Linux y, después, Flathub.

**Estado del documento:** guía inicial basada en la auditoría de la versión 1.2.0.
**Rama de trabajo:** `codex/distribucion-tiendas`.
**Fecha de inicio:** 2026-07-20.
**Última actualización:** 2026-07-23.

Las reglas vigentes de aislamiento entre Windows y Linux están en
[`ARCHITECTURE.md`](ARCHITECTURE.md#política-para-cambios-específicos-de-windows-o-linux).
La marca de canal y los comandos de cada build están en
[`COMPILACION_Y_VERSIONES.md`](COMPILACION_Y_VERSIONES.md#21-plataforma-formato-y-canal-no-son-lo-mismo).

---

## 1. Decisión de alcance

El orden acordado es:

1. Publicar primero en Microsoft Store.
2. Dejar preparada desde ahora la base común de distribución.
3. Probar físicamente la aplicación en Linux antes de prometer soporte en una tienda.
4. Intentar Flathub como primer catálogo general de Linux.
5. Evaluar después Debian y Fedora oficiales; Ubuntu normalmente se beneficia de la
   entrada previa en Debian.
6. Considerar Snap Store y openSUSE después, sin confundir repositorios comunitarios
   con repositorios oficiales de una distribución.

GitHub Releases seguirá siendo un canal válido para MSI, NSIS, DEB, RPM y AppImage.
Una tienda complementa ese canal; no lo reemplaza automáticamente.

---

## 2. Estado comprobado de la versión 1.2.0

### Lo que ya está bien encaminado

- El proyecto declara `GPL-3.0-or-later` en `package.json` y `Cargo.toml`.
- La versión coincide en `package.json`, `Cargo.toml` y `tauri.conf.json`.
- Existe un identificador estable y verificable mediante GitHub:
  `io.github.yosoyluisfernando.LF-Botonera-de-efectos`.
- Tauri genera instaladores MSI y NSIS en Windows, y está configurado para generar
  DEB, RPM y AppImage en Linux.
- Existen iconos de aplicación para las plataformas actuales.
- La interfaz tiene traducciones en español, inglés, portugués de Brasil y portugués
  de Portugal.
- En la auditoría de esta misma revisión pasaron 206 pruebas de Rust; 4 pruebas de
  hardware o servicios reales quedaron ignoradas deliberadamente. También pasaron
  `cargo build --lib` y `npm run build`.
- `npm audit` no informó vulnerabilidades conocidas.

### Hallazgos que impiden considerar lista la publicación

> **Registro histórico de la auditoría 1.2.0.** Estos puntos explican las decisiones
> posteriores; no significan que Microsoft Store continúe pendiente.

- El MSI actual no está firmado digitalmente.
- El instalador actual puede descargar WebView2 durante la instalación. Microsoft
  exige que los instaladores MSI o EXE enviados a Store sean autónomos, no descarguen
  componentes y admitan instalación silenciosa.
- Tauri genera MSI y EXE, pero no genera directamente un MSIX listo para Store.
- Falta una política de privacidad pública y estable.
- El paquete debe ofrecer de forma clara la licencia GPL, el código fuente
  correspondiente, la ausencia de garantía y los avisos de licencias de terceros.
- El actualizador consulta GitHub al iniciar. Una instalación administrada por Store
  no debe invitar al usuario a saltarse la actualización de Store.
- La descripción de `Cargo.toml` y la descripción larga de `tauri.conf.json` contienen
  texto mal codificado. Debe corregirse antes de generar metadatos públicos.
- Falta validar en el entorno de Store el acceso a archivos de audio, dispositivos de
  sonido, atajos globales, ventanas auxiliares y persistencia de datos.

El MSI 1.2.0 auditado se generó correctamente en:

`src-tauri/target/release/bundle/msi/LF Botonera de Efectos_1.2.0_x64_en-US.msi`

El MSI saneado tiene SHA-256
`A66E6F51C65CD7939388184A51855026D8B578A562C80FD76302B11857C5208C`.
Los hashes de MSI, NSIS y ejecutable se registran en
`CHECKLIST_PREPUBLICACION_LOCAL.md`; cualquier nueva compilación tendrá que calcular y
registrar los suyos.

---

## 3. Microsoft Store: ruta principal

> **Estado histórico:** completada. La versión 1.2.1 y la actualización de su ficha
> fueron aprobadas. Esta sección conserva las razones y el procedimiento que llevaron
> a la publicación; no define trabajo activo.

### 3.1 Cuenta y titular de la publicación

La cuenta de desarrollador de Microsoft puede abrirse como individuo. Microsoft usa
verificación de identidad y actualmente no cobra la inscripción en los mercados que
admiten el nuevo proceso. Antes de terminar el registro hay que contestar con datos
reales si la actividad es personal o si existe una relación comercial o profesional.
No se inventará una empresa ni una firma de programador.

La GPL no exige ser una empresa y no impide vender o distribuir el programa. La cuenta
de Store, la identidad del publicador y la licencia del código son asuntos distintos.

Fuente oficial: [Apertura de una cuenta de desarrollador](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account).

### 3.2 Reservar la identidad del producto

Una vez validada la cuenta:

1. Reservar el nombre público `LF Botonera de Efectos` si está disponible.
2. Guardar exactamente los valores de identidad que entregue Partner Center:
   nombre del paquete, nombre del publicador, identificador de Store y familia del
   paquete si corresponde.
3. No cambiar el identificador Tauri ni el `upgradeCode` del MSI por intuición. Primero
   se documentará cómo se relacionan con la identidad asignada por Microsoft.

La reserva del nombre es una acción externa y se hará con confirmación del autor.

### 3.3 Prueba de concepto de empaquetado

Se evaluarán dos rutas, en este orden:

**Ruta A — MSIX personalizado.** Es la primera prueba porque Microsoft firma el MSIX
distribuido por Store y eso puede evitar que un desarrollador independiente compre un
certificado solo para este canal. Tauri no produce MSIX de forma nativa, por lo que
antes de adoptarlo hay que demostrar que el paquete se puede generar de manera
repetible y que conserva todas las funciones de escritorio.

La prueba debe comprobar:

- inclusión o provisión válida de WebView2;
- lectura de archivos y carpetas de audio elegidos por el usuario;
- arrastrar y soltar archivos y carpetas;
- persistencia y migración de `botonera_config.json` y `tracks.db`;
- salida principal, preescucha y selección de dispositivos;
- atajos globales;
- editor de pistas, ventana separada y diálogos nativos;
- red para clima y consulta de versión;
- instalación, actualización y desinstalación sin perder datos del usuario.

**Ruta B — MSI o EXE no empaquetado.** Es el respaldo oficialmente documentado por
Tauri. Requiere que el instalador sea autónomo, admita modo silencioso y esté firmado
con un certificado aceptado. Para el MSI se verificará `msiexec /i paquete.msi /quiet`.
Esta ruta tiene un coste de certificado y por eso no es la primera elección.

Microsoft documenta los requisitos para MSI/EXE en
[Requisitos de paquetes de aplicaciones](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/app-package-requirements),
las opciones de firma en
[Firma de código para aplicaciones Windows](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)
y Tauri resume su flujo en
[Distribuir en Microsoft Store](https://v2.tauri.app/distribute/microsoft-store/).

### 3.4 WebView2 sin descargas durante la instalación

El instalador enviado a Store no puede ejecutar una descarga auxiliar. Si se usa MSI
o EXE, se configurará el modo sin conexión de WebView2 y se medirá el aumento real del
tamaño. Si se usa MSIX, se verificará cuál es la dependencia admitida por el paquete y
por el sistema objetivo.

La referencia técnica es
[Distribución de WebView2 Runtime](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution).

### 3.5 Adaptaciones de la aplicación

Se implementó una única noción de canal de distribución en Rust, definida al
compilar. Actualmente distingue `direct` y `store`. El frontend solo muestra el
estado recibido desde Rust.

En el canal Microsoft Store:

- Store será responsable de las actualizaciones;
- no se mostrará una descarga de GitHub como actualización principal;
- los enlaces de soporte, privacidad, código fuente y licencia serán estables;
- la identidad y las rutas de datos se comprobarán antes de decidir una migración.

Esta modificación afecta arquitectura y flujo; se diseñará y aprobará antes de tocar
el código, de acuerdo con las reglas del proyecto.

### 3.6 Material legal y ficha pública

Hay que preparar:

- política de privacidad en una URL pública y permanente;
- página de soporte con un medio real de contacto;
- enlace al repositorio y al código fuente de la versión distribuida;
- texto completo de GPL-3.0-or-later y aviso de ausencia de garantía;
- inventario y avisos de licencias de dependencias de Rust, Node y recursos incluidos;
- descripción corta y larga en los idiomas que se publiquen;
- categoría, palabras clave, requisitos y notas de la versión;
- iconos y capturas legibles, sin datos personales no autorizados ni material ajeno.

La aplicación usa Internet para Open-Meteo, la comprobación de versiones en GitHub y
el enlace de donaciones. No se encontró telemetría en la auditoría, pero la política de
privacidad debe explicar con precisión esos accesos y cualquier dato guardado
localmente. La declaración final se contrastará otra vez con el código.

Microsoft explica los datos de soporte en
[Información de soporte](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/support-info),
la ficha en
[Información de la descripción de Store](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info)
y los recursos gráficos en
[Capturas e imágenes](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images).

### 3.7 Certificación y entrega

Cada candidato de publicación tendrá:

1. versión sincronizada y changelog cerrado;
2. compilación Release limpia y reproducible en lo posible;
3. hashes SHA-256 de todos los artefactos;
4. pruebas Rust y build frontend aprobados;
5. Windows App Certification Kit aprobado cuando aplique;
6. instalación, actualización, desinstalación y persistencia probadas en una cuenta
   limpia de Windows;
7. prueba auditiva real del autor;
8. revisión de las políticas vigentes de Microsoft el día del envío.

Referencias: [Políticas de Microsoft Store](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies)
y [Windows App Certification Kit](https://learn.microsoft.com/en-us/windows/uwp/debug-test-perf/windows-app-certification-kit).

---

## 4. Base común para cualquier plataforma

Este trabajo sí conviene hacerlo durante la fase Microsoft porque después se reutiliza:

- una identidad estable de aplicación y publicador;
- versiones y notas de publicación sincronizadas;
- política de privacidad y página de soporte;
- licencia GPL incluida en los paquetes;
- enlace verificable al código fuente de cada versión;
- avisos de terceros generados desde las dependencias resueltas;
- inventario de componentes y hashes de artefactos;
- textos de ficha traducidos;
- iconos, capturas y material promocional de origen propio;
- metadatos AppStream para Linux;
- definición explícita del canal de actualización;
- automatización de compilación separada de la publicación.

No se guardarán secretos, certificados ni credenciales de tienda en el repositorio.

---

## 5. Linux: preparación y orden recomendado

### 5.1 Puerta obligatoria: prueba física

Antes de solicitar entrada en un catálogo Linux se probarán, en una máquina Linux real,
la compilación Release y los paquetes DEB y AppImage. Hay que verificar audio,
preescucha, dispositivos, atajos globales tanto en X11 como en Wayland cuando sea
posible, diálogos, rutas, arrastrar y soltar, red, editor y persistencia.

Tauri recomienda construir AppImage sobre la distribución más antigua que se quiera
soportar; Ubuntu 22.04 o Debian 12 son bases de referencia razonables en su guía:
[Distribución AppImage](https://v2.tauri.app/distribute/appimage/).

### 5.2 Flathub: primer objetivo Linux

Flathub será el primer objetivo por alcance, pero se describirá correctamente como un
catálogo general de aplicaciones Linux, no como el repositorio oficial de Debian,
Ubuntu o Fedora.

La publicación exige compilar completamente desde fuentes y sin red durante el build,
un identificador coherente, metadatos AppStream válidos, texto en inglés y recursos de
calidad. Referencias:

- [Requisitos para autores de Flathub](https://docs.flathub.org/docs/for-app-authors/requirements)
- [Guía de metadatos AppStream](https://docs.flathub.org/docs/for-app-authors/metainfo-guidelines)
- [Guía de calidad de la ficha](https://docs.flathub.org/docs/for-app-authors/metainfo-guidelines/quality-guidelines)
- [Manifiestos de Flatpak](https://docs.flatpak.org/en/latest/manifests.html)
- [Permisos del sandbox](https://docs.flatpak.org/en/latest/sandbox-permissions.html)

Hay un riesgo técnico que debe resolverse con una prueba de concepto: la aplicación
guarda rutas absolutas, explora carpetas, usa audio profesional y atajos globales.
Flatpak bloquea por defecto la mayoría del sistema anfitrión. Se intentarán portales y
permisos mínimos; no se prometerá compatibilidad hasta probar archivos persistentes,
PulseAudio/PipeWire, ALSA cuando corresponda y atajos en Wayland.

### 5.3 Debian y Ubuntu oficiales

Debian exige un paquete fuente conforme a sus políticas, revisión por un patrocinador
y que las dependencias de compilación estén disponibles de forma aceptable en el
archivo. El proyecto resuelve cientos de paquetes Cargo, además del frontend Node/Vite;
por eso la primera tarea no será escribir `debian/`, sino producir un informe de
disponibilidad de dependencias. Empaquetar cada crate ausente podría ser un proyecto
considerable.

Si resulta viable, el proceso será:

1. comprobar si ya existe una solicitud en WNPP;
2. presentar un ITP;
3. preparar el paquete fuente y pasar `lintian`, `sbuild` y `autopkgtest`;
4. publicar en mentors.debian.net y solicitar patrocinio;
5. atender revisión, NEW queue y mantenimiento posterior.

Ubuntu recomienda que los paquetes nuevos entren primero por Debian y luego se
sincronicen. Una entrada directa en Ubuntu requiere el proceso de revisión de MOTU.

Referencias:

- [Guía para nuevos mantenedores de Debian](https://www.debian.org/doc/manuals/maint-guide/first.en.html)
- [Debian Mentors](https://mentors.debian.net/intro-maintainers/)
- [Política Debian](https://www.debian.org/doc/debian-policy/)
- [Política de empaquetado Rust de Debian](https://wiki.debian.org/Teams/RustPackaging/Policy)
- [Paquetes nuevos en Ubuntu](https://wiki.ubuntu.com/UbuntuDevelopment/NewPackages)

### 5.4 Fedora oficial

Fedora también exige revisión, patrocinio para nuevos empaquetadores y dependencias de
compilación disponibles en repositorios Fedora. Solo se preparará un archivo SPEC
después del mismo estudio de crates y dependencias Node.

Referencias: [Mantenedores de paquetes Fedora](https://docs.fedoraproject.org/en-US/package-maintainers/)
y [Guías de empaquetado Fedora](https://docs.fedoraproject.org/en-US/packaging-guidelines/).

### 5.5 Snap Store y openSUSE

Snap Store es una opción secundaria. La aplicación puede necesitar interfaces amplias
o confinamiento clásico por archivos, audio y atajos; el confinamiento clásico exige
aprobación. Primero se intentará resolver Flathub, donde el aprendizaje es reutilizable.

openSUSE Build Service puede generar paquetes para varias distribuciones. Un proyecto
personal de OBS es un canal comunitario; la entrada en Factory requiere enviar el
paquete mediante el proyecto de desarrollo correspondiente y superar revisión.

Referencias:

- [Registrar una aplicación en Snap Store](https://snapcraft.io/docs/registering-your-app-name/)
- [Publicar una aplicación Snap](https://snapcraft.io/docs/releasing-your-app/)
- [openSUSE Build Service](https://en.opensuse.org/Portal:Build_Service)

AUR, PPA, COPR y repositorios personales de OBS pueden ser útiles, pero no se
presentarán como repositorios oficiales de la distribución.

---

## 6. Fases y criterios de salida

### Fase 0 — Verificación y saneamiento local

- [x] Crear rama dedicada.
- [x] Auditar el estado de Windows, licencias, red, empaquetado y herramientas.
- [x] Definir el orden Microsoft Store → prueba Linux → Flathub → distribuciones.
- [x] Corregir metadatos dañados y documentación de versión.
- [x] Crear un checklist local con estado y evidencia.
- [x] Verificar pruebas Rust, build Rust, build frontend, i18n y auditoría npm.
- [x] Repetir las verificaciones después de cerrar los cambios de esta fase.

### Fase 1 — Preparación legal y pública

- [x] Preparar privacidad y soporte en español e inglés.
- [x] Confirmar GPL y el enlace al código fuente.
- [x] Crear el inventario inicial de licencias de terceros.
- [x] Generar los textos completos de licencias Rust y Node de forma reproducible.
- [x] Revisar y aprobar con el autor privacidad, soporte y ficha base.
- [x] Redactar la ficha base en español e inglés.
- [x] Revisar la ficha con el autor y preparar los recursos gráficos elegidos.

### Fase 2 — Preparación técnica de Windows

- [x] Resolver WebView2 mediante la ruta MSIX aceptada por Microsoft Store.
- [x] Instalar o preparar las herramientas MSIX necesarias.
- [x] Generar una prueba MSIX repetible con una identidad provisional local.
- [x] Ejecutar la matriz funcional que no dependa de Partner Center.
- [x] Documentar qué valores deberán sustituirse con la identidad de Store.

### Fase 3 — Cuenta e identidad Microsoft

- [x] Completar la verificación del titular real.
- [x] Reservar el nombre del producto.
- [x] Registrar en este documento la identidad asignada, sin secretos.
- [x] Confirmar mercado, precio, visibilidad y público objetivo.

### Fase 4 — Paquete definitivo y adaptación

- [x] Generar el paquete sin firma con la identidad asignada por Store.
- [x] Completar la matriz funcional del MSIX definitivo.
- [x] Documentar los resultados y conservar MSIX como ruta aprobada.
- [x] Aprobar una sola ruta de publicación.
- [x] Implementar el canal de distribución `store`.
- [x] Incluir licencia, privacidad, soporte y avisos finales.

### Fase 5 — Certificación y envío Microsoft

- [x] Crear y verificar el artefacto Release final.
- [x] Pasar certificación, instalación, actualización y prueba auditiva.
- [x] Completar la ficha de Partner Center y enviar.
- [x] Registrar aprobación y actualización de la ficha pública.

### Fase 6 — Linux físico y Flathub

- [ ] Probar DEB y AppImage en Linux real.
- [ ] Crear y validar metadatos AppStream.
- [ ] Construir Flatpak sin red desde fuentes.
- [ ] Resolver permisos mínimos y probar audio, archivos y atajos.
- [ ] Enviar a Flathub y mantener el manifiesto.

### Fase 7 — Repositorios oficiales de distribuciones

- [ ] Medir disponibilidad de todas las dependencias en Debian y Fedora.
- [ ] Decidir con evidencia si el mantenimiento es sostenible para una persona.
- [ ] Iniciar Debian/Ubuntu o Fedora solo si supera esa puerta.
- [ ] Evaluar openSUSE Factory, Snap u otros canales después.

---

## 7. Próximo paso concreto

El trabajo inmediato es la **Fase 6**:

1. instalar Ubuntu Desktop 26.04 LTS en una máquina física;
2. probar primero DEB y AppImage Release del canal directo;
3. registrar audio, dispositivos, Wayland/X11, archivos, atajos, red y persistencia;
4. corregir hallazgos sin romper la matriz Windows;
5. diseñar Flatpak y su canal administrado únicamente después de esa evidencia.

Debian, Fedora y otros repositorios oficiales permanecen fuera de alcance hasta
completar Flathub y evaluar por separado el coste real de mantenimiento.
