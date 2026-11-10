# Control remoto LAN — primera entrega, sin publicar

La PC conserva todos los archivos y reproduce el audio. El navegador o el APK
solo envían órdenes y dibujan la paleta activa de la PC. Esta entrega inicia el
control remoto con consulta, disparo, parada individual y Detener efectos.
El control de perfiles, selección de paletas, edición, reproductor, consola,
Biblioteca y OBS quedan para ampliar el control total; no se anuncian como hechos.

## Uso

1. Conectar PC y cliente a la misma red local de confianza.
2. En la PC: Ajustes → Control remoto → elegir la IPv4 y pulsar Activar acceso LAN. Puerto inicial 17865.
3. Abrir la dirección mostrada en un navegador y pegar la clave. En Android:
   Ajustes → Control remoto, introducir la dirección de la PC y la clave.
4. Pulsar los botones para reproducir en la PC. La paleta sigue los cambios locales.
   Debajo aparecen las paradas individuales de efectos activos, incluso si ya
   pertenecen a otra paleta o al panel fijo. Detener efectos usa la parada local.
5. Desactivar acceso LAN en la PC revoca todas las conexiones. Volver a activarlo
   genera otra clave. Cerrar el cliente no detiene los sonidos que ya suenan.

Puede ser necesario permitir el puerto en el cortafuegos de la PC. No se cambia
el cortafuegos automáticamente. La dirección sugerida corresponde a la interfaz
elegida por el sistema; con VPN o varias tarjetas usar la IPv4 de la red compartida.
Si aparece 127.0.0.1, sustituirla por la IPv4 de la PC accesible desde el teléfono.
El usuario puede seleccionar otro puerto si está ocupado. No hay descubrimiento
automático, conexión por Internet ni transporte de audio por la red.

## Arquitectura

- `engine/remote/`: servidor HTTP limitado, recursos embebidos y protocolo cerrado.
- `AppState.remote`: servicio de sesión, desactivado al iniciar, sin cambios al JSON
  de configuración ni a formatos de intercambio. Una clave de 256 bits por sesión.
- `cmd_remote`: `remote_status`, `remote_start(ip, port)`, `remote_stop` y
  `remote_request(address, token, request)`. Pasan por `api.js` en la aplicación.
- `remote_request` conecta desde Rust con `ureq` también en Android: evita cargar
  una página remota con acceso a Tauri y evita depender de CORS o HTTP del WebView.
- `src/remote.html`: entrada web independiente. El servidor sirve exclusivamente
  recursos enumerados, nunca rutas del disco ni el frontend Tauri completo.
- `src/js/remote/`: vista compartida entre navegador y APK. JS usa las APIs DOM,
  fetch en navegador e IPC en APK; ninguna decisión de audio ni tiempo vive en JS.
- Estado por consultas secuenciales cada 400 ms, con timeout y desconexión explícita.
  Ante fallo de conexión los botones de disparo quedan desactivados hasta refrescar.
  Una orden de disparo nunca se reintenta automáticamente.
- `play_remote_button` valida paleta activa y botón sobre la misma configuración
  que usa el disparo local. Paletas y botones OBS no se exponen en esta entrega.
- Las paradas reutilizan el transporte de audio local y respetan los fundidos.
  El motor independiente del reproductor auxiliar no recibe órdenes de parada.
- Todos los textos están en los cuatro idiomas. Colores por `colorAdapter.js`.

## Dependencias y límites

`getrandom 0.3` ya estaba en Cargo.lock como dependencia transitiva y ahora se
declara directamente para obtener claves del generador del sistema operativo.
Proyecto Rust Rand, MIT/Apache-2.0 compatible con GPL-3.0-or-later; no introduce
otro servidor ni otro runtime. Evita implementar un generador criptográfico local.
Las licencias Rust existentes ya incluyen este componente. `ureq` ya era directo.
`httparse` (MIT o Apache-2.0) ya estaba en Cargo.lock y analiza las cabeceras HTTP.

Se acepta únicamente IPv4 privada, loopback o link-local y puertos 1024–65535.
HTTP sin cifrar: usar solo redes de confianza, sin reenvío de puertos a Internet.
Clave enviada por Authorization, nunca en URL, almacenamiento web ni registros.
La dirección y la clave del cliente no se guardan entre sesiones.
Host exige la dirección y puerto de escucha. Si se envía Origin, debe coincidir con el origen del panel. No se habilita
CORS. Órdenes limitadas a snapshot/play/stop/stop_all; no hay puente IPC genérico.
Cabeceras de hasta 8 KiB, cuerpos de hasta 4 KiB, máximo ocho peticiones en curso,
plazo de lectura y escritura, sin chunking, redirecciones ni proxy del sistema.

## Revisión y verificación

Se añadieron tests Rust de protocolo, rechazo de órdenes/rutas arbitrarias,
selección de paleta y límites del parser HTTP. Ejecutar:

```
cd src-tauri
cargo test --lib
cargo build --lib
cd ..
npm run build
```

Craft Anything compiló el frontend. La revisión posterior corrigió la selección
de la interfaz LAN, el análisis y la validación de cabeceras HTTP y la
validación de órdenes y botones. Con MSVC en D: pasaron 398 pruebas Rust
y se omitieron 19. Pasaron también la compilación de la biblioteca Rust,
la del frontend y 10 pruebas JavaScript de traducciones y controles táctiles.
La prueba entre equipos y la generación del APK quedan pendientes.

Prueba funcional del autor: PC y navegador/teléfono en red compartida, clave
correcta/incorrecta, dos clientes, cambios de paleta y perfil desde la PC, sonidos
simultáneos, parada individual/general y fundidos, reproductor independiente,
interrupción y recuperación de Wi-Fi, desactivación/reinicio del servicio y clave
antigua rechazada. Verificar además puerto ocupado y acceso con varias tarjetas.
