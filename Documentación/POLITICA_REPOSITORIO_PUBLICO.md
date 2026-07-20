# Política del repositorio público

Esta política define qué forma parte del código fuente público de LF Botonera de
Efectos, qué se entrega únicamente como archivo de una Release y qué nunca debe llegar
a GitHub. Se aplica antes de subir la rama de distribución, crear un pull request,
fusionar en la rama principal o publicar una versión.

## 1. Principio

El repositorio debe permitir estudiar, compilar, auditar y mantener la aplicación sin
exponer credenciales, datos de usuarios ni residuos de una máquina concreta.

La transparencia técnica es bienvenida. Un identificador público, un hash o el
resultado de una prueba no es un secreto. Una clave privada, un documento de identidad,
una configuración personal o un audio del usuario sí debe excluirse.

## 2. Sí se publica en el repositorio

### Código y compilación reproducible

- `src/` y `src-tauri/src/`, incluidas las pruebas automáticas.
- `package.json`, `package-lock.json`, `Cargo.toml` y `Cargo.lock`.
- configuraciones de Tauri, Vite, capacidades, manifiestos e iconos del producto;
- scripts que generan versiones, licencias y MSIX sin contener claves;
- `.github/`, plantillas de incidencias y flujos de compilación;
- `.gitignore`, `AGENTS.md`, `CLAUDE.md` y ajustes compartidos sin credenciales.

Los archivos de bloqueo son parte del código fuente: fijan las dependencias exactas y
permiten repetir la auditoría de la versión distribuida.

### Documentación y material legal

- `README.md`, `CHANGELOG.md`, `LICENSE`, privacidad, soporte y seguridad;
- avisos y textos completos de licencias de terceros;
- arquitectura, reglas, compilación, contribución, glosario y libro del proyecto;
- planes y registros históricos que expliquen decisiones todavía útiles;
- documentación de MSIX, WACK, migración, distribución y evidencias verificables;
- ficha de publicación, guía de contenidos y esta política.

`CONTINUIDAD_SESION.md`, `CHECKLIST_PREPUBLICACION_LOCAL.md`, `MSIX_LOCAL.md` y
`WACK_MSIX.md` pueden ser públicos. Contienen hashes, una huella de certificado local,
identidad del paquete y número de Submission, pero no claves privadas ni acceso a la
cuenta. Sirven como evidencia del proceso de publicación.

### Capturas

Se publica la biblioteca `Capturas/` con su catálogo. Las 17 imágenes actuales son del
propio programa, usan datos de demostración y no incluyen credenciales.

La captura de Hora y Clima muestra intencionalmente Aragua de Barcelona, Anzoátegui,
Venezuela, ciudad de origen del autor. Está autorizada para GitHub, manual, web, redes
o futuras fichas cuando exista contexto. No se considera un dato filtrado.

Una captura histórica o poco adecuada para publicidad puede permanecer en el código
fuente si el catálogo explica su estado. Estar en GitHub no obliga a usarla como imagen
principal de la aplicación.

## 3. Se publica como archivo de una Release, no dentro de git

- instaladores NSIS y MSI;
- paquetes MSIX, DEB, RPM y AppImage;
- archivos de símbolos o paquetes auxiliares necesarios para distribución;
- hashes y firmas públicas asociados a una versión.

Estos archivos los debe producir el flujo de compilación desde el código etiquetado.
No se guardan junto al código porque aumentan el repositorio, duplican versiones y no
permiten revisar cómo fueron construidos. GitHub Releases es su canal correcto.

## 4. Nunca se publica

### Secretos e identidad privada

- claves privadas, certificados con clave privada, contraseñas y tokens;
- códigos de verificación, sesiones, cookies o archivos de variables de entorno;
- documentos usados para verificar la identidad del titular;
- credenciales de Microsoft Partner Center, GitHub, PayPal u otros servicios.

Los secretos necesarios para automatización se guardan en GitHub Actions Secrets o en
el almacén seguro del servicio correspondiente, nunca en archivos del repositorio.

### Datos locales y del usuario

- `botonera_config.json`, `tracks.db` y estados de ventana de una instalación real;
- pestañas, perfiles y listas exportadas: `.bdelf`, `.bdeplf` y `.LFPlay`;
- audios, carpetas de efectos o locuciones aportados por usuarios;
- rutas que revelen perfiles personales, salvo una captura autorizada conscientemente;
- logs, volcados, cachés, informes temporales y datos de pruebas descartables.

### Construcciones y herramientas locales

- `node_modules/`, `target/`, `dist/` y carpetas de staging o inspección;
- `Compilados/` y ejecutables o instaladores copiados localmente;
- certificados `.pfx`, `.p12`, `.pvk`, `.cer`, `.crt`, `.key` o `.pem`;
- paquetes `.msix`, `.appx`, bundles y archivos de carga de Store;
- certificados de prueba instalados en Windows o su clave privada exportada.

## 5. Identificadores que sí pueden ser públicos

No son secretos y pueden documentarse:

- identificador de Store `9NJ8ST39QP7V`;
- nombre y familia públicos del paquete;
- identificador Flatpak propuesto;
- número de Submission y estados de certificación;
- hashes SHA-256 de artefactos;
- huellas de certificados públicos o locales de prueba;
- nombre real del autor y enlaces de soporte ya destinados a publicación.

Una huella identifica un certificado, pero no permite reconstruir su clave privada.

## 6. Estado de la auditoría del 2026-07-20

- No hay binarios, instaladores, bases de datos ni certificados rastreados por git.
- No se encontraron claves privadas, contraseñas ni tokens literales rastreados.
- `Compilados/`, `node_modules/` y `src-tauri/target/` están ignorados.
- El token usado por GitHub Actions es la referencia segura `${{ github.token }}`,
  no una credencial escrita en el repositorio.
- `.claude/settings.json` solo desactiva atribuciones automáticas y no contiene secretos.
- La biblioteca de 17 capturas está autorizada para formar parte del código fuente.

## 7. Revisión antes de subir

1. Esperar el resultado del primer envío de Microsoft Store.
2. Actualizar continuidad, ficha, changelog y README con el resultado real.
3. Revisar `git status --short` y cada cambio que vaya a entrar en el commit.
4. Confirmar que ningún instalador, certificado, base de datos o exportación esté
   rastreado, aunque `.gitignore` deba impedirlo.
5. Buscar credenciales y rutas personales en los archivos rastreados.
6. Ejecutar pruebas y builds si cambió código; para documentación, comprobar enlaces.
7. Subir primero la rama de trabajo. No etiquetar ni crear una Release hasta decidir
   cómo integrar la versión aprobada y verificar el código fuente correspondiente.

El resultado de Microsoft decide el momento de publicación, no qué archivos son
seguros. Esta política deja preparado el conjunto público sin realizar todavía ningún
push a GitHub.
