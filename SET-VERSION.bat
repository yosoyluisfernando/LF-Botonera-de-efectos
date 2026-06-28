@echo off
setlocal
if "%~1"=="" (
    set /p VERSION=Nueva version (ejemplo 1.2.0):
) else (
    set VERSION=%~1
)
if "%VERSION%"=="" ( echo Cancelado. & pause & exit /b 1 )
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0SET-VERSION.ps1" -Version "%VERSION%"
pause
