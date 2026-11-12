# Primera distribución estable Android

Preparación autorizada por el autor el 2026-11-11. La rama sigue siendo
`codex/desarrollo`; main y la publicación estable de escritorio no se modifican.

## Identidad y versiones

- Producto Android: LF Botonera de Efectos.
- Paquete: `io.github.yosoyluisfernando.lfbotonera`.
- Versión inicial: `1.6.0`; versionCode inicial explícito: `1006000`.
- Fuente de identidad Android: `src-tauri/tauri.android.conf.json`.
- La Beta anterior conserva `...lfbotonera.lanprueba`, su firma de pruebas y
  sus datos. Se puede instalar junto a la estable. No se migran datos solos;
  el autor puede exportar/importar sus perfiles mediante las funciones de la app.
- Escritorio conserva su identidad Beta y su versión de desarrollo. Es una
  excepción deliberada a la sincronización de versiones entre plataformas:
  About, notas y actualizaciones consultan la versión efectiva de Tauri,
  no `CARGO_PKG_VERSION`, para respetar el override Android.
- Incrementar versionCode en cada futura entrega pública Android. Mantener
  paquete y certificado para que las actualizaciones conserven sus datos.

## Firma del autor

Android acepta certificados de aplicación autofirmados. Esta distribución
usa una clave propia del autor, nunca una clave genérica compartida.

El script versionado `scripts/sign-android-release.ps1` utiliza las herramientas
Android/JDK que ya están en D:. La clave privada, su contraseña y el respaldo
no están en Git y no se envían a GitHub Actions.

- Clave local cifrada: `D:/BOTONERA/Claves/Android/produccion/`.
- Respaldo restringido: `D:/BOTONERA/Respaldos/Firma-Android-Estable/`.
- `password.dpapi` permite el uso desde la misma cuenta de Windows.
- `RECUPERACION-PRIVADA.txt` permite recuperar la contraseña en otro equipo;
  contiene un secreto y no debe compartirse con el APK. Guardar esa contraseña
  en un gestor de contraseñas y una copia del PKCS12 fuera de este equipo,
  en ubicaciones separadas. Dos carpetas del mismo disco no sustituyen ese respaldo.
- `certificado-publico.pem` se puede distribuir; no permite firmar aplicaciones.

Para firmar una nueva salida de GitHub sin regenerar la clave:

```powershell
./scripts/sign-android-release.ps1 -Source 'ruta/al/origen.apk' -Destination 'ruta/al/APK-publico.apk'
```

El parámetro `-Initialize` solo crea la primera clave si no existe. El script
rechaza claves incompletas o un respaldo previo que requiera recuperación.

## Compilación y verificación

El workflow `android.yml` conserva las comprobaciones de depuración en cada
push. Su ejecución manual con `build_release=true` añade un APK de producción:

1. Pruebas Node, traducciones y DOM.
2. APK de comprobación y pruebas funcionales en Android 35, incluida navegación
   Atrás nativa, editor y consulta de dispositivos.
3. APK de producción sin depuración; firma efímera de CI para instalarlo en
   un emulador y comprobar dos arranques, identidad, versión y arquitectura.
4. Artefacto `LF-Botonera-Android-ARM64-release`. La clave efímera de CI no
   identifica la distribución pública; se firma localmente con la clave del autor.
5. Verificar la firma final y comparar todos los archivos del ZIP salvo las
   entradas de firma para comprobar que el contenido de aplicación no cambia.
6. Registrar SHA-256, certificado público, commit y ejecución junto al APK final
   en `D:/BOTONERA/Compilados/`. Conservar las entregas anteriores.

La publicación de un release es un paso separado de la compilación. No crear
un tag `v1.6.0` por esta entrega: puede activar la publicación de escritorio.
Un futuro tag Android independiente puede ser `android-v1.6.0`; el comprobador
de versiones reconoce ese formato y exige un APK estable adjunto.

## Canales y actualizaciones

GitHub Releases es el único canal disponible actualmente. Android consulta
el instalador registrado por el sistema para reconocer en el futuro Google
Play (`com.android.vending`) y F-Droid (clientes oficiales). Esos canales
administran sus propias actualizaciones y no consultan GitHub desde la app.

`LF_DISTRIBUTION_CHANNEL=play_store` o `fdroid` permite preparar una compilación
específica; el instalador reconocido tiene prioridad. No usar ese valor para
el APK directo de GitHub. No se anuncia disponibilidad en tiendas inexistentes.

La futura publicación en Google Play requiere su gestión de firma y AAB.
F-Droid tiene su propio proceso de compilación, revisión y firma. Revisar la
continuidad de certificados entre canales antes de anunciar actualizaciones
cruzadas. Esta preparación no registra ni publica la app en ninguna tienda.

GitHub selecciona únicamente releases estables con un archivo para el sistema
actual. Un release Android sin instaladores de escritorio no se ofrece como
actualización de Windows o Linux, ni al revés.

## Notas y alcance

`CHANGELOG.android.md` describe la primera edición pública por sus funciones
y límites. `CHANGELOG.md` conserva las notas e historia de escritorio. Las
versiones estables solo muestran sus notas exactas; las betas pueden mostrar
las notas pendientes de su propia plataforma.

El autor confirmó Launchpad por Bluetooth MIDI desde la app. El teclado
Bluetooth de escritura sigue pendiente de prueba física. Dispositivos cercanos
no activa Bluetooth: el usuario debe encenderlo en Android.

La primera edición advierte de apartados aún en adaptación y posibles
desajustes visuales según teléfono o tablet. OBS local, Mic y otros apartados
de escritorio ausentes se enumeran en las notas Android.

Fuentes: [firma Android](https://developer.android.com/studio/publish/app-signing),
[configuración Tauri](https://v2.tauri.app/reference/config/).

## Primera compilación preparada

Comprobada en [GitHub Actions 43131269897](https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/actions/runs/43131269897),
commit df90c10daae840c7dc528ec0cf650d49508766a3. APK final en
`D:/BOTONERA/Compilados/Android-Estable-1.6.0/`, firmado localmente con el
certificado propio de producción. SHA-256 del APK:
`8127c5c170454368cd0e4394e13a91806cb478ca2f6fbdc91011731b9409bbde`.

Se comprobaron Android 35, Atrás nativo, metadatos Android/GitHub/1.6.0,
contenido idéntico salvo firma y arranque del APK de producción. La compilación
no publica automáticamente el archivo: el release público sigue pendiente.
