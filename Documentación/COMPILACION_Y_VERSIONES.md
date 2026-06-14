# 📦 REGLAS DE COMPILACIÓN Y VERSIONES

Este documento establece las normativas sobre los sistemas operativos soportados y el proceso de compilación del proyecto (Tauri + Rust) para facilitar su distribución.

## 1. Sistemas Operativos Soportados

**Windows (10 y 11):**
*   **Decisión Técnica:** Queda **totalmente descartada** la compatibilidad con Windows 7, 8 y 8.1. 
*   **Motivo:** Tauri utiliza *WebView2* (Edge nativo) para renderizar la interfaz. Microsoft finalizó oficialmente el soporte de WebView2 para Windows 7 y 8 en enero de 2023. Además, el propio compilador moderno de Rust (v1.78+) exige Windows 10 como mínimo. Tratar de forzar la compatibilidad obligaría a usar herramientas obsoletas, repletas de fallas de seguridad y bugs de audio insalvables.
*   **Soporte Final:** Windows 10 y Windows 11 (Home/Pro/LTSC).

**Linux:**
*   Se compilará nativamente en paquetes `.deb` (para distros basadas en Debian/Ubuntu) y en `.AppImage` (para distribución universal).

---

## 2. Instrucciones de Compilación (Source Code)

Para cualquier usuario o desarrollador que descargue el código fuente y desee compilarlo por sí mismo, los pasos deben ser extremadamente sencillos:

### Preparación (Primera vez)
1. Instalar `Node.js` y `npm`.
2. Instalar `Rust` (usando `rustup`).
3. En la raíz del proyecto, ejecutar:
   ```bash
   npm install
   ```

### Compilar para Windows (Ejecutable .exe y MSI)
Ejecutar el siguiente comando en PowerShell o CMD:
```bash
npm run tauri build
```
Esto generará los instaladores listos para distribuir en la carpeta: `src-tauri/target/release/bundle/msi/`

### Compilar para Linux (.deb y .AppImage)
Ejecutar el siguiente comando en la terminal de Linux:
```bash
npm run tauri build
```
Los empaquetados `.deb` y `.AppImage` aparecerán en: `src-tauri/target/release/bundle/deb/` y `src-tauri/target/release/bundle/appimage/`

---

## 3. Subida de Versión (Version Bumping)

Para mantener el control del proyecto, subir de versión (ej. de `1.0.0` a `1.1.0`) será tan sencillo como modificar **un solo archivo**:
1. Abrir el archivo `package.json` en la raíz.
2. Modificar el campo `"version": "1.1.0"`.
3. Al compilar (`npm run tauri build`), Tauri leerá automáticamente esta versión y se la inyectará a los binarios de Rust (`.exe`, `.deb`, `.AppImage`) y a las propiedades nativas del sistema operativo.
