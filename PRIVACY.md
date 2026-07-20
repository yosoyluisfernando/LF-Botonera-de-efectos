# Política de privacidad / Privacy Policy

**LF Botonera de Efectos**

Versión de esta política: 2026-07-20

Versión web oficial: <https://lfbotonera.blogspot.com/p/privacidad.html>

## Español

### Resumen

LF Botonera de Efectos es una aplicación de escritorio de código abierto. No exige
crear una cuenta, no contiene publicidad y no incorpora telemetría ni herramientas de
seguimiento. El desarrollador no recibe los archivos de audio, configuraciones ni
estadísticas de uso del usuario.

### Datos guardados en el equipo

La aplicación guarda localmente la información necesaria para funcionar, que puede
incluir:

- perfiles, paletas, botones y preferencias;
- rutas de archivos y carpetas de audio elegidos por el usuario;
- nombres de dispositivos de audio y atajos configurados;
- ciudad, coordenadas y unidad elegidas para las locuciones de clima;
- cola y preferencias del reproductor;
- duración, puntos de entrada y salida, ganancia, análisis y fecha de última
  reproducción de las pistas;
- caché de formas de onda y datos necesarios para migrar configuraciones anteriores.

En las instalaciones de escritorio actuales estos datos se encuentran normalmente en:

- Windows: `%APPDATA%\LF Botonera\`
- Linux: `$XDG_CONFIG_HOME/LF Botonera/` o `~/.config/LF Botonera/`

Los archivos de audio permanecen en la ubicación seleccionada por el usuario. La
aplicación no los carga a ningún servidor.

### Conexiones a Internet

La aplicación puede realizar estas conexiones:

1. **GitHub Releases.** Consulta la versión más reciente al iniciar y posteriormente
   con una separación mínima de doce horas. Envía una solicitud técnica con el agente
   `LF-Botonera`; GitHub recibe también información ordinaria de red, como la dirección
   IP. En una edición administrada por una tienda, este mecanismo puede deshabilitarse
   en favor de las actualizaciones de la tienda.
2. **Open-Meteo.** Solo cuando se utiliza el módulo de clima, la búsqueda de ciudades
   envía el texto buscado y la consulta meteorológica envía coordenadas, unidad de
   temperatura e información técnica ordinaria de red. Open-Meteo devuelve ciudad,
   temperatura y humedad.
3. **PayPal.** La página de donación se abre en el navegador únicamente cuando el
   usuario acepta la invitación o pulsa el enlace correspondiente. Desde ese momento
   se aplica la política de privacidad de PayPal.

Estos servicios son independientes y aplican sus propias políticas de privacidad:

- [Privacidad de GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
- [Privacidad de Open-Meteo](https://open-meteo.com/en/terms)
- [Privacidad de PayPal](https://www.paypal.com/webapps/mpp/ua/privacy-full)

### Conservación y eliminación

La aplicación conserva sus datos locales hasta que el usuario los modifica o elimina.
La desinstalación puede no borrar la carpeta de configuración. Para eliminar todos los
datos, cierre la aplicación y elimine manualmente la carpeta indicada arriba. Antes de
hacerlo, conserve cualquier configuración o lista que quiera mantener.

El desarrollador no puede recuperar esos datos porque no recibe una copia.

### Menores y datos sensibles

La aplicación no está dirigida específicamente a menores y no solicita nombres,
correos, pagos, contactos ni datos de salud. Las rutas de archivos y el nombre de la
ciudad pueden ser información personal; permanecen locales salvo las consultas de
clima descritas anteriormente.

### Cambios y contacto

Los cambios se publicarán en la
[versión web oficial](https://lfbotonera.blogspot.com/p/privacidad.html) y en este
archivo, y quedarán registrados en el historial del repositorio. Para consultas de
privacidad, contacte al mantenedor mediante el perfil de
[Yo Soy Luis Fernando en GitHub](https://github.com/yosoyluisfernando). No publique
datos personales o archivos privados en una incidencia pública.

## English

### Summary

LF Botonera de Efectos is an open-source desktop application. It requires no account,
contains no advertising, and includes no telemetry or tracking tools. The developer
does not receive users' audio files, settings, or usage statistics.

### Data stored on the device

The application stores locally the information it needs to operate, which may include:

- profiles, tabs, buttons, and preferences;
- paths to audio files and folders selected by the user;
- audio device names and configured shortcuts;
- the city, coordinates, and unit selected for weather announcements;
- player queue and preferences;
- track duration, cue points, gain, analysis, and last-played time;
- waveform cache and data required to migrate older configurations.

Current desktop installations normally store this data in:

- Windows: `%APPDATA%\LF Botonera\`
- Linux: `$XDG_CONFIG_HOME/LF Botonera/` or `~/.config/LF Botonera/`

Audio files remain in the location selected by the user. The application does not
upload them to any server.

### Internet connections

The application may make the following connections:

1. **GitHub Releases.** It checks the latest version at startup and subsequently with
   a minimum interval of twelve hours. It sends a technical request with the
   `LF-Botonera` user agent; GitHub also receives ordinary network information such as
   the IP address. A store-managed edition may disable this mechanism in favor of
   store updates.
2. **Open-Meteo.** Only when the weather module is used, city search sends the entered
   query, while weather requests send coordinates, temperature unit, and ordinary
   network information. Open-Meteo returns city, temperature, and humidity data.
3. **PayPal.** The donation page opens in the browser only when the user accepts the
   prompt or selects the donation link. PayPal's privacy policy applies from that
   point onward.

These independent services apply their own privacy policies:

- [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
- [Open-Meteo Terms and Privacy](https://open-meteo.com/en/terms)
- [PayPal Privacy Statement](https://www.paypal.com/webapps/mpp/ua/privacy-full)

### Retention and deletion

Local data remains until the user changes or deletes it. Uninstalling the application
may not remove its configuration directory. To remove all data, close the application
and manually delete the directory shown above. Back up any configuration or playlist
you want to keep first.

The developer cannot recover this data because no copy is received.

### Children and sensitive data

The application is not specifically directed at children and does not request names,
email addresses, payment details, contacts, or health data. File paths and the selected
city may be personal information; they remain local except for the weather requests
described above.

### Changes and contact

Changes will be published on the
[official web version](https://lfbotonera.blogspot.com/p/privacidad.html) and in this
file, and recorded in the repository history. For privacy questions, contact the
maintainer through the
[Yo Soy Luis Fernando GitHub profile](https://github.com/yosoyluisfernando). Do not
post personal information or private files in a public issue.
