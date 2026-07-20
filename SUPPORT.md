# Soporte / Support

## Español

### Canales de ayuda

- Para errores reproducibles o solicitudes de funciones, abra una
  [incidencia en GitHub](https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/issues).
- Para conversación con la comunidad, use el
  [grupo de Telegram](https://t.me/+bXppwWvJvSg5YjNh).
- Para avisos del proyecto, consulte el
  [canal de Telegram](https://t.me/+XKof2wDvGVw1YTRh).
- Para vulnerabilidades, siga la [política de seguridad](SECURITY.md). No publique
  detalles sensibles en una incidencia abierta.

El proyecto es mantenido por una persona independiente. No se garantiza atención
inmediata ni un nivel de servicio comercial, pero los informes claros y reproducibles
se revisarán cuando sea posible.

### Sistemas admitidos

- Windows 10 y Windows 11.
- Linux: los paquetes DEB, RPM y AppImage son objetivos de compilación. El soporte
  general permanecerá marcado como experimental hasta completar pruebas físicas en
  distribuciones reales.

La ficha de cada tienda indicará sus requisitos más específicos.

### Antes de informar un problema

1. Instale la versión más reciente del canal que utiliza.
2. Compruebe si el problema ya está informado en GitHub.
3. Pruebe con un archivo de audio que pueda compartir legalmente, si corresponde.
4. No borre su configuración antes de guardar una copia.

Incluya en el informe:

- versión exacta de LF Botonera de Efectos;
- canal de instalación: Microsoft Store, GitHub MSI/EXE, DEB, RPM, AppImage o Flatpak;
- versión y edición del sistema operativo;
- pasos exactos para reproducir el problema;
- resultado esperado y resultado observado;
- formato del audio y características relevantes del dispositivo;
- mensajes de error o logs, revisados para retirar rutas, nombres u otros datos
  personales.

Si el problema es un corte, clic, saturación, ruteo incorrecto o fallo de dispositivo,
indíquelo expresamente aunque las pruebas automáticas pasen. Para audio se evalúa
siempre una compilación Release.

### Datos y copias de seguridad

La configuración actual se encuentra normalmente en:

- Windows: `%APPDATA%\LF Botonera\`
- Linux: `$XDG_CONFIG_HOME/LF Botonera/` o `~/.config/LF Botonera/`

Cierre la aplicación antes de copiar o restaurar esa carpeta. Los archivos de audio no
se copian allí; la configuración guarda referencias a sus ubicaciones originales.

Consulte también la [política de privacidad](PRIVACY.md).

Para cambiar desde un instalador MSI o EXE a Microsoft Store, siga la
[guía de migración](Documentación/MIGRACION_MICROSOFT_STORE.md).

## English

### Support channels

- For reproducible bugs or feature requests, open a
  [GitHub issue](https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/issues).
- For community discussion, use the
  [Telegram group](https://t.me/+bXppwWvJvSg5YjNh).
- For project announcements, see the
  [Telegram channel](https://t.me/+XKof2wDvGVw1YTRh).
- For vulnerabilities, follow the [security policy](SECURITY.md). Do not publish
  sensitive details in an open issue.

The project is maintained by an independent developer. Immediate responses or a
commercial service level are not guaranteed, but clear and reproducible reports will
be reviewed when possible.

### Supported systems

- Windows 10 and Windows 11.
- Linux: DEB, RPM, and AppImage are build targets. General Linux support remains
  experimental until physical testing on real distributions is complete.

Each store listing will state any more specific requirements.

### Before reporting a problem

1. Install the latest version from the channel you use.
2. Check whether the problem is already reported on GitHub.
3. If relevant, test with an audio file you can legally share.
4. Back up your configuration before deleting anything.

Include the application version, installation channel, operating system version,
exact reproduction steps, expected and observed results, relevant audio format or
device details, and any error messages or logs. Remove file paths, names, and other
personal information first.

For clicks, dropouts, clipping, incorrect routing, or device failures, say so even if
automated checks pass. Audio must always be evaluated with a Release build.

### Data and backups

The current configuration is normally stored in:

- Windows: `%APPDATA%\LF Botonera\`
- Linux: `$XDG_CONFIG_HOME/LF Botonera/` or `~/.config/LF Botonera/`

Close the application before copying or restoring that directory. Audio files are not
copied there; the configuration references their original locations.

See also the [privacy policy](PRIVACY.md).

To move from an MSI or EXE installer to Microsoft Store, follow the
[migration guide](Documentación/MIGRACION_MICROSOFT_STORE.md#migrating-to-microsoft-store).
