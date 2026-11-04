# Flatpak de prueba

GitHub Actions empaqueta para x86_64 el `.deb` que ya compiló y publicó el
proyecto. El flujo `Flatpak test package` verifica arquitectura y versión,
instala el resultado en el runner y comprueba el enlace dinámico y los recursos
sin abrir la Botonera. El artefacto `flatpak-x86_64` contiene el instalador y
`SHA256SUMS`. La rama de trabajo `codex/flatpak-test` dispara la primera prueba
con `v1.5.0`. Si se integra el flujo en la rama predeterminada, se puede ejecutar
manualmente para otra etiqueta publicada que incluya el `.deb` amd64.

## Prueba en Linux

Descomprimir el artefacto y ejecutar desde ese directorio:

```sh
sha256sum -c SHA256SUMS
flatpak install --user ./LF-Botonera-1.5.0-x86_64.flatpak
flatpak run io.github.yosoyluisfernando.LF-Botonera-de-efectos//test
```

El paquete usa GNOME Platform 50. La configuración queda en el directorio
privado de Flatpak. Para comparar con una instalación anterior, exportar un
`.lfbackup` y probar su restauración con una copia de la biblioteca.

- Probar efectos, reproductor, Programa, CUE y Mic con el equipo de emisión.
- Probar detección, asignación y reconexión MIDI. El paquete base concede
  acceso al servidor PulseAudio y a gráficos, con acceso directo a ALSA por
  comprobar. Para aislar un problema de permisos durante una prueba, se puede
  añadir `--device=all` a `flatpak run` antes del identificador.
- Probar la Biblioteca y el arrastre. El permiso permanente de Música es de
  solo lectura. Para comprobar escritura de tags, renombrado o una ruta fuera
  de Música, se puede conceder temporalmente `--filesystem=/ruta/de/prueba` a
  `flatpak run` antes del identificador.
- Comparar atajos globales en X11 y Wayland, con la ventana fuera de foco.

## Flathub

La política de Flathub exige la admisión inicial en `stable`. Las aplicaciones
ya admitidas pueden publicar en `beta`, que se instala desde el remoto
Flathub Beta y queda fuera del catálogo web. El formato de prueba de GitHub
sirve para validar la app mientras se mantiene el requisito de evitar un
estreno estable en Flathub.

El envío a Flathub requiere un manifiesto redactado sin asistencia de IA,
un PR presentado por el autor, revisión y una declaración sobre el material
generado por IA que se incluya en la aplicación o el empaquetado. El flujo de
este repositorio es para la prueba directa en GitHub Actions.

- [Requisitos de Flathub](https://docs.flathub.org/docs/for-app-authors/requirements)
- [Mantenimiento de ramas en Flathub](https://docs.flathub.org/docs/for-app-authors/maintenance)
