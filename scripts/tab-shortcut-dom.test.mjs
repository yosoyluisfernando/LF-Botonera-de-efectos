import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, existsSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
const root=fileURLToPath(new URL('../',import.meta.url));
const browser=process.env.LF_TEST_BROWSER || 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe';
const body=name=>readFileSync(join(root,'src/js',name),'utf8').replace(/^import[\s\S]*?from ['"][^'"]+['"];\r?\n/gm,'').replace(/export /g,'');
for(const [width,height] of [[1366,900],[640,360]])
test(`context shortcut and MIDI availability at ${width}x${height}`,{skip:!existsSync(browser)},()=>{
    const folder=mkdtempSync(join(tmpdir(),'lf-tab-shortcut-'));
    let html=readFileSync(join(root,'src/index.html'),'utf8').replace(/<script\b[^>]*>[\s\S]*?<\/script>/g,'')
        .replace(/href="(\/css\/[^"?]+)(?:\?[^" ]*)?"/g,(_,path)=>`href="${pathToFileURL(join(root,'src',path.slice(1))).href}"`);
    const locale=readFileSync(join(root,'src/public/i18n/es.json'),'utf8');
    const script=`
      const dictionary=${locale};const t=key=>key.split('.').reduce((v,k)=>v?.[k],dictionary)??key;
      let available=false,pending=null;const calls=[],saved=[],events={};
      const binding={device_id:'controller',device_name:'Control',message:'note',channel:1,number:42};
      const cfg={active_profile_id:'p',profiles:[{id:'p',active_paleta_id:'a',paletas:[
        {id:'a',nombre:'Principal',rows:5,cols:5,shortcut:'',midi:{}},
        {id:'b',nombre:'Cortinas',rows:4,cols:6,shortcut:'F7',midi:binding}]}]};
      let conflict=false;
      const invoke=async(cmd,args)=>{
        calls.push({cmd,args});
        if(cmd==='get_shortcut_editor_state')return {keyboard_available:true,midi_available:available};
        if(cmd==='update_paleta_meta'){
          if(conflict && !args.replaceShortcut)throw 'shortcut_conflict_tab|F8|Principal';
          saved.push(args);return cfg;
        }
        if(cmd==='midi_capture_next')return new Promise((resolve,reject)=>{pending={resolve,reject};});
        if(cmd==='midi_capture_cancel'){pending?.reject('midi_capture_cancelled');pending=null;return;}
        throw Error('Unexpected IPC '+cmd);
      };
      const listen=async(name,fn)=>{events[name]=fn;return ()=>{};};
      const appAlert=async()=>{},appConfirm=async()=>true,initButtonSearch=()=>{},initTabWheel=()=>{},paintAdaptive=()=>{},confirmDelete=async()=> 'cancel';
      const placeMenu=el=>el.classList.remove('hidden');
      ${body('util/keyInputs.js')}
      ${body('ui/midiControls.js')}
      const midiDisplay=display;
      ${body('ui/shortcutSave.js')}
      ${body('ui/mapping.js')}
      ${body('ui/tabs.js').replace(/\b_onRefresh\b/g,'_tabsRefresh').replace(/\b_wired\b/g,'_tabsWired')}
      const check=(v,m)=>{if(!v)throw Error(m);};const wait=()=>new Promise(r=>setTimeout(r,20));
      const section=document.getElementById('capture-midi-section');
      const open=async()=>{
        document.querySelector('[data-paleta-id="b"]').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,cancelable:true}));
        document.getElementById('tab-menu-shortcut').click();await wait();
      };
      (async()=>{
        initMapping(()=>{});initTabs(cfg,()=>{});await open();
        check(!document.getElementById('capture-modal').classList.contains('hidden'),'shortcut modal did not open');
        check(document.getElementById('capture-target-name').textContent.includes('Cortinas'),'wrong contextual target');
        check(document.getElementById('capture-key-input').value==='F7','current keyboard shortcut missing');
        check(section.classList.contains('hidden') && !section.open,'unavailable MIDI visible');
        const key=document.getElementById('capture-key-input');
        key.dispatchEvent(new KeyboardEvent('keydown',{key:'F8',code:'F8',bubbles:true,cancelable:true}));
        conflict=true;document.getElementById('btn-save-capture').click();await wait();conflict=false;
        check(saved.at(-1).profileId==='p' && saved.at(-1).paletaId==='b' && saved.at(-1).shortcut==='F8','keyboard shortcut saved to wrong tab');
        check(saved.at(-1).replaceShortcut===true,'conflict confirmation not reused');
        check(saved.at(-1).midi.device_id==='controller','hidden existing MIDI binding was lost');
        check(cfg.profiles[0].active_paleta_id==='a','context shortcut activated another tab');
        available=true;await open();
        check(!section.classList.contains('hidden') && !section.open,'available MIDI not initially collapsed');
        section.open=true;await wait();
        const box=document.querySelector('#capture-modal .modal-content').getBoundingClientRect();
        const footer=document.querySelector('#capture-modal .modal-footer').getBoundingClientRect();
        check(box.top>=0 && box.bottom<=innerHeight+1 && footer.bottom<=innerHeight+1,'expanded MIDI clipped the capture modal');
        document.getElementById('btn-capture-midi').click();await wait();
        check(hasActiveMidiCapture(),'capture did not start');
        available=false;events['midi-devices-changed']();await wait();
        check(section.classList.contains('hidden') && !section.open && !hasActiveMidiCapture(),'disconnect did not hide and cancel capture');
        available=true;events['midi-devices-changed']();await wait();
        check(!section.classList.contains('hidden') && !section.open,'reconnect did not restore collapsed section');
        section.open=true;await wait();document.getElementById('btn-capture-midi').click();await wait();
        section.open=false;await wait();check(!hasActiveMidiCapture(),'folding MIDI left capture pending');
        section.open=true;await wait();document.getElementById('btn-capture-midi').click();await wait();
        pending.resolve({binding:{...binding,number:60}});pending=null;await wait();
        document.getElementById('btn-save-capture').click();await wait();
        check(saved.at(-1).midi.number===60 && saved.at(-1).paletaId==='b','captured MIDI not saved');
        await open();section.open=true;await wait();document.getElementById('btn-capture-midi').click();await wait();
        document.getElementById('btn-cancel-capture').click();await wait();
        check(!hasActiveMidiCapture() && document.getElementById('capture-modal').classList.contains('hidden'),'cancel left MIDI capture pending');
        check(saved.length===2,'cancel saved changes');
        check(!calls.some(c=>/play|stop_audio|set_active/.test(c.cmd)),'assignment changed audio or active tab');
        document.body.dataset.tabShortcutTest='passed';
      })().catch(e=>{document.body.dataset.tabShortcutTest='failed: '+e.message;});
    `;
    html=html.replace('</body>',`<script>addEventListener('error',e=>document.body.dataset.tabShortcutTest='failed: '+e.message);</script><script>${script}</script></body>`);
    const fixture=join(folder,'shortcut.html');writeFileSync(fixture,html);
    const result=spawnSync(browser,['--headless','--disable-gpu','--no-first-run','--no-default-browser-check',
        `--window-size=${width},${height}`,'--disable-background-networking','--allow-file-access-from-files',
        `--user-data-dir=${resolve(folder,'browser')}`,'--virtual-time-budget=4000','--dump-dom',pathToFileURL(fixture).href],
        {encoding:'utf8',timeout:45000,maxBuffer:3*1024*1024});
    assert.ifError(result.error);
    const outcome=result.stdout?.match(/data-tab-shortcut-test="([^"]*)"/)?.[1];
    assert.equal(outcome,'passed',outcome || result.stderr);
});
