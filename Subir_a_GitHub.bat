@echo off
echo ========================================================
echo   Iniciando subida del proyecto a GitHub...
echo ========================================================
echo.

git init
git add .
git commit -m "Versión inicial de la Botonera"
git branch -M main
git remote add origin https://github.com/yosoyluisfernando/LF-Botonera-de-efectos.git
git push -u origin main

echo.
echo ========================================================
echo   Proceso finalizado. Verifica los mensajes arriba.
echo ========================================================
pause
