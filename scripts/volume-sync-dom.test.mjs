import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, existsSync, mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
const root=fileURLToPath(new URL('../',import.meta.url));
const browser=process.env.LF_TEST_BROWSER||'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe';
const body=name=>readFileSync(join(root,'src/js/ui',name+'.js'),'utf8').replace(/^import .*;\r?\n/gm,'').replace(/export /g,'');
const module=(name,exports)=>`(()=>{${body(name)};return {${exports}};})()`;
const bootstrap=`
const backend=window.parent.backend||window.backend;
const invoke=(...args)=>backend.invoke(...args);
const listen=async(name,callback)=>{if(name==='volume-controls-changed')backend.listeners.push(callback);};
const dictionary=${readFileSync(join(root,'src/public/i18n/es.json'),'utf8')};
const t=key=>key.split('.').reduce((v,k)=>v?.[k],dictionary)??key;
const placeMenu=()=>{},appConfirm=async()=>true,activeMeterTransition=()=>20;
const sync=${module('volumeSync','registerVolumeView,initVolumeSync,refreshVolumeControls')};
const {registerVolumeView,initVolumeSync,refreshVolumeControls}=sync;
const slider=${module('volumeSlider','bindVolumeSlider,paintVolumeSlider')};
const {bindVolumeSlider,paintVolumeSlider}=slider;
const cf=${module('consoleFaders','wireConsoleFader,paintConsoleFaders')};
const {wireConsoleFader,paintConsoleFaders}=cf;
const consoleView=${module('consoleView','renderConsole,clearConsole')};
const master=${module('masterVolume','initMasterVolume')};
const player=${module('playerVolume','initPlayerVolume')};
window.controls={sync,consoleView,master,player};
master.initMasterVolume();player.initPlayerVolume();
`;
for(const theme of ['light','dark'])test(`shared volumes and separate view: ${theme}`,{skip:!existsSync(browser)},()=>{
const folder=mkdtempSync(join(tmpdir(),'lf-volume-sync-'));
try{
const base=readFileSync(join(root,'src/index.html'),'utf8').replace(/<script\b[^>]*>[\s\S]*?<\/script>/g,'')
.replace(/href="(\/css\/[^"?]+)(?:\?[^" ]*)?"/g,(_,path)=>`href="${pathToFileURL(join(root,'src',path.slice(1))).href}"`);
const script=`
const clone=v=>structuredClone(v);
const state={master:{volume:1,max:1,boost:false,remember:false},player_volume:1,
 faders:Object.fromEntries(['programa','efectos','panel','reproductor','cue'].map(b=>[b,{value:1,max:1}]))};
const calls=[];
window.backend={listeners:[],publish(){for(const callback of this.listeners)callback({payload:clone(state)});},
 async invoke(cmd,args){
 calls.push({cmd,args});
 if(cmd==='get_volume_controls'){if(this.deferRead){this.deferRead=false;const saved=clone(state);return new Promise(resolve=>{this.releaseRead=()=>resolve(saved);});}return clone(state);}
 if(cmd==='get_master_volume_state')return clone(state.master);
 if(cmd==='get_console_view'){
 const strip=bus=>({bus,fader:state.faders[bus].value,max:state.faders[bus].max,in_program:bus!=='cue'});
 return {strips:['efectos','panel','reproductor'].map(strip),program:strip('programa'),cue:strip('cue')};}
 const setMaster=value=>{state.master.volume=Math.min(value,state.master.max);state.faders.programa.value=state.master.volume;};
 if(cmd==='set_master_volume')setMaster(args.volume);
 else if(cmd==='set_master_volume_options'){
 Object.assign(state.master,args);state.master.max=args.boost?1.5:1;state.faders.programa.max=state.master.max;setMaster(state.master.volume);
 }else if(cmd==='player_set_volume'){
 state.player_volume=args.volume;state.faders.reproductor.value=args.volume;
 }else if(cmd==='set_bus_fader'){
 if(args.bus==='programa')setMaster(args.value);
 else{state.faders[args.bus].value=args.value;if(args.bus==='reproductor')state.player_volume=args.value;}
 }else throw Error('Unexpected IPC '+cmd);
 this.publish();return clone(state.master);
 }};
${bootstrap}
const assert=(v,m)=>{if(!v)throw Error(m);};
const settle=()=>new Promise(r=>setTimeout(r,30));
const fader=(doc,bus)=>[...doc.querySelectorAll('.fader')].find(el=>el.getAttribute('aria-label')===dictionary.console.bus[bus]);
const move=(el,value)=>{el.value=String(value);el.dispatchEvent(new Event('input'));el.dispatchEvent(new Event('change'));};
(async()=>{
 document.documentElement.dataset.theme='${theme}';
 await controls.sync.initVolumeSync();await controls.consoleView.renderConsole();
 document.getElementById('player-volume-btn').click();
 const frame=document.createElement('iframe');document.body.append(frame);
 frame.contentDocument.open();frame.contentDocument.write(${JSON.stringify(base)});frame.contentDocument.close();
 frame.contentWindow.eval(${JSON.stringify(bootstrap)});
 await frame.contentWindow.controls.sync.initVolumeSync();await frame.contentWindow.controls.consoleView.renderConsole();
 const remote=frame.contentDocument;
 const playerInput=document.getElementById('player-volume-slider');
 move(playerInput,35);await settle();
 assert(fader(document,'reproductor').value==='0.35'&&fader(remote,'reproductor').value==='0.35','player panel did not update both consoles');
 assert(fader(document,'reproductor').max==='1','player fader allows boost');
 move(fader(remote,'reproductor'),0.62);await settle();
 assert(playerInput.value==='62'&&document.getElementById('player-volume-value').textContent==='62%','console did not update player slider and readout');
 const masterInput=document.getElementById('master-volume-slider');
 move(masterInput,47);await settle();
 assert(fader(document,'programa').value==='0.47'&&fader(remote,'programa').value==='0.47','master panel did not update both consoles');
 move(fader(remote,'programa'),0.28);await settle();
 assert(masterInput.value==='28'&&document.getElementById('master-volume-readout').textContent==='28%','remote console did not update master panel');
 const toggleBoost=async()=>{
 document.getElementById('master-volume-btn').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,cancelable:true}));
 document.querySelectorAll('.master-volume-menu button')[1].click();await settle();};
 await toggleBoost();
 assert(fader(document,'programa').closest('.fader-track').classList.contains('has-boost'),'master boost segment missing');
 assert(fader(document,'programa').closest('.fader-track').querySelector('.fader-unity').textContent==='100%','unity guide missing');
 assert(masterInput.max==='150'&&fader(document,'programa').max==='1.5'&&fader(remote,'programa').max==='1.5','boost limit not synchronized');
 move(masterInput,140);await settle();
 assert(fader(remote,'programa').value==='1.4'&&fader(document,'programa').title==='140%','boosted master position not synchronized');
 assert(fader(remote,'reproductor').max==='1'&&fader(remote,'cue').max==='1','master option boosted another bus');
 await toggleBoost();
 assert(masterInput.max==='100'&&masterInput.value==='100'&&fader(remote,'programa').max==='1'&&fader(remote,'programa').value==='1','disabling boost did not clamp all master controls');
 for(const value of [0,1,25,80,90,100]) {
   move(masterInput,value);await settle();await toggleBoost();await toggleBoost();
   assert(masterInput.value===String(value)&&Number(fader(remote,'programa').value)===value/100,'disabling boost changed a value at or below 100%: '+value);
 }
 assert(!fader(document,'programa').closest('.fader-track').classList.contains('has-boost'),'boost color remained with boost disabled');
 for(const input of document.querySelectorAll('.fader')) {
   input.dispatchEvent(new PointerEvent('pointerdown'));
   input.value='0.37';input.dispatchEvent(new Event('input'));
   const track=input.closest('.fader-track');
   assert(track.classList.contains('show-value')&&track.querySelector('.fader-value').textContent==='37%','live percentage missing for '+input.getAttribute('aria-label'));
   assert(input.getAttribute('aria-valuetext')==='37%','accessible percentage stale');
   input.dispatchEvent(new PointerEvent('pointerup'));
 }
 await new Promise(r=>setTimeout(r,450));
 assert(!document.querySelector('.fader-track.show-value'),'percentage did not hide after drag');
 const keyboard=fader(document,'cue');
 keyboard.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowUp'}));
 keyboard.value='0.38';keyboard.dispatchEvent(new Event('input'));
 assert(keyboard.closest('.fader-track').querySelector('.fader-value').textContent==='38%','keyboard percentage missing');
 keyboard.dispatchEvent(new KeyboardEvent('keyup',{key:'ArrowUp'}));
 window.backend.deferRead=true;
 const oldRead=controls.sync.refreshVolumeControls();
 move(fader(remote,'programa'),0.56);await settle();
 window.backend.releaseRead();await oldRead;await settle();
 assert(masterInput.value==='56'&&fader(document,'programa').value==='0.56','old IPC response overwrote a newer volume event');
 const local=fader(document,'programa');
 local.dispatchEvent(new PointerEvent('pointerdown'));local.value='0.8';local.dispatchEvent(new Event('input'));await settle();
 move(fader(remote,'programa'),0.2);await settle();assert(local.value==='0.8','incoming state interrupted local drag');
 local.dispatchEvent(new PointerEvent('pointerup'));await settle();assert(local.value==='0.2','finishing drag did not restore canonical value');
 assert(!calls.some(c=>/play_|stop_|queue/.test(c.cmd)),'volume control changed transport');
 document.body.dataset.volumeTest='passed';
})().catch(e=>{document.body.dataset.volumeTest='failed: '+e.message;});
`;
const fixture=join(folder,'volume.html');writeFileSync(fixture,base.replace('</body>',`<script>${script}</script></body>`));
const result=spawnSync(browser,['--headless','--disable-gpu','--no-first-run','--no-default-browser-check',
'--disable-background-networking','--allow-file-access-from-files',`--user-data-dir=${resolve(folder,'browser')}`,
'--virtual-time-budget=8000','--dump-dom',pathToFileURL(fixture).href],{encoding:'utf8',timeout:45000,maxBuffer:4*1024*1024});
assert.ifError(result.error);const outcome=result.stdout?.match(/data-volume-test="([^"]*)"/)?.[1];
assert.equal(outcome,'passed',outcome||result.stderr);
}finally{rmSync(folder,{recursive:true,force:true});}
});
