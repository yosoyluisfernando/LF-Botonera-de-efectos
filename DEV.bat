@echo off
cd /d "%~dp0"
title LF Botonera - Desarrollo
echo.
echo Carpeta de desarrollo: %CD%
echo Version del proyecto:
node -p "require('./package.json').version"
echo.
echo Si otra instancia de desarrollo esta abierta, cierrala antes de continuar.
echo.
set "LF_PREVIEW_RELEASE_NOTES="
if exist "%TEMP%\LF-Botonera-Desarrollo-Novedades.once" (
    set "LF_PREVIEW_RELEASE_NOTES=1"
    del /q "%TEMP%\LF-Botonera-Desarrollo-Novedades.once"
)
if /i "%~1"=="novedades" set "LF_PREVIEW_RELEASE_NOTES=1"
call npm run tauri dev
pause
