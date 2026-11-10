# MIDI Bluetooth LE en escritorio

Windows y Linux ofrecen **Buscar por Bluetooth** en Ajustes → Atajos del
teclado → MIDI → Añadir dispositivo. Solo se listan controladores que anuncian
el servicio BLE MIDI. Seleccionar el dispositivo abre su característica GATT
de notificaciones y los mensajes alimentan la captura y el despacho MIDI
existentes. Los puertos WinMM y ALSA siguen disponibles en la misma lista.

La búsqueda dura seis segundos. Se inicia al pulsar el botón y también al
arrancar si MIDI está habilitado y ya hay un dispositivo BLE guardado. La
selección usa su dirección Bluetooth y se vuelve a intentar la conexión tras
una desconexión. Un dispositivo que cambia de dirección puede requerir una
nueva búsqueda y selección.

El adaptador compartido usa btleplug 0.11.8 (MIT o Apache-2.0) sobre WinRT en
Windows y BlueZ en Linux. BlueZ debe estar en ejecución y el usuario debe
tener acceso al servicio Bluetooth del sistema. Para compilar en Linux se
necesitan los encabezados D-Bus (libdbus-1-dev en Debian y Ubuntu), además de
los requisitos de escritorio existentes. El soporte se limita a BLE MIDI 1.0.

El decodificador quita las marcas temporales del transporte antes de pasar
los mensajes al analizador común. El disparo es inmediato al recibir el
paquete. Windows y Linux comparten pruebas de marcas temporales, running
status, mensajes truncados y SysEx intercalado.
