# Organización de ramas

Desde el 10 de noviembre de 2026, GitHub conserva dos ramas.

| Rama | Uso |
| --- | --- |
| main | Código estable publicado |
| codex/desarrollo | Mejoras de Android y escritorio, OBS, MIDI y control remoto |

El trabajo nuevo continúa en codex/desarrollo. Una publicación o integración
en main se realiza cuando el autor la solicite.

## Consolidación

La base fue codex/control-remoto-lan, con todas las mejoras Android y de
escritorio probadas. Se integraron mediante fusión los commits de diagnóstico
de latencia y la preparación de Flatpak. Esta última incluye la documentación
publicada en main. Las demás ramas ya estaban contenidas en esos historiales.

El árbol de código de la aplicación coincide con el del último APK probado.
Se comprobaron las carpetas src y src-tauri, package.json, package-lock.json
y vite.config.js contra b59109a. La organización también actualiza los flujos
de GitHub Actions para codex/desarrollo.

La comparación de latencia se ejecuta manualmente o al cambiar sus propios
archivos. Las comprobaciones de Android y Flatpak conservan sus flujos.

## Respaldo de la unificación

El bundle verificado está en
D:/BOTONERA/app/.tools/git-respaldos/unificacion-2026-11-10-b59109a/ramas-antes.bundle.

El archivo ramas-antes.json, en esa misma carpeta, registra las referencias
anteriores. El bundle conserva el historial completo de esas referencias.

SHA-256 del bundle
EDF3F0A4F6C1818DA5C414A41867200D4552EE21DB6E48D24F4193B99DC5DA74

## Referencias antes de consolidar

| Rama anterior | Commit |
| --- | --- |
| codex/android | 5780d23ac9abab4908ee7bddd2d8c52f09134b20 |
| codex/android-latency-check | 78f246b6018840e98126720aea3756ceb2a1eaf9 |
| codex/beta-1.6.0 | 59680a5d400e921e7173c27acdfd2e4153e15d2a |
| codex/control-remoto-lan | b59109a3f9ddae869cd8bac7fb33ace7b3d26878 |
| codex/flatpak-test | 23f6b8e8e3d9a5a3b86590ece626eef5b843e1f1 |
| codex/midi-ble-desktop | 2fa20a21920da233d45420f087c6709a7d682e8e |
| main | ed38181d1e318784020addeff621feea8b5bc304 |
| respaldo/pre-obs-2026-10-22 | 935d788e5d795c6bd6902f9f75cdb307e7aa389f |
