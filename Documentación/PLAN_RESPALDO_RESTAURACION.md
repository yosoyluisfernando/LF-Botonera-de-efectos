# Respaldo y restauración

Documento rector del respaldo completo de LF Botonera. Decisiones aprobadas por el
autor el 2026-07-29.

## Objetivo

Permitir que una persona conserve y restaure como una sola unidad los dos estados
esenciales de la aplicación:

- `botonera_config.json`: preferencias, perfiles, paletas, botones y reproductor;
- `tracks.db`: catálogo, raíces, cue, ganancia, normalización y datos técnicos.

Los audios y `waveforms/` no forman parte del respaldo. La interfaz debe decirlo
explícitamente tanto antes como después de crear el archivo.

## Formato `.lfbackup`

Un respaldo es una base SQLite autocontenida con extensión `.lfbackup`. Parte de una
copia online consistente de `tracks.db` y añade la tabla reservada
`lf_backup_manifest`.

El manifiesto guarda versión del formato y esquema SQLite; identificador, fecha,
versión de la aplicación y plataforma; cantidades de perfiles, paletas y pistas; y la
serialización completa de `AppConfig`.

No se añade un contenedor ZIP ni una segunda dependencia. Se habilita únicamente la
característica `backup` de `rusqlite`, ya usado por el proyecto. SQLite comprueba el
archivo con `PRAGMA integrity_check`.

## Creación

1. El usuario elige destino y nombre con el diálogo nativo.
2. El historial pendiente se vuelca a SQLite.
3. La configuración se toma de la fuente de verdad en RAM.
4. SQLite Online Backup copia una instantánea consistente, incluso en modo WAL.
5. Se añade el manifiesto y se comprueban configuración, cantidades e integridad.
6. Se sincroniza el archivo temporal con el disco.
7. Solo entonces se renombra al nombre definitivo.
8. El programa vuelve a abrirlo y muestra el resumen verificado.

Un destino ya existente no se sobrescribe. Así un error de permisos, espacio o
sincronización nunca destruye un respaldo anterior.

## Restauración

La selección inicial es solo de lectura. Antes de ofrecer la confirmación se validan
el formato, el esquema, el JSON, las cantidades y la integridad SQLite.

Tras confirmar:

1. el archivo elegido se copia y verifica dentro del directorio de datos;
2. se crea y verifica un `.lfbackup` de emergencia del estado actual;
3. se escribe atómicamente `restore-pending.json`;
4. Tauri reinicia la aplicación;
5. `lib::run()` resuelve la transacción antes de construir `AppState`;
6. se retiran los sidecars WAL/SHM cerrados y se instalan base y configuración;
7. ambos estados se vuelven a abrir y verificar;
8. se registra el resultado para mostrarlo en el siguiente arranque.

La presencia del marcador vuelve idempotente el proceso. Una interrupción entre
renombres repite la instalación en el siguiente inicio. Si el archivo preparado
falla, se instala el respaldo de emergencia y se conserva un resultado de error.

Los respaldos de emergencia viven en `automatic-backups` dentro del directorio de
datos. No contienen audio.

## Interfaz

La Biblioteca muestra el botón `Archivo y restauración` con el texto `Respaldo`,
inmediatamente antes del selector de etiquetas. La barra principal usa
`Carpeta con lupa` para abrir la Biblioteca.

El modal ofrece `Crear respaldo` y `Restaurar respaldo`. La confirmación destaca que
la aplicación se reiniciará. Los cuatro idiomas usan redacción natural y el diseño
permite varias líneas sin truncamiento.

El resumen aparece al crear, inspeccionar y completar una restauración. Incluye fecha,
versión, perfiles, paletas, pistas, tamaño, integridad, audios no incluidos y ruta.

## IPC y separación de responsabilidades

Los comandos son `backup_create`, `backup_choose_restore`,
`backup_prepare_restore`, `backup_restart` y `backup_take_restore_result`.

Rust decide formato, validación, copia, transacción y recuperación. JavaScript
presenta estado, abre el modal y envía órdenes.

## Pruebas

La suite automática cubre ida y vuelta, destino existente, corrupción, manifiesto
incongruente, esquema futuro, filas comprometidas todavía en WAL, intercambio
interrumpido y rollback mediante el respaldo de emergencia.

La prueba manual ignorada `real_database_creates_and_restores_verified_package`
requiere rutas explícitas `LF_BACKUP_TEST_*`. Lee la base real sin modificarla, crea
un paquete en una carpeta temporal y restaura solamente dentro de otra carpeta de
ensayo.
