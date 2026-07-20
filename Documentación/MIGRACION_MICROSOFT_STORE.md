# Migración a Microsoft Store

Estas instrucciones se aplicarán a quienes ya instalaron LF Botonera de Efectos desde
GitHub mediante MSI o EXE y quieran pasar a la versión de Microsoft Store.

## Antes de migrar

1. Cierre LF Botonera de Efectos.
2. Guarde una copia de `%APPDATA%\LF Botonera\`.
3. No mueva ni elimine sus archivos de audio.

La configuración guarda referencias a los archivos de audio originales; no los duplica.

## Orden seguro

1. Instale LF Botonera de Efectos desde Microsoft Store.
2. Abra la entrada nueva desde Inicio.
3. Compruebe perfiles, pestañas, botones, pistas y dispositivos de audio.
4. Cierre la aplicación.
5. Solo después de comprobar los datos, desinstale la versión tradicional anterior.
6. No acepte ninguna opción que elimine datos personales, si llegara a ofrecerse.

Windows puede mostrar temporalmente dos entradas con el mismo nombre. No es una copia
de los perfiles: son dos instalaciones que leen la misma carpeta de datos. El acceso
directo antiguo del escritorio normalmente corresponde a la versión tradicional.

La prueba local de MSIX 1.2.0 confirmó que actualizar, desinstalar y reinstalar el
paquete conserva sin cambios los archivos de `%APPDATA%\LF Botonera\`.

---

# Migrating to Microsoft Store

These instructions apply to users who installed LF Botonera de Efectos from GitHub
using MSI or EXE and want to move to the Microsoft Store version.

## Before migrating

1. Close LF Botonera de Efectos.
2. Back up `%APPDATA%\LF Botonera\`.
3. Do not move or delete your audio files.

The configuration references the original audio files; it does not duplicate them.

## Safe order

1. Install LF Botonera de Efectos from Microsoft Store.
2. Open the new Start menu entry.
3. Check profiles, tabs, buttons, tracks, and audio devices.
4. Close the application.
5. Only after verifying the data, uninstall the previous traditional version.
6. Do not select any option that removes personal data, if one is offered.

Windows may temporarily show two entries with the same name. They are separate
installations reading the same data folder, not duplicated profiles. The old desktop
shortcut normally belongs to the traditional installation.
