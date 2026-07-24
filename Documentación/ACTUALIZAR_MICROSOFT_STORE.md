# Actualizar LF Botonera de Efectos en Microsoft Store

Procedimiento permanente para publicar una versión nueva de la aplicación ya
existente en Microsoft Store. No se crea otro producto ni se reserva otro nombre.

## Solicitud de inicio

Cuando el autor decida publicar, basta con indicar:

> Vamos a publicar la versión X.Y.Z en Microsoft Store.

Antes de empezar se define qué cambios forman parte de la versión y se evita añadir
funciones nuevas mientras se prepara el paquete final.

## 1. Preparar la versión local

1. Revisar los cambios y cerrar las notas de `CHANGELOG.md`.
2. Ejecutar `SET-VERSION.bat X.Y.Z` para sincronizar `package.json`,
   `src-tauri/Cargo.toml` y `src-tauri/tauri.conf.json`.
3. Ejecutar `cargo check` en `src-tauri` para actualizar `Cargo.lock`.
4. Regenerar los avisos de licencias con `npm run licenses`.
5. Verificar:

   ```powershell
   cd src-tauri
   cargo test --lib
   cargo build --lib
   cd ..
   npm run build
   ```

6. Realizar la prueba funcional y auditiva con una compilación Release.

## 2. Crear el paquete de Microsoft Store

Desde la raíz del repositorio:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build-store-msix.ps1
```

El script compila el canal `store`, utiliza la identidad oficial ya asignada y crea:

`src-tauri/target/msix/LF-Botonera-X.Y.Z.0-x64-unsigned.msix`

La versión MSIX debe ser superior a la publicada y conservar el cuarto componente en
cero. El archivo destinado a Partner Center se sube **sin la firma del certificado
local**; Microsoft Store lo firma durante su proceso. Antes de subirlo se comprueban
su manifiesto, tamaño, hash SHA-256 y funcionamiento mediante una copia de prueba.

## 3. Enviar la actualización

1. Abrir el producto existente **LF Botonera de Efectos** en Partner Center.
2. Seleccionar **Iniciar actualización** o **Crear una nueva presentación**.
3. Abrir **Paquetes** y subir el nuevo MSIX.
4. Confirmar que Partner Center reconoce la versión, arquitectura, idiomas y
   capacidades esperados.
5. Actualizar la descripción, capturas y notas de la versión solo cuando hayan
   cambiado.
6. Revisar disponibilidad y opciones de publicación.
7. Guardar todas las secciones y seleccionar **Enviar a certificación**.

La presentación anterior sirve como punto de partida. La versión pública existente
continúa disponible durante la certificación; cuando Microsoft aprueba la nueva, esta
la sustituye y Microsoft Store distribuye la actualización a los usuarios.

## 4. Después de la aprobación

1. Comprobar la versión desde la ficha pública y desde una instalación de Store.
2. Registrar el resultado de la certificación y cualquier observación.
3. Crear el commit y la etiqueta `vX.Y.Z`, y publicar la versión en GitHub cuando el
   autor lo autorice.
4. Actualizar README, sitio oficial y otros canales de descarga.
5. Crear una nueva sección `[Sin publicar]` en `CHANGELOG.md`.

## Información que nunca se guarda en GitHub

- contraseñas, códigos de verificación o cookies de sesión;
- documentos de identidad, domicilio, teléfono o información fiscal;
- claves privadas, archivos PFX o contraseñas de certificados;
- capturas de Partner Center que expongan datos personales no públicos;
- cualquier credencial o secreto incorporado accidentalmente en registros.

Esta guía, los nombres públicos del producto, el identificador público de Store y la
identidad incluida en el manifiesto MSIX sí pueden conservarse en el repositorio. Son
datos técnicos públicos y permiten que el proceso sea reproducible. Los artefactos
generados dentro de `src-tauri/target/` no se versionan.

## Referencias

- [Publicar una actualización MSIX](https://learn.microsoft.com/windows/apps/publish/publish-your-app/msix/publish-update-to-your-app-on-store)
- [Cargar paquetes MSIX](https://learn.microsoft.com/windows/apps/publish/publish-your-app/msix/upload-app-packages)
