const { app, BrowserWindow, ipcMain, dialog } = require('electron');
const path = require('path');
const fs = require('fs');

const configPath = path.join(app.getPath('userData'), 'botonera_config.json');

const osPlatform = process.platform;
const isWindows = osPlatform === 'win32';
const isLinux = osPlatform === 'linux';
console.log(`Botonera ejecutándose en: ${osPlatform} (${isWindows ? 'Windows' : isLinux ? 'Linux' : 'Otro'})`);

function createWindow () {
  const win = new BrowserWindow({
    width: 1000,
    height: 700,
    title: 'LF Botonera de efectos',
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false,
      webSecurity: false 
    }
  });
  win.loadFile('index.html');
  win.setMenuBarVisibility(false); 
}

app.whenReady().then(createWindow);

ipcMain.handle('leer-config', () => {
  if (fs.existsSync(configPath)) {
    return JSON.parse(fs.readFileSync(configPath, 'utf-8'));
  }
  return null;
});

ipcMain.handle('guardar-config', (event, data) => {
  fs.writeFileSync(configPath, JSON.stringify(data));
  return true;
});

ipcMain.handle('abrir-explorador', async () => {
  const { canceled, filePaths } = await dialog.showOpenDialog({
    title: 'Seleccionar efecto de sonido',
    properties: ['openFile'],
    filters: [{ name: 'Archivos de Audio', extensions: ['mp3', 'wav', 'm4a', 'ogg', 'aac'] }]
  });
  if (canceled) return null;
  return filePaths[0]; 
});

ipcMain.handle('preguntar-eliminar', async (event, nombre) => {
  const res = await dialog.showMessageBox({
    type: 'warning',
    buttons: ['Eliminar', 'Exportar (.bdelf) y Eliminar', 'Cancelar'],
    title: 'Eliminar Botonera',
    message: `¿Qué deseas hacer con la botonera "${nombre}"?`,
    cancelId: 2
  });
  return res.response; 
});

ipcMain.handle('importar-bdelf', async () => {
  const { canceled, filePaths } = await dialog.showOpenDialog({
    title: 'Importar configuración de Botonera',
    properties: ['openFile'],
    filters: [{ name: 'Botonera LF', extensions: ['bdelf'] }]
  });
  
  if (!canceled && filePaths.length > 0) {
    try { return JSON.parse(fs.readFileSync(filePaths[0], 'utf-8')); } 
    catch (e) { return null; }
  }
  return null;
});

ipcMain.handle('exportar-bdelf', async (event, data) => {
  const { canceled, filePath } = await dialog.showSaveDialog({
    title: 'Guardar configuración de Botonera',
    defaultPath: `${data.nombre}.bdelf`,
    filters: [{ name: 'Botonera LF', extensions: ['bdelf'] }]
  });
  if (!canceled) {
    fs.writeFileSync(filePath, JSON.stringify(data, null, 2));
    return true;
  }
  return false;
});

ipcMain.handle('preguntar-eliminar-perfil', async (event, nombre) => {
  const res = await dialog.showMessageBox({
    type: 'warning',
    buttons: ['Eliminar', 'Exportar (.bdeplf) y Eliminar', 'Cancelar'],
    title: 'Eliminar Perfil',
    message: `¿Qué deseas hacer con el perfil "${nombre}"?`,
    cancelId: 2
  });
  return res.response; 
});

ipcMain.handle('importar-bdeplf', async () => {
  const { canceled, filePaths } = await dialog.showOpenDialog({
    title: 'Importar Perfil',
    properties: ['openFile'],
    filters: [{ name: 'Perfil LF', extensions: ['bdeplf'] }]
  });
  
  if (!canceled && filePaths.length > 0) {
    try { return JSON.parse(fs.readFileSync(filePaths[0], 'utf-8')); } 
    catch (e) { return null; }
  }
  return null;
});

ipcMain.handle('exportar-bdeplf', async (event, data) => {
  const { canceled, filePath } = await dialog.showSaveDialog({
    title: 'Exportar Perfil',
    defaultPath: `${data.name}.bdeplf`,
    filters: [{ name: 'Perfil LF', extensions: ['bdeplf'] }]
  });
  if (!canceled) {
    fs.writeFileSync(filePath, JSON.stringify(data, null, 2));
    return true;
  }
  return false;
});

ipcMain.handle('preguntar-reemplazo', async () => {
  const res = await dialog.showMessageBox({
    type: 'question',
    buttons: ['Reemplazar', 'Añadir en un lugar vacío', 'Cancelar'],
    title: 'Casilla Ocupada',
    message: 'Ya hay un sonido en esta casilla. ¿Qué deseas hacer?',
    cancelId: 2
  });
  return res.response; 
});