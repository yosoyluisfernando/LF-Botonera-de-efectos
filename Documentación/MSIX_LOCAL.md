# Empaquetado MSIX local

Este documento describe la prueba de concepto MSIX de LF Botonera de Efectos. El
paquete generado es técnico y local: no se envía a Microsoft ni se considera un
artefacto publicable hasta incorporar la identidad asignada por Partner Center.

## Decisiones

- La ruta principal será MSIX porque Microsoft Store vuelve a firmar el paquete. Esto
  evita exigir a un desarrollador individual un certificado comercial solo para Store.
- El ejecutable se declara como aplicación de escritorio clásica, nivel `mediumIL`,
  con la capacidad restringida `runFullTrust`. Es necesaria para conservar audio,
  archivos, diálogos, ventanas y atajos de una aplicación Win32 normal.
- El identificador Tauri
  `io.github.yosoyluisfernando.LF-Botonera-de-efectos` sigue siendo la identidad
  técnica multiplataforma. No se sustituye por el `Package/Identity/Name` de Store:
  Microsoft asignará ese segundo valor después de reservar el producto.
- El manifiesto local usa `LF.Botonera.Efectos.Local` y
  `CN=Luis Fernando Velasquez` solo para validar la estructura. No son identidades de
  producción ni deben copiarse a Partner Center.
- El paquete exige Windows 10 2004, compilación 19041, o posterior. Esto permite usar
  la declaración moderna `packagedClassicApp` con confianza `mediumIL`.

## Generación

Desde la raíz del repositorio:

```powershell
npm run licenses
npm run tauri -- build --no-bundle
powershell -ExecutionPolicy Bypass -File scripts/build-msix.ps1
```

El resultado queda fuera de Git, en:

`src-tauri/target/msix/LF-Botonera-<versión>-x64-unsigned.msix`

El script toma la versión de `tauri.conf.json`, la convierte de `A.B.C` a `A.B.C.0`,
incluye el ejecutable Release, los tres recursos visuales exigidos y los documentos
legales. `MakeAppx.exe` valida el manifiesto y produce el hash SHA-256.

## Lo que no hace el script

- No crea certificados ni instala certificados de prueba.
- No firma el MSIX.
- No registra ni instala el paquete.
- No reserva un nombre ni consulta datos privados de Partner Center.
- No descarga ni incluye WebView2 Fixed Version.

Un MSIX debe estar firmado para instalarse normalmente. La firma local requeriría
crear un certificado y confiar en él en Windows; esa modificación del almacén de
confianza solo se hará con aprobación explícita.

## Prueba firmada e instalada

Con autorización del autor se creó un certificado de desarrollo no exportable y se
firmó el paquete local. Estos valores pertenecen solo al equipo de pruebas:

- sujeto: `CN=Luis Fernando Velasquez`;
- huella SHA-1: `618B5F1B2283598D9FC4C6E590531D44ADD5C3BE`;
- vigencia: 2026-07-20 a 2027-07-20;
- almacén de firma: `CurrentUser\My`;
- confianza exigida por App Installer: `LocalMachine\TrustedPeople`.

El artefacto firmado es
`src-tauri/target/msix/LF-Botonera-1.2.0-x64-local-signed.msix`, mide 8.368.758 bytes
y tiene SHA-256
`7782803F64DD2642DF51B18999AACB0585F9E3D2E4A20F94163E979392184EA9`.
`SignTool` lo verificó sin advertencias ni errores.

Windows lo instaló con estos datos:

- paquete: `LF.Botonera.Efectos.Local_1.2.0.0_x64__b7gt2fsps2vdj`;
- familia: `LF.Botonera.Efectos.Local_b7gt2fsps2vdj`;
- aplicación: `LFBotonera`;
- AUMID: `LF.Botonera.Efectos.Local_b7gt2fsps2vdj!LFBotonera`;
- tipo de firma: `Developer`;
- estado: `Ok`.

La pestaña de propiedades de Windows mostró el firmante `Luis Fernando Velasquez`,
algoritmo `sha256` y ausencia de marca de tiempo. Esto es correcto para la prueba:
el certificado es temporal, no se distribuye, y Microsoft Store sustituirá la firma.

### Primera prueba funcional

El autor confirmó el 2026-07-20 que funcionaron correctamente:

- inicio y cierre del MSIX exacto;
- carga de perfiles, pestañas y botones existentes;
- reproducción de audio;
- salida principal y preescucha.
- apertura de enlaces web con el permiso reducido;
- presentación y tamaño normal de la interfaz con el manifiesto DPI nuevo;
- cierre accidental y segundo inicio correcto, conservando el estado.

La comparación posterior demostró que el paquete usó el almacenamiento histórico:

- `%APPDATA%\LF Botonera\botonera_config.json` pasó de 31.280 a 31.336 bytes y su
  fecha de modificación coincidió con el cierre de la prueba;
- `tracks.db` permaneció intacto;
- no apareció una copia virtual de ninguno de esos dos archivos;
- MSIX solo virtualizó `.window-state.json`, estado interno de posición y tamaño de
  ventana administrado por Tauri.

Esto verifica la migración de una instalación existente. Aún falta comprobar el ciclo
de datos de un usuario nuevo y qué ocurre al desinstalar el paquete.

### Windows App Certification Kit

WACK ejecutó las 24 pruebas aplicables. El diagnóstico completo, la procedencia de las
dos observaciones y la evidencia DPI están en [`WACK_MSIX.md`](WACK_MSIX.md).

El certificado de prueba no sirve para producción. Microsoft Store reemplazará esta
firma por una de confianza pública. Al terminar todas las pruebas hay que desinstalar
el paquete y retirar la huella anterior de `LocalMachine\TrustedPeople` mediante una
consola elevada. También debe eliminarse de `CurrentUser\My` para borrar su clave
privada. Ninguna clave o PFX se guardó en el repositorio.

## WebView2

La compilación usa WebView2 Evergreen instalado en Windows. Windows 11 lo incluye,
pero Microsoft advierte que algunos equipos Windows 10 pueden no tenerlo. Un MSIX no
debe ejecutar un instalador auxiliar desde el paquete.

Por eso la prueba queda dividida en dos puertas:

1. validar ahora el paquete y su ejecución en el equipo de desarrollo, donde ya está
   WebView2;
2. antes del envío, probar un equipo o máquina virtual Windows 10 limpio y decidir
   entre la provisión admitida por Store o WebView2 Fixed Version incluido junto a la
   aplicación. La segunda opción aumenta mucho el tamaño y obliga a mantener ese
runtime actualizado, así que no se adoptará sin necesidad comprobada.

## Datos existentes y virtualización

La aplicación usa `%APPDATA%\LF Botonera` como fuente única para
`botonera_config.json`, `tracks.db` y la caché de ondas. MSIX virtualiza las nuevas
escrituras en AppData para una aplicación `packagedClassicApp`.

En la versión mínima elegida, Windows 10 2004, la lectura busca primero en la zona
privada del paquete y luego en el AppData normal. Si un archivo ya existe fuera del
paquete, Windows permite modificarlo allí. Esto debería hacer visible una instalación
MSI anterior, pero debe comprobarse con una copia controlada de datos reales.

Los archivos creados por una instalación nueva de Store quedan asociados al paquete
y pueden eliminarse al desinstalarlo. Por ahora no se declara
`unvirtualizedResources`: es otra capacidad restringida y ampliaría permisos antes de
demostrar que sea imprescindible. La prueba de instalación decidirá si basta con el
comportamiento normal, una migración explícita o una exclusión limitada.

## Identidad definitiva

Después de reservar `LF Botonera de Efectos`, Partner Center mostrará al menos:

- `Package/Identity/Name`;
- `Package/Identity/Publisher`;
- `Package/Properties/PublisherDisplayName`.

Los tres valores deben pasarse exactamente al script:

```powershell
./scripts/build-msix.ps1 `
  -IdentityName '<Package/Identity/Name>' `
  -Publisher '<Package/Identity/Publisher>' `
  -PublisherDisplayName '<PublisherDisplayName>'
```

El nombre público y el identificador de Store no deben deducirse ni abreviarse.

## Verificación pendiente con paquete instalado

Cuando exista una firma de prueba o un paquete firmado por Store, se comprobarán:

- inicio y cierre normal;
- perfiles, paletas y migración de datos existentes;
- selección, arrastre y reproducción de archivos externos;
- salida principal, preescucha y cambio de dispositivos;
- atajos globales;
- editor modal y ventana independiente;
- clima, enlaces externos y ausencia del actualizador de GitHub en el canal Store;
- actualización, desinstalación y conservación de datos del usuario.

Fuentes oficiales:

- [Elegir una ruta de distribución](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/choose-distribution-path)
- [Generar componentes MSIX](https://learn.microsoft.com/es-es/windows/msix/desktop/desktop-to-uwp-manual-conversion)
- [Declaraciones de capacidades](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/app-capability-declarations)
- [Distribución de WebView2](https://learn.microsoft.com/es-es/microsoft-edge/webview2/concepts/distribution)
- [Identidad asignada por Store](https://learn.microsoft.com/en-us/windows/apps/publish/view-app-identity-details)
