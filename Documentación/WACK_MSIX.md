# Resultado de Windows App Certification Kit

Registro técnico de la prueba local de **LF Botonera de Efectos 1.2.0** empaquetada
como MSIX. Los informes XML completos quedan en `src-tauri/target/msix/`, fuera de Git.

## Entorno y resultado

- WACK: 10.0.26100.7705.
- Sistema: Windows 10 Pro 10.0.19045.
- Tipo detectado: Centennial.
- Revisión MSIX local: `1.2.0.2`.
- Ejecución parcial: no.
- Pruebas ejecutadas: 24.
- Aprobadas: 22.
- Resultado global: `WARNING`.

El paquete instalado fue
`LF.Botonera.Efectos.Local_1.2.0.2_x64__b7gt2fsps2vdj`. El MSIX firmado tiene
SHA-256 `D0FF484FCDD0A54BAA93102E3804036E2937C411EAFB6A277CEB582524BE53C7`.

## Prueba opcional Blocked executables

La prueba opcional falló por referencias encontradas dentro de `tauri-app.exe`. Se
auditó cada una:

- `cmd.exe` está dentro de la implementación para archivos por lotes de la biblioteca
  estándar de Rust. La aplicación no invoca esa orden.
- `DNx`, `basH` y `CDb` son coincidencias accidentales en bytes del ejecutable, no
  nombres de programas usados por la aplicación.
- `CreateProcessW` forma parte del soporte general de procesos de Rust.
- `ShellExecuteW` y `ShellExecuteExW` pertenecen a `tauri-plugin-opener`, utilizado
  para abrir enlaces HTTP o HTTPS en el navegador predeterminado.

La aplicación conserva la apertura externa porque los enlaces de soporte,
actualizaciones y donación la necesitan. El permiso Tauri se redujo de
`opener:default` a apertura de URLs; ya no permite revelar archivos en Explorer.

## Advertencia DPI

WACK informó que no pudo procesar el ejecutable y por eso no confirmó DPI. Para
resolver la carencia original se añadió `src-tauri/windows-app-manifest.xml` con:

- identidad Win32 `LF.Botonera.Efectos`;
- `dpiAware=true/pm`;
- `dpiAwareness=PerMonitorV2`.

`mt.exe` extrajo del Release los tres datos y confirmó que están incrustados. WACK
mantuvo la advertencia después de dos reconstrucciones, incluso tras añadir la
identidad de ensamblado. Se clasifica como limitación del analizador sobre este binario
Rust, no como ausencia demostrada de DPI.

No se añadió `requestedExecutionLevel`: Microsoft indica que esa declaración desactiva
virtualización de archivos y Registro. La migración MSIX ya probada depende de no
alterar ese comportamiento sin necesidad.

## Decisión

No se eliminarán funciones reales ni se añadirán parches especulativos para obtener un
informe artificialmente limpio. El candidato final debe repetir WACK, conservar el
informe y presentar esta explicación si la certificación de Store la solicita.

Fuente: [Windows App Certification Kit](https://learn.microsoft.com/en-us/windows/uwp/debug-test-perf/windows-app-certification-kit).
