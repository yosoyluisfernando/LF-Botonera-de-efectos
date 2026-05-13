const electron = require('electron');
const { ipcRenderer, shell } = electron;
const webUtils = electron.webUtils; 

let appState = { activeProfileId: 'default', profiles: [] };
let state = null; 

let activeTabIndex = 0;
let botonSeleccionado = null; 
let tabSeleccionadaIndex = null; 
let modoTab = 'nuevo'; 
let modoProfile = 'nuevo'; 
let undoStack = []; 
let volumenOriginalAlEditar = 1;
let audioDevices = [];

let isMappingMode = false;
let mappingTarget = null; 
let mappingType = ''; 
let prelistenAudio = null;

const DEFAULT_TAB_BG = '#3a3f44';
const DEFAULT_TAB_TEXT = '#cccccc';
const DEFAULT_PROFILE_BG = '#008c3a'; 
const DEFAULT_PROFILE_TEXT = '#ffffff';

const grid = document.getElementById('grid');
const tabsContainer = document.getElementById('tabs');
const contextMenu = document.getElementById('context-menu');
const tabContextMenu = document.getElementById('tab-context-menu');
const profileContextMenu = document.getElementById('profile-context-menu');
const editModal = document.getElementById('edit-modal');
const tabModal = document.getElementById('tab-modal');
const profileModal = document.getElementById('profile-modal'); 
const settingsModal = document.getElementById('settings-modal');
const captureModal = document.getElementById('capture-modal');

const editFilepath = document.getElementById('edit-filepath');
const editName = document.getElementById('edit-name');
const editVolume = document.getElementById('edit-volume');
const editBgColor = document.getElementById('edit-bg-color');
const editTextColor = document.getElementById('edit-text-color');
const editShortcut = document.getElementById('edit-shortcut');

const tabNameInput = document.getElementById('tab-name');
const tabVInput = document.getElementById('tab-v');
const tabHInput = document.getElementById('tab-h');
const tabAudioOut = document.getElementById('tab-audio-out');
const tabBgColor = document.getElementById('tab-bg-color');
const tabTextColor = document.getElementById('tab-text-color');
const btnTabDefaultColors = document.getElementById('btn-tab-default-colors');

const profileNameInput = document.getElementById('profile-name');
const profileBgColor = document.getElementById('profile-bg-color');
const profileTextColor = document.getElementById('profile-text-color');
const btnProfileDefaultColors = document.getElementById('btn-profile-default-colors');

const configOutMain = document.getElementById('config-out-main');
const configOutPre = document.getElementById('config-out-pre');
const configKeyStop = document.getElementById('config-key-stop');
const configKeyNext = document.getElementById('config-key-next');
const configKeyPrev = document.getElementById('config-key-prev');

document.addEventListener('dragover', (e) => e.preventDefault());
document.addEventListener('drop', (e) => e.preventDefault());

async function init() {
  await cargarDispositivosAudio(); 
  const guardado = await ipcRenderer.invoke('leer-config');
  
  if (guardado) {
    if (guardado.profiles) {
      appState = guardado;
    } else {
      appState.activeProfileId = 'default';
      appState.profiles = [{
        id: 'default', name: 'Principal', bg: DEFAULT_PROFILE_BG, text: DEFAULT_PROFILE_TEXT,
        config: guardado.config || { outMain: 'default', outPre: 'default', keys: { stopAll: '', next: '', prev: '' } },
        paletas: guardado.paletas || []
      }];
    }
  } else {
    appState.activeProfileId = 'default';
    appState.profiles = [{
      id: 'default', name: 'Principal', bg: DEFAULT_PROFILE_BG, text: DEFAULT_PROFILE_TEXT,
      config: { outMain: 'default', outPre: 'default', keys: { stopAll: '', next: '', prev: '' } },
      paletas: []
    }];
  }

  appState.profiles.forEach(prof => {
      if (!prof.config) prof.config = { outMain: 'default', outPre: 'default', keys: { stopAll: '', next: '', prev: '' } };
      if (!prof.bg) prof.bg = DEFAULT_PROFILE_BG;
      if (!prof.text) prof.text = DEFAULT_PROFILE_TEXT;
      prof.paletas.forEach(p => {
          if (!p.rows) p.rows = 6; if (!p.cols) p.cols = 5; if (!p.audioOut) p.audioOut = 'global';
          if (!p.shortcut) p.shortcut = ''; 
          if (!p.tabBg) p.tabBg = DEFAULT_TAB_BG; if (!p.tabText) p.tabText = DEFAULT_TAB_TEXT;
          p.botones.forEach(b => { 
              b.audioElement = null; 
              b.clones = []; 
              if(!b.shortcut) b.shortcut = ''; 
              if(b.overlap === undefined) b.overlap = false; 
          });
      });
  });

  cambiarPerfil(appState.activeProfileId);
}

function updateProfileButton() {
    const btn = document.getElementById('btn-profiles');
    btn.innerText = `👤 ${state.name}`;
    btn.style.backgroundColor = state.bg || DEFAULT_PROFILE_BG;
    btn.style.color = state.text || DEFAULT_PROFILE_TEXT;
}

function cambiarPerfil(id) {
    if (state) detenerTodoGlobal(); 
    appState.activeProfileId = id;
    state = appState.profiles.find(p => p.id === id);
    if (!state) { state = appState.profiles[0]; appState.activeProfileId = state.id; }
    if (state.paletas.length === 0) { crearConfigPaleta('BOTONERA 1', 5, 5); }

    activeTabIndex = 0; undoStack = [];
    updateProfileButton();
    renderTabs(); renderGrid(); renderProfileList();
}

function renderProfileList() {
    const list = document.getElementById('profile-list');
    list.innerHTML = '';
    appState.profiles.forEach(prof => {
        let li = document.createElement('li');
        li.innerText = prof.name;
        if (prof.id === appState.activeProfileId) { li.style.fontWeight = 'bold'; li.style.color = '#00FF55'; }
        li.onclick = () => { cambiarPerfil(prof.id); profileContextMenu.classList.add('hidden'); guardarMemoria(); };
        list.appendChild(li);
    });
}

document.getElementById('btn-profiles').onclick = (e) => {
    const rect = e.target.getBoundingClientRect();
    profileContextMenu.style.left = `${rect.left - 150}px`; profileContextMenu.style.top = `${rect.bottom + 5}px`;
    profileContextMenu.classList.remove('hidden');
};

document.getElementById('menu-new-profile').onclick = () => {
    profileContextMenu.classList.add('hidden'); modoProfile = 'nuevo'; document.getElementById('profile-modal-title').innerText = 'Nuevo Perfil';
    profileNameInput.value = `Perfil ${appState.profiles.length + 1}`; profileBgColor.value = DEFAULT_PROFILE_BG; profileTextColor.value = DEFAULT_PROFILE_TEXT;
    profileModal.classList.remove('hidden'); profileNameInput.focus();
};

document.getElementById('menu-edit-profile').onclick = () => {
    profileContextMenu.classList.add('hidden'); modoProfile = 'editar'; document.getElementById('profile-modal-title').innerText = 'Editar Perfil';
    profileNameInput.value = state.name; profileBgColor.value = state.bg || DEFAULT_PROFILE_BG; profileTextColor.value = state.text || DEFAULT_PROFILE_TEXT;
    profileModal.classList.remove('hidden'); profileNameInput.focus();
};

document.getElementById('menu-delete-profile').onclick = async () => {
    profileContextMenu.classList.add('hidden');
    if (appState.profiles.length <= 1) { alert("¡No puedes eliminar este perfil porque es el único que tienes!"); return; }
    const accion = await ipcRenderer.invoke('preguntar-eliminar-perfil', state.name);
    if (accion === 0) { borrarPerfilActual(); }
    else if (accion === 1) {
        const exportado = await ipcRenderer.invoke('exportar-bdeplf', obtenerEstadoLimpioPerfil());
        if (exportado) borrarPerfilActual();
    }
};

function borrarPerfilActual() {
    detenerTodoGlobal();
    appState.profiles = appState.profiles.filter(p => p.id !== state.id);
    cambiarPerfil(appState.profiles[0].id);
    guardarMemoria();
}

btnProfileDefaultColors.onclick = (e) => { e.preventDefault(); profileBgColor.value = DEFAULT_PROFILE_BG; profileTextColor.value = DEFAULT_PROFILE_TEXT; };
document.getElementById('close-profile-modal').onclick = () => profileModal.classList.add('hidden');

document.getElementById('btn-save-profile').onclick = () => {
    const nombre = profileNameInput.value.trim() || 'Perfil'; const bg = profileBgColor.value; const text = profileTextColor.value;
    if (modoProfile === 'nuevo') {
        const newId = Date.now().toString();
        appState.profiles.push({ id: newId, name: nombre, bg: bg, text: text, config: { outMain: 'default', outPre: 'default', keys: { stopAll: '', next: '', prev: '' } }, paletas: [] });
        cambiarPerfil(newId);
    } else {
        state.name = nombre; state.bg = bg; state.text = text; guardarMemoria(); updateProfileButton(); renderProfileList();
    }
    profileModal.classList.add('hidden');
};

document.getElementById('menu-import-profile').onclick = async () => {
    profileContextMenu.classList.add('hidden');
    const imported = await ipcRenderer.invoke('importar-bdeplf');
    if (imported && imported.name && imported.paletas) {
        imported.id = Date.now().toString(); 
        if(!imported.bg) imported.bg = DEFAULT_PROFILE_BG; if(!imported.text) imported.text = DEFAULT_PROFILE_TEXT;
        imported.paletas.forEach(p => p.botones.forEach(b => { b.audioElement = null; b.clones = []; }));
        appState.profiles.push(imported); cambiarPerfil(imported.id);
    }
};

document.getElementById('menu-export-profile').onclick = async () => {
    profileContextMenu.classList.add('hidden'); await ipcRenderer.invoke('exportar-bdeplf', obtenerEstadoLimpioPerfil());
};

async function cargarDispositivosAudio() {
  try {
    await navigator.mediaDevices.getUserMedia({ audio: true }).then(s => s.getTracks().forEach(t => t.stop())).catch(e=>{});
    const devices = await navigator.mediaDevices.enumerateDevices();
    audioDevices = devices.filter(d => d.kind === 'audiooutput');
  } catch (err) { console.error("Error audio:", err); }
}

function poblarSelectAudio(selectElement, opcionExtra = null, valorSeleccionado = 'default') {
  selectElement.innerHTML = '';
  if (opcionExtra) {
    const opt = document.createElement('option'); opt.value = opcionExtra.value; opt.innerText = opcionExtra.label; selectElement.appendChild(opt);
  }
  const defOpt = document.createElement('option'); defOpt.value = 'default'; defOpt.innerText = 'Salida por Defecto (Sistema)'; selectElement.appendChild(defOpt);
  audioDevices.forEach(device => {
    if (device.deviceId !== 'default' && device.deviceId !== 'communications') {
      const opt = document.createElement('option'); opt.value = device.deviceId; opt.innerText = device.label || `Dispositivo (${device.deviceId.substring(0,5)}...)`; selectElement.appendChild(opt);
    }
  });
  selectElement.value = valorSeleccionado;
}

async function enrutarAudio(audioElement, tipo, paletaIndex = null) {
    if (!audioElement || typeof audioElement.setSinkId !== 'function') return; 
    let sinkId = 'default';
    if (tipo === 'prelisten') { sinkId = state.config.outPre || 'default'; } 
    else {
        const paleta = state.paletas[paletaIndex !== null ? paletaIndex : activeTabIndex];
        sinkId = (paleta && paleta.audioOut && paleta.audioOut !== 'global') ? paleta.audioOut : (state.config.outMain || 'default');
    }
    try { 
        const realSinkId = (sinkId === 'default' || sinkId === 'global') ? '' : sinkId;
        await audioElement.setSinkId(realSinkId); 
    } catch (e) { console.warn("Error al enrutar:", e); }
}

function obtenerEstadoLimpioPerfil() {
  return {
    id: state.id, name: state.name, bg: state.bg, text: state.text,
    config: JSON.parse(JSON.stringify(state.config)),
    paletas: state.paletas.map(p => ({
      nombre: p.nombre, rows: p.rows, cols: p.cols, audioOut: p.audioOut || 'global', shortcut: p.shortcut || '',
      tabBg: p.tabBg || DEFAULT_TAB_BG, tabText: p.tabText || DEFAULT_TAB_TEXT,
      botones: p.botones.map(b => ({
        id: b.id, label: b.label, file: b.file, name: b.name, bg: b.bg, text: b.text, vol: b.vol, loop: b.loop, stopOther: b.stopOther, overlap: b.overlap || false, shortcut: b.shortcut || ''
      }))
    }))
  };
}

function guardarEstadoPrevio() { undoStack.push(obtenerEstadoLimpioPerfil()); if (undoStack.length > 50) undoStack.shift(); }

function guardarMemoria() { 
  const cleanProfile = obtenerEstadoLimpioPerfil();
  const idx = appState.profiles.findIndex(p => p.id === state.id);
  if(idx !== -1) appState.profiles[idx] = cleanProfile;

  const fullCleanState = {
      activeProfileId: appState.activeProfileId,
      profiles: appState.profiles.map(prof => (prof.id === state.id) ? cleanProfile : prof)
  };
  ipcRenderer.invoke('guardar-config', fullCleanState); 
}

function crearConfigPaleta(nombre, v, h) {
  guardarEstadoPrevio(); let botones = []; const total = v * h;
  for (let i = 1; i <= total; i++) { botones.push({ id: i, label: (i).toString(), file: '', name: '', bg: '', text: '#FFFFFF', vol: 1, loop: false, stopOther: false, overlap: false, shortcut: '' }); }
  state.paletas.push({ nombre, rows: v, cols: h, audioOut: 'global', shortcut: '', tabBg: DEFAULT_TAB_BG, tabText: DEFAULT_TAB_TEXT, botones }); 
  guardarMemoria();
}

function ajustarBotonesPaleta(paleta, v, h) {
  const total = v * h;
  if (paleta.botones.length < total) {
    for (let i = paleta.botones.length + 1; i <= total; i++) { paleta.botones.push({ id: i, label: i.toString(), file: '', name: '', bg: '', text: '#FFFFFF', vol: 1, loop: false, stopOther: false, overlap: false, shortcut: '' }); }
  } else if (paleta.botones.length > total) { paleta.botones = paleta.botones.slice(0, total); }
  paleta.botones.forEach((b, i) => b.id = i + 1); paleta.rows = v; paleta.cols = h;
}

function actualizarEstadoPestañasVisibles() {
  const tabsDOM = document.querySelectorAll('#tabs .tab');
  state.paletas.forEach((paleta, index) => {
    const isPlaying = paleta.botones.some(b => 
        (b.audioElement && !b.audioElement.paused) || 
        (b.clones && b.clones.length > 0)
    );
    const tabEl = tabsDOM[index];
    if (tabEl) {
      if (isPlaying && index !== activeTabIndex) tabEl.classList.add('tab-playing');
      else tabEl.classList.remove('tab-playing');
    }
  });
}

function renderTabs() {
  tabsContainer.innerHTML = '';
  state.paletas.forEach((paleta, index) => {
    let tab = document.createElement('div');
    tab.className = `tab ${index === activeTabIndex ? 'active' : ''}`;
    
    if (paleta.tabBg && paleta.tabBg !== DEFAULT_TAB_BG) tab.style.backgroundColor = paleta.tabBg;
    if (paleta.tabText && paleta.tabText !== DEFAULT_TAB_TEXT) tab.style.color = paleta.tabText;

    tab.innerHTML = paleta.shortcut ? `<b>[${paleta.shortcut}]</b> ${paleta.nombre}` : paleta.nombre;
    
    tab.onclick = (e) => { 
      if (isMappingMode) { e.preventDefault(); e.stopPropagation(); abrirCaptura(paleta, 'tab'); return; }
      activeTabIndex = index; renderTabs(); renderGrid(); 
    };
    tab.oncontextmenu = (e) => {
      e.preventDefault(); tabSeleccionadaIndex = index;
      tabContextMenu.style.left = `${e.clientX}px`; tabContextMenu.style.top = `${e.clientY}px`; tabContextMenu.classList.remove('hidden');
    };
    tabsContainer.appendChild(tab);
  });
  actualizarEstadoPestañasVisibles();
}

function formatTime(seconds) {
  if (isNaN(seconds)) return "00:00"; const m = Math.floor(seconds / 60).toString().padStart(2, '0'); const s = Math.floor(seconds % 60).toString().padStart(2, '0'); return `${m}:${s}`;
}

function abrirEdicion(btnInfo) {
    botonSeleccionado = btnInfo; volumenOriginalAlEditar = btnInfo.vol || 1; 
    editFilepath.value = botonSeleccionado.file || ''; editName.value = botonSeleccionado.name || ''; editVolume.value = botonSeleccionado.vol || 1; editBgColor.value = botonSeleccionado.bg || '#444444'; editTextColor.value = botonSeleccionado.text || '#FFFFFF'; editShortcut.value = botonSeleccionado.shortcut || '';
    contextMenu.classList.add('hidden'); editModal.classList.remove('hidden'); editName.focus();
}

editVolume.addEventListener('input', (e) => { if (botonSeleccionado && botonSeleccionado.audioElement) { botonSeleccionado.audioElement.volume = parseFloat(e.target.value); } });

function cancelarEdicion() {
    if (botonSeleccionado && botonSeleccionado.audioElement) { botonSeleccionado.audioElement.volume = volumenOriginalAlEditar; }
    editModal.classList.add('hidden');
}

function renderGrid() {
  grid.innerHTML = ''; const paletaActual = state.paletas[activeTabIndex];
  grid.style.gridTemplateColumns = `repeat(${paletaActual.cols}, 1fr)`; grid.style.gridTemplateRows = `repeat(${paletaActual.rows}, 1fr)`;
  
  paletaActual.botones.forEach((btnInfo) => {
    let btn = document.createElement('div'); btn.className = 'grid-item'; btn.id = `btn-dom-${btnInfo.id}`; 
    btn.draggable = false; 
    
    if (btnInfo.bg) btn.style.backgroundColor = btnInfo.bg; if (btnInfo.text) btn.style.color = btnInfo.text;
    
    const isPlayingUI = (btnInfo.audioElement && !btnInfo.audioElement.paused) || (btnInfo.clones && btnInfo.clones.length > 0);
    if (isPlayingUI) btn.classList.add('playing');
    
    let shortcutHtml = btnInfo.shortcut ? `<span style="position:absolute; top:5px; right:5px; font-size:10px; background:rgba(0,0,0,0.6); padding:2px 5px; border-radius:3px;">${btnInfo.shortcut}</span>` : '';

    btn.innerHTML = `<span class="index">${btnInfo.id}</span>${shortcutHtml}<span class="name">${btnInfo.name || ''}</span><span class="timer" id="timer-${btnInfo.id}">${btnInfo.file ? 'LISTO' : ''}</span><div class="progress-container"><div class="progress-bar" id="progress-${btnInfo.id}"></div></div>`;

    btn.addEventListener('mousedown', (e) => { if (e.altKey) { btn.draggable = true; } else { btn.draggable = false; } });
    btn.addEventListener('mouseup', () => { btn.draggable = false; });

    btn.addEventListener('click', (e) => { 
       if (isMappingMode) { e.preventDefault(); e.stopPropagation(); abrirCaptura(btnInfo, 'button'); return; }
       reproducirAudio(btnInfo, btn); 
    });

    btn.addEventListener('contextmenu', (e) => {
      e.preventDefault(); e.stopPropagation(); botonSeleccionado = btnInfo;
      document.getElementById('check-bucle').innerText = btnInfo.loop ? '✓' : ''; 
      document.getElementById('check-detener').innerText = btnInfo.stopOther ? '✓' : '';
      document.getElementById('check-overlap').innerText = btnInfo.overlap ? '✓' : '';
      contextMenu.style.left = `${e.clientX}px`; contextMenu.style.top = `${e.clientY}px`; contextMenu.classList.remove('hidden');
    });

    btn.addEventListener('dragstart', (e) => { e.dataTransfer.setData('text/plain', btnInfo.id); });
    
    btn.addEventListener('dragenter', (e) => { e.preventDefault(); btn.classList.add('drag-over'); });
    btn.addEventListener('dragover', (e) => { e.preventDefault(); btn.classList.add('drag-over'); });
    btn.addEventListener('dragleave', () => btn.classList.remove('drag-over'));

    btn.addEventListener('drop', async (e) => {
      e.preventDefault(); e.stopPropagation(); btn.classList.remove('drag-over');
      
      if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
        const file = e.dataTransfer.files[0]; 
        let filePath = file.path;

        if (webUtils && webUtils.getPathForFile) {
            try { const realPath = webUtils.getPathForFile(file); if (realPath) filePath = realPath; } 
            catch(err) { console.warn("Error webUtils:", err); }
        }

        if (!filePath || !filePath.match(/\.(mp3|wav|ogg|m4a|aac)$/i)) return; 
        
        const prepararYABrirEdicion = (targetBtn) => {
           abrirEdicion(targetBtn);
           editFilepath.value = filePath; 
           let nombre = filePath.split('\\').pop().split('/').pop(); 
           if(nombre.includes('.')) nombre = nombre.substring(0, nombre.lastIndexOf('.'));
           editName.value = nombre.toUpperCase();
        };
        
        if (btnInfo.file) {
          const accion = await ipcRenderer.invoke('preguntar-reemplazo');
          if (accion === 0) prepararYABrirEdicion(btnInfo);
          else if (accion === 1) {
            const vacio = paletaActual.botones.find(b => !b.file);
            if (vacio) prepararYABrirEdicion(vacio); else alert("No hay botones vacíos.");
          }
        } else {
          prepararYABrirEdicion(btnInfo);
        }
        return;
      }
      
      const originId = parseInt(e.dataTransfer.getData('text/plain'));
      if (originId && originId !== btnInfo.id) {
        guardarEstadoPrevio(); 
        const indexA = paletaActual.botones.findIndex(b => b.id === originId);
        const indexB = paletaActual.botones.findIndex(b => b.id === btnInfo.id);
        const tempA = { ...paletaActual.botones[indexA] };
        const tempB = { ...paletaActual.botones[indexB] };
        paletaActual.botones[indexA] = { ...tempB, id: originId };
        paletaActual.botones[indexB] = { ...tempA, id: btnInfo.id };
        guardarMemoria(); renderGrid();
      }
    });
    grid.appendChild(btn);

    if (btnInfo.audioElement) {
      const audio = btnInfo.audioElement;
      let paletaIndex = activeTabIndex; 
      
      if (audio.duration && !audio.paused) {
        const pb = document.getElementById(`progress-${btnInfo.id}`); const tt = document.getElementById(`timer-${btnInfo.id}`);
        if(pb) pb.style.width = `${(audio.currentTime / audio.duration) * 100}%`; if(tt) tt.innerText = `${formatTime(audio.currentTime)} / ${formatTime(audio.duration)}`;
      }
      audio.ontimeupdate = () => {
        if (audio.duration) {
          if (activeTabIndex === paletaIndex) {
            const pb = document.getElementById(`progress-${btnInfo.id}`); const tt = document.getElementById(`timer-${btnInfo.id}`);
            if (pb && tt) { pb.style.width = `${(audio.currentTime / audio.duration) * 100}%`; tt.innerText = `${formatTime(audio.currentTime)} / ${formatTime(audio.duration)}`; }
          }
        }
      };
      audio.onended = () => { 
        if (!btnInfo.loop) {
            if (!btnInfo.clones || btnInfo.clones.length === 0) detenerAudio(btnInfo);
        }
        actualizarEstadoPestañasVisibles(); 
      };
    }
  });
}

function detenerAudio(btnInfo) {
  if (btnInfo.audioElement && typeof btnInfo.audioElement.pause === 'function') { 
      btnInfo.audioElement.pause(); 
      btnInfo.audioElement.currentTime = 0; 
  }
  
  if (btnInfo.clones && btnInfo.clones.length > 0) {
      btnInfo.clones.forEach(c => { c.pause(); c.currentTime = 0; });
      btnInfo.clones = [];
  }

  let paletaIndex = state.paletas.findIndex(p => p.botones.includes(btnInfo));
  if (activeTabIndex === paletaIndex) {
      const btnDOM = document.getElementById(`btn-dom-${btnInfo.id}`);
      if (btnDOM) { 
        btnDOM.classList.remove('playing'); 
        const pb = document.getElementById(`progress-${btnInfo.id}`);
        const tt = document.getElementById(`timer-${btnInfo.id}`);
        if(pb) pb.style.width = '0%'; 
        if(tt) tt.innerText = btnInfo.file ? 'LISTO' : ''; 
      }
  }
  actualizarEstadoPestañasVisibles();
}

async function reproducirAudio(btnInfo, btnDOM) {
  if (!btnInfo.file) return;

  if (btnInfo.audioElement && typeof btnInfo.audioElement.paused !== 'undefined' && !btnInfo.audioElement.paused) { 
      if (btnInfo.overlap) {
          let clone = new Audio('file:///' + btnInfo.file.replace(/\\/g, '/'));
          clone.volume = btnInfo.vol;
          clone.loop = btnInfo.loop;
          if (!btnInfo.clones) btnInfo.clones = [];
          btnInfo.clones.push(clone);
          
          let paletaIndex = state.paletas.findIndex(p => p.botones.includes(btnInfo));
          await enrutarAudio(clone, 'main', paletaIndex !== -1 ? paletaIndex : activeTabIndex);
          
          clone.onended = () => { 
              btnInfo.clones = btnInfo.clones.filter(c => c !== clone); 
              if (btnInfo.audioElement && btnInfo.audioElement.paused && btnInfo.clones.length === 0) {
                  detenerAudio(btnInfo);
              }
              actualizarEstadoPestañasVisibles();
          };
          clone.play().catch(e => console.error("Error reproduciendo clon:", e));
          return; 
      } else {
          detenerAudio(btnInfo); 
          return; 
      }
  }

  if (btnInfo.stopOther) {
    state.paletas[activeTabIndex].botones.forEach(b => { if (b.audioElement && !b.audioElement.paused && b.id !== btnInfo.id && b.stopOther === true) detenerAudio(b); });
  }
  
  if (!btnInfo.audioElement || !btnInfo.audioElement.play) { btnInfo.audioElement = new Audio('file:///' + btnInfo.file.replace(/\\/g, '/')); }
  const audio = btnInfo.audioElement; audio.volume = btnInfo.vol; audio.loop = btnInfo.loop;
  
  let paletaIndex = state.paletas.findIndex(p => p.botones.includes(btnInfo));
  await enrutarAudio(audio, 'main', paletaIndex !== -1 ? paletaIndex : activeTabIndex);

  audio.ontimeupdate = () => {
    if (audio.duration) {
      if (activeTabIndex === paletaIndex) {
        const pb = document.getElementById(`progress-${btnInfo.id}`); const tt = document.getElementById(`timer-${btnInfo.id}`);
        if (pb && tt) { pb.style.width = `${(audio.currentTime / audio.duration) * 100}%`; tt.innerText = `${formatTime(audio.currentTime)} / ${formatTime(audio.duration)}`; }
      }
    }
  };
  audio.onended = () => { 
    if (!btnInfo.loop) {
        if (!btnInfo.clones || btnInfo.clones.length === 0) detenerAudio(btnInfo);
    }
    actualizarEstadoPestañasVisibles(); 
  };
  
  if(!btnDOM && activeTabIndex === paletaIndex) btnDOM = document.getElementById(`btn-dom-${btnInfo.id}`);
  audio.play().then(() => { 
    if(btnDOM) btnDOM.classList.add('playing'); 
    actualizarEstadoPestañasVisibles();
  }).catch(e => console.error("Error reproduciendo:", e));
}

function detenerTodoGlobal() { state.paletas.forEach(p => p.botones.forEach(b => detenerAudio(b))); }

async function iniciarPrelisten(ruta, nombre, volumenBase) {
  if (prelistenAudio) detenerPrelisten();
  if (!ruta) return;
  prelistenAudio = new Audio('file:///' + ruta.replace(/\\/g, '/'));
  prelistenAudio.volume = volumenBase; document.getElementById('prelisten-volume').value = volumenBase;
  await enrutarAudio(prelistenAudio, 'prelisten');
  document.getElementById('prelisten-name').innerText = nombre || 'AUDIO';
  document.getElementById('prelisten-player').classList.remove('hidden');

  prelistenAudio.ontimeupdate = () => {
      if (prelistenAudio.duration) {
          const pb = document.getElementById('prelisten-progress'); const tt = document.getElementById('prelisten-time');
          if(pb && tt) { pb.style.width = `${(prelistenAudio.currentTime / prelistenAudio.duration) * 100}%`; tt.innerText = `${formatTime(prelistenAudio.currentTime)} / ${formatTime(prelistenAudio.duration)}`; }
      }
  };
  prelistenAudio.onended = detenerPrelisten;
  prelistenAudio.play().catch(e => console.error("Error prelisten:", e));
}

function detenerPrelisten() {
  if (prelistenAudio) { prelistenAudio.pause(); prelistenAudio = null; }
  document.getElementById('prelisten-player').classList.add('hidden');
}

document.getElementById('close-prelisten').onclick = detenerPrelisten;
document.getElementById('btn-stop-prelisten').onclick = detenerPrelisten;
document.getElementById('prelisten-volume').addEventListener('input', (e) => { if (prelistenAudio) prelistenAudio.volume = e.target.value; });

document.getElementById('menu-previa').onclick = () => { iniciarPrelisten(botonSeleccionado.file, botonSeleccionado.name, botonSeleccionado.vol); contextMenu.classList.add('hidden'); };
document.getElementById('btn-prelisten').onclick = () => { iniciarPrelisten(editFilepath.value, editName.value, parseFloat(editVolume.value)); };

document.getElementById('btn-enter-mapping').onclick = () => {
  settingsModal.classList.add('hidden');
  isMappingMode = true; document.body.classList.add('mapping-mode'); document.getElementById('mapping-banner').classList.remove('hidden');
};

function salirModoMapeo() {
  isMappingMode = false; mappingTarget = null; document.body.classList.remove('mapping-mode'); document.getElementById('mapping-banner').classList.add('hidden'); captureModal.classList.add('hidden');
}

function abrirCaptura(target, type) {
  mappingTarget = target; mappingType = type;
  document.getElementById('capture-target-name').innerText = type === 'tab' ? `Pestaña: ${target.nombre}` : `Botón ${target.id}: ${target.name || '(Sin Nombre)'}`;
  document.getElementById('capture-key-input').value = target.shortcut || '';
  captureModal.classList.remove('hidden'); document.getElementById('capture-key-input').focus();
}

document.getElementById('btn-cancel-capture').onclick = () => captureModal.classList.add('hidden');
document.getElementById('btn-save-capture').onclick = () => {
  guardarEstadoPrevio(); mappingTarget.shortcut = document.getElementById('capture-key-input').value; guardarMemoria(); renderTabs(); renderGrid(); salirModoMapeo();
};

function registrarInputAtajo(inputElement) {
    inputElement.addEventListener('keydown', (e) => {
        e.preventDefault();
        if(e.key === 'Escape' || e.key === 'Backspace' || e.key === 'Delete') { inputElement.value = ''; return; }
        let keyText = ''; if (e.ctrlKey) keyText += 'Ctrl+'; if (e.altKey) keyText += 'Alt+'; if (e.shiftKey) keyText += 'Shift+';
        if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) return; 
        keyText += e.key.toUpperCase(); inputElement.value = keyText;
    });
}
registrarInputAtajo(editShortcut); registrarInputAtajo(configKeyStop); registrarInputAtajo(configKeyNext); registrarInputAtajo(configKeyPrev);
registrarInputAtajo(document.getElementById('capture-key-input')); 

// NUEVO EVENTO PARA DONACIONES
document.getElementById('btn-donate').onclick = () => {
    shell.openExternal('https://www.paypal.com/paypalme/yosoyluisfernando');
};

document.addEventListener('keydown', (e) => {
  
  if (e.key === 'Escape') {
      if (e.target.tagName === 'SELECT') return; 
      let handled = false;
      if (!captureModal.classList.contains('hidden')) { if (isMappingMode) salirModoMapeo(); else captureModal.classList.add('hidden'); handled = true; }
      else if (!profileModal.classList.contains('hidden')) { profileModal.classList.add('hidden'); handled = true; }
      else if (!editModal.classList.contains('hidden')) { cancelarEdicion(); handled = true; }
      else if (!tabModal.classList.contains('hidden')) { tabModal.classList.add('hidden'); handled = true; }
      else if (!settingsModal.classList.contains('hidden')) { settingsModal.classList.add('hidden'); handled = true; }
      if (prelistenAudio) { detenerPrelisten(); handled = true; }
      if (handled) { e.preventDefault(); return; }
  }
  
  if (e.key === 'Enter') {
      if (e.target.tagName === 'SELECT' || e.target.tagName === 'BUTTON') return; 
      let handled = false;
      if (!captureModal.classList.contains('hidden')) { document.getElementById('btn-save-capture').click(); handled = true; }
      else if (!profileModal.classList.contains('hidden')) { document.getElementById('btn-save-profile').click(); handled = true; }
      else if (!editModal.classList.contains('hidden')) { document.getElementById('btn-save-edit').click(); handled = true; }
      else if (!tabModal.classList.contains('hidden')) { document.getElementById('btn-save-tab').click(); handled = true; }
      else if (!settingsModal.classList.contains('hidden')) { document.getElementById('btn-save-settings').click(); handled = true; }
      if (handled) { e.preventDefault(); return; }
  }

  if (['INPUT', 'TEXTAREA'].includes(e.target.tagName) && !e.target.classList.contains('key-input') && e.target.id !== 'edit-shortcut' && e.target.id !== 'capture-key-input') {
      return; 
  }

  if (e.ctrlKey && e.key.toLowerCase() === 'z') { 
      if (undoStack.length > 0) { 
          const restoredState = undoStack.pop();
          
          state.paletas.forEach(currentPaleta => {
              currentPaleta.botones.forEach(currentBtn => {
                  if (currentBtn.clones && currentBtn.clones.length > 0) {
                      currentBtn.clones.forEach(c => { c.pause(); c.currentTime = 0; });
                      currentBtn.clones = [];
                  }

                  if (currentBtn.file && currentBtn.audioElement && !currentBtn.audioElement.paused) {
                      let transferred = false;
                      for (let rPaleta of restoredState.paletas) {
                          let rBtn = rPaleta.botones.find(b => b.file === currentBtn.file && b.file !== '');
                          if (rBtn) {
                              rBtn.audioElement = currentBtn.audioElement;
                              transferred = true;
                              break;
                          }
                      }
                      if (!transferred) { currentBtn.audioElement.pause(); currentBtn.audioElement.currentTime = 0; }
                  } else {
                      if (currentBtn.audioElement) { currentBtn.audioElement.pause(); currentBtn.audioElement = null; }
                  }
              });
          });

          state = restoredState; 
          guardarMemoria(); updateProfileButton(); renderTabs(); renderGrid(); 
      } 
      return; 
  }

  if (e.key === 'F1') { e.preventDefault(); shell.openExternal('https://www.paypal.com/paypalme/yosoyluisfernando'); return; }
  if (e.ctrlKey && e.key.toLowerCase() === 't') { e.preventDefault(); document.getElementById('add-tab').click(); return; }

  let pressedKey = '';
  if (e.ctrlKey) pressedKey += 'Ctrl+'; if (e.altKey) pressedKey += 'Alt+'; if (e.shiftKey) pressedKey += 'Shift+';
  if (!['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) pressedKey += e.key.toUpperCase();

  if (pressedKey === state.config.keys.stopAll && pressedKey !== '') { detenerTodoGlobal(); return; }
  if (pressedKey === state.config.keys.next && pressedKey !== '') { activeTabIndex = (activeTabIndex + 1) % state.paletas.length; renderTabs(); renderGrid(); return; }
  if (pressedKey === state.config.keys.prev && pressedKey !== '') { activeTabIndex = (activeTabIndex - 1 + state.paletas.length) % state.paletas.length; renderTabs(); renderGrid(); return; }

  const tabTargetIndex = state.paletas.findIndex(p => p.shortcut === pressedKey && p.shortcut !== '');
  if (tabTargetIndex !== -1) { activeTabIndex = tabTargetIndex; renderTabs(); renderGrid(); return; }

  let btnTriggered = null;
  for (const paleta of state.paletas) {
     const b = paleta.botones.find(btn => btn.shortcut === pressedKey && btn.shortcut !== '');
     if (b) { btnTriggered = b; break; }
  }
  if (btnTriggered) {
     const btnDOM = document.getElementById(`btn-dom-${btnTriggered.id}`); 
     reproducirAudio(btnTriggered, btnDOM); return;
  }
});

document.addEventListener('click', (e) => {
  if (!contextMenu.contains(e.target)) contextMenu.classList.add('hidden');
  if (!tabContextMenu.contains(e.target)) tabContextMenu.classList.add('hidden');
  if (!profileContextMenu.contains(e.target) && e.target.id !== 'btn-profiles') profileContextMenu.classList.add('hidden');
});

document.getElementById('tab-menu-exportar').onclick = async () => { tabContextMenu.classList.add('hidden'); await ipcRenderer.invoke('exportar-bdelf', state.paletas[tabSeleccionadaIndex]); };
document.getElementById('tab-menu-importar').onclick = async () => {
  tabContextMenu.classList.add('hidden');
  const importedData = await ipcRenderer.invoke('importar-bdelf');
  if (importedData && importedData.nombre && Array.isArray(importedData.botones)) {
    guardarEstadoPrevio(); importedData.audioOut = 'global'; importedData.tabBg = DEFAULT_TAB_BG; importedData.tabText = DEFAULT_TAB_TEXT; importedData.botones.forEach(b => b.audioElement = null);
    state.paletas.push(importedData); activeTabIndex = state.paletas.length - 1; guardarMemoria(); renderTabs(); renderGrid();
  }
};

document.getElementById('tab-menu-editar').onclick = () => {
  modoTab = 'editar'; const paleta = state.paletas[tabSeleccionadaIndex]; document.getElementById('tab-modal-title').innerText = 'Editar Botonera';
  tabNameInput.value = paleta.nombre; tabVInput.value = paleta.rows; tabHInput.value = paleta.cols; tabBgColor.value = paleta.tabBg || DEFAULT_TAB_BG; tabTextColor.value = paleta.tabText || DEFAULT_TAB_TEXT;
  poblarSelectAudio(tabAudioOut, {value: 'global', label: 'Usar Salida Principal (Global)'}, paleta.audioOut || 'global');
  tabContextMenu.classList.add('hidden'); tabModal.classList.remove('hidden');
};

document.getElementById('tab-menu-eliminar').onclick = async () => {
  tabContextMenu.classList.add('hidden');
  if (state.paletas.length <= 1) { alert("¡No puedes eliminar esta pestaña!"); return; }
  const paleta = state.paletas[tabSeleccionadaIndex]; const accion = await ipcRenderer.invoke('preguntar-eliminar', paleta.nombre);
  if (accion === 0) { guardarEstadoPrevio(); state.paletas[tabSeleccionadaIndex].botones.forEach(b => detenerAudio(b)); state.paletas.splice(tabSeleccionadaIndex, 1); activeTabIndex = 0; guardarMemoria(); renderTabs(); renderGrid(); } 
};

document.getElementById('menu-overlap').onclick = () => {
    guardarEstadoPrevio(); 
    botonSeleccionado.overlap = !botonSeleccionado.overlap; 
    guardarMemoria(); 
    contextMenu.classList.add('hidden'); 
};

btnTabDefaultColors.onclick = (e) => { e.preventDefault(); tabBgColor.value = DEFAULT_TAB_BG; tabTextColor.value = DEFAULT_TAB_TEXT; };

document.getElementById('add-tab').onclick = () => {
  modoTab = 'nuevo'; document.getElementById('tab-modal-title').innerText = 'Nueva Botonera';
  tabNameInput.value = `BOTONERA ${state.paletas.length + 1}`; tabVInput.value = 5; tabHInput.value = 5; tabBgColor.value = DEFAULT_TAB_BG; tabTextColor.value = DEFAULT_TAB_TEXT;
  poblarSelectAudio(tabAudioOut, {value: 'global', label: 'Usar Salida Principal (Global)'}, 'global');
  tabModal.classList.remove('hidden'); tabNameInput.focus(); 
};

document.getElementById('close-tab-modal').onclick = () => tabModal.classList.add('hidden');

document.getElementById('btn-save-tab').onclick = () => {
  guardarEstadoPrevio();
  const nombre = tabNameInput.value.trim() || 'Botonera'; const v = parseInt(tabVInput.value) || 5; const h = parseInt(tabHInput.value) || 5;
  if (modoTab === 'nuevo') {
    let botones = []; for (let i = 1; i <= (v * h); i++) { botones.push({ id: i, label: (i).toString(), file: '', name: '', bg: '', text: '#FFFFFF', vol: 1, loop: false, stopOther: false, overlap: false, shortcut: '' }); }
    state.paletas.push({ nombre, rows: v, cols: h, audioOut: tabAudioOut.value, shortcut: '', tabBg: tabBgColor.value, tabText: tabTextColor.value, botones }); activeTabIndex = state.paletas.length - 1; 
  } else {
    const paleta = state.paletas[tabSeleccionadaIndex]; paleta.nombre = nombre; paleta.audioOut = tabAudioOut.value; paleta.tabBg = tabBgColor.value; paleta.tabText = tabTextColor.value; ajustarBotonesPaleta(paleta, v, h);
  }
  guardarMemoria(); renderTabs(); renderGrid(); tabModal.classList.add('hidden');
};

document.getElementById('menu-editar').onclick = () => abrirEdicion(botonSeleccionado);
document.getElementById('menu-limpiar').onclick = () => { guardarEstadoPrevio(); detenerAudio(botonSeleccionado); botonSeleccionado.file = ''; botonSeleccionado.name = ''; botonSeleccionado.bg = ''; botonSeleccionado.shortcut = ''; botonSeleccionado.overlap = false; guardarMemoria(); renderGrid(); contextMenu.classList.add('hidden'); };
document.getElementById('menu-bucle').onclick = () => { guardarEstadoPrevio(); botonSeleccionado.loop = !botonSeleccionado.loop; if(botonSeleccionado.audioElement) botonSeleccionado.audioElement.loop = botonSeleccionado.loop; guardarMemoria(); contextMenu.classList.add('hidden'); };
document.getElementById('menu-detener').onclick = () => { guardarEstadoPrevio(); botonSeleccionado.stopOther = !botonSeleccionado.stopOther; guardarMemoria(); contextMenu.classList.add('hidden'); };

document.getElementById('close-modal').onclick = cancelarEdicion; document.getElementById('btn-cancel-edit').onclick = cancelarEdicion;

document.getElementById('btn-select-file').onclick = async () => {
  const ruta = await ipcRenderer.invoke('abrir-explorador');
  if (ruta) { editFilepath.value = ruta; const nombre = ruta.split('\\').pop().split('/').pop(); editName.value = (nombre.substring(0, nombre.lastIndexOf('.')) || nombre).toUpperCase(); }
};

document.getElementById('btn-save-edit').onclick = () => {
  guardarEstadoPrevio(); 
  if (botonSeleccionado.file !== editFilepath.value) { detenerAudio(botonSeleccionado); botonSeleccionado.audioElement = null; }
  botonSeleccionado.file = editFilepath.value; botonSeleccionado.name = editName.value; botonSeleccionado.vol = parseFloat(editVolume.value); 
  botonSeleccionado.bg = editBgColor.value; botonSeleccionado.text = editTextColor.value; botonSeleccionado.shortcut = editShortcut.value;
  if (botonSeleccionado.audioElement) botonSeleccionado.audioElement.volume = botonSeleccionado.vol;
  guardarMemoria(); renderGrid(); editModal.classList.add('hidden');
};

document.getElementById('btn-settings').onclick = () => {
    poblarSelectAudio(configOutMain, null, state.config.outMain); poblarSelectAudio(configOutPre, null, state.config.outPre);
    configKeyStop.value = state.config.keys.stopAll || ''; configKeyNext.value = state.config.keys.next || ''; configKeyPrev.value = state.config.keys.prev || '';
    settingsModal.classList.remove('hidden');
};
document.getElementById('close-settings-modal').onclick = () => settingsModal.classList.add('hidden');
document.getElementById('btn-save-settings').onclick = () => {
    state.config.outMain = configOutMain.value; state.config.outPre = configOutPre.value; state.config.keys.stopAll = configKeyStop.value; state.config.keys.next = configKeyNext.value; state.config.keys.prev = configKeyPrev.value;
    guardarMemoria(); settingsModal.classList.add('hidden');
};

document.querySelectorAll('.s-tab').forEach(tab => {
    tab.onclick = () => { document.querySelectorAll('.s-tab').forEach(t => t.classList.remove('active')); document.querySelectorAll('.s-panel').forEach(p => p.classList.add('hidden')); tab.classList.add('active'); document.getElementById(tab.getAttribute('data-target')).classList.remove('hidden'); }
});

function hacerVentanaArrastrable(modalId, isFloating = false) {
  const modal = document.getElementById(modalId);
  if (!modal) return;
  const content = isFloating ? modal : modal.querySelector('.modal-content');
  const header = isFloating ? modal.querySelector('.prelisten-header') : modal.querySelector('.modal-header');

  if (!header || !content) return;

  let isDragging = false;
  let offsetX, offsetY;

  header.addEventListener('mousedown', (e) => {
    if(e.target.classList.contains('close-btn')) return;
    isDragging = true;
    const rect = content.getBoundingClientRect();
    content.style.position = 'absolute';
    content.style.left = rect.left + 'px';
    content.style.top = rect.top + 'px';
    content.style.margin = '0'; 
    offsetX = e.clientX - rect.left;
    offsetY = e.clientY - rect.top;
  });

  document.addEventListener('mousemove', (e) => {
    if (!isDragging) return;
    content.style.left = (e.clientX - offsetX) + 'px';
    content.style.top = (e.clientY - offsetY) + 'px';
  });

  document.addEventListener('mouseup', () => { isDragging = false; });
}

hacerVentanaArrastrable('edit-modal');
hacerVentanaArrastrable('tab-modal');
hacerVentanaArrastrable('profile-modal'); 
hacerVentanaArrastrable('settings-modal');
hacerVentanaArrastrable('capture-modal');
hacerVentanaArrastrable('prelisten-player', true);

init();