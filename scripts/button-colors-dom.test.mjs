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
for(const [width,height] of [[1366,900],[900,600],[640,420],[360,640],[568,320]])
for(const theme of ['dark','light']) test(`color editor ${width}x${height} in ${theme}`,{skip:!existsSync(browser)},()=>{
    const folder=mkdtempSync(join(tmpdir(),'lf-button-colors-'));
    let html=readFileSync(join(root,'src/index.html'),'utf8')
        .replace(/<script\b[^>]*>[\s\S]*?<\/script>/g,'')
        .replace(/href="(\/css\/[^"?]+)(?:\?[^" ]*)?"/g,(_,path)=>`href="${pathToFileURL(join(root,'src',path.slice(1))).href}"`);
    const locale=readFileSync(join(root,'src/public/i18n/es.json'),'utf8');
    const script=`
      const dictionary=${locale};const t=key=>key.split('.').reduce((v,k)=>v?.[k],dictionary)??key;
      const createSvgVisual=()=>null,typeIcon=()=>'',currentEditVisual=()=>({kind:'auto',mode:'text'});
      const colorBg=c=>c.base,colorText=c=>c.darkText,_theme=()=>document.documentElement.dataset.theme;
      const palette=[{base:'#DB2424',darkText:'#FFFFFF'},{base:'#3324DB',darkText:'#FFFFFF'}];
      let savedFavorites=[];const calls=[],writes=[];const invoke=async(cmd,args)=>{
        calls.push(cmd);if(cmd==='get_color_palette')return palette;
        if(cmd==='get_color_favorites')return savedFavorites;
        if(cmd==='save_color_favorite'){savedFavorites.push(structuredClone(args.favorite));return savedFavorites;}
        if(cmd==='remove_color_favorite'){savedFavorites=savedFavorites.filter(f=>f.name!==args.name);return savedFavorites;}
        if(cmd==='pick_audio_file')return 'C:/Music/otro.mp4';
        if(cmd==='get_grid_state')return {obs:false};if(cmd==='get_edit_button_types')return {};
        if(cmd==='update_button_data'||cmd==='update_fixed_button'){writes.push({cmd,args});return {};}
        throw Error(cmd);
      };
      const setObsEditor=()=>{},openObsEditor=()=>{},prepareEditShortcutSections=async()=>{},
        setTypeState=()=>{},syncTypeOptions=()=>{},selectedType=()=> 'audio',isFolderType=()=>false,
        isLocution=()=>false,defaultFolder=()=>'',placeholderKey=()=>'',labelKey=()=>'',
        setEditVolumeFromLinear=()=>{},syncEditVolumeControl=()=>{},currentEditVolumeLinear=()=>1,
        canPrelisten=()=>true,initEditVisual=()=>{},refreshEditVisualPreview=()=>{},
        initMidiField=()=>{},setMidiField=()=>{},getMidiField=()=>({});
      const alerts=[];const appAlert=async text=>alerts.push(text);
      const appConfirm=async()=>true;const alertIpcError=e=>{throw e;};
      const invokeShortcutSave=(cmd,args)=>invoke(cmd,args);
      ${body('util/colorAdapter.js')}
      ${body('ui/colorPalette.js')}
      ${body('ui/buttonVisual.js')}
      ${body('ui/buttonStyle.js')}
      ${body('ui/colorFavorites.js').replace(/\bwired\b/g,'favoritesWired')}
      ${body('ui/editStyle.js')}
      ${body('ui/editModal.js')}
      const assert=(v,m)=>{if(!v)throw Error(m);};const wait=()=>new Promise(r=>setTimeout(r,20));
      const set=(id,value,event='input')=>{const e=document.getElementById(id);e.value=value;e.dispatchEvent(new Event(event,{bubbles:true}));};
      const visible=id=>!document.getElementById(id).classList.contains('hidden');
      (async()=>{
        document.documentElement.dataset.theme='${theme}';
        document.getElementById('edit-modal').classList.remove('hidden');
        document.getElementById('edit-bg-color').value='#DB2424';document.getElementById('edit-text-color').value='#FFFFFF';
        document.getElementById('edit-name').value='Aplausos';
        initColorPicker();attachPalette(document.getElementById('edit-bg-color'),document.getElementById('edit-text-color'));
        await refreshColorInputs();await initEditStyle(null);
        assert(document.getElementById('edit-fill').options[0].value==='preset','presets not first');
        assert(mode==='preset' && visible('edit-preset-colors') && !visible('edit-custom-colors'),'initial preset mode');
        const preview=document.getElementById('edit-style-preview');
        await openColorPicker(document.getElementById('edit-bg-color'),document.getElementById('edit-text-color'));
        document.querySelectorAll('.color-choice')[1].click();document.getElementById('color-picker-apply').click();
        assert(preview.dataset.baseBg.toUpperCase()==='#3324DB','palette did not update preview immediately');
        assert(Object.keys(currentEditStyle()).length===0,'presets wrote advanced style');
        set('edit-fill','solid','change');
        assert(!visible('edit-preset-colors') && visible('edit-custom-colors') && !visible('edit-second-bg'),'solid controls');
        set('edit-bg-hex','#f00');set('edit-text-hex','#00FF00');
        assert(getComputedStyle(preview).backgroundColor==='rgb(255, 0, 0)','solid HEX not exact');
        assert(getComputedStyle(preview).color==='rgb(0, 255, 0)','text HEX not exact');
        const previous=preview.style.backgroundColor;
        set('edit-bg-hex','#oops');assert(editStyleError()!==null,'invalid HEX allowed');
        assert(preview.style.backgroundColor===previous,'invalid HEX corrupted preview');
        set('edit-bg-hex','#FF0000');assert(editStyleError()===null,'valid HEX blocked');
        const actual=document.createElement('div');actual.className='grid-item';actual.innerHTML='<span class="label"><span class="button-visual-text">Aplausos</span></span>';document.body.append(actual);
        const fixed=document.createElement('button');fixed.className='fixed-panel-item';fixed.innerHTML='<span class="fixed-panel-name"><span class="button-visual-text">Aplausos</span></span>';document.body.append(fixed);
        for(const direction of ['horizontal','vertical','diagonal','diagonal_inv']){
          set('edit-fill',direction,'change');set('edit-custom-bg2','#0000ff');
          assert(visible('edit-second-bg'),'gradient second color hidden');
          assert(getComputedStyle(preview).backgroundImage.startsWith('linear-gradient('),'preview gradient absent');
          const data={color_bg:document.getElementById('edit-bg-color').value,color_text:document.getElementById('edit-text-color').value,style:currentEditStyle()};
          paintButtonStyle(actual,data);paintButtonStyle(fixed,data);
          assert(getComputedStyle(preview).backgroundImage===getComputedStyle(actual).backgroundImage,'grid differs from preview');
          assert(getComputedStyle(preview).backgroundImage===getComputedStyle(fixed).backgroundImage,'fixed differs from preview');
        }
        document.getElementById('edit-text-gradient').checked=true;document.getElementById('edit-text-gradient').dispatchEvent(new Event('change'));
        set('edit-text2-hex','#FFD700');
        assert(visible('edit-second-text') && preview.querySelector('.label').classList.contains('text-gradient'),'text gradient absent');
        assert(getComputedStyle(preview.querySelector('.button-visual-text')).backgroundImage.startsWith('linear-gradient('),'text gradient CSS absent');
        set('edit-name','Cambio inmediato');assert(preview.textContent==='Cambio inmediato','name not live');
        actual.classList.add('playing');assert(getComputedStyle(actual).backgroundImage==='none','gradient hides playback state');
        actual.classList.remove('playing');assert(getComputedStyle(actual).backgroundImage!=='none','gradient not restored after playback');
        set('edit-fill','preset','change');
        assert(getComputedStyle(preview).backgroundImage==='none' && !preview.querySelector('.label').classList.contains('text-gradient'),'old gradient survived preset');
        assert(document.getElementById('edit-bg-color').value==='#3324db','preset draft not restored');
        await initEditStyle({style:{gradient:'vertical',bg2:'#123456',text2:'#FFEEDD',exact:true}});
        assert(mode==='vertical' && visible('edit-second-bg') && visible('edit-second-text'),'saved mode not restored');
        await initEditStyle(null);assert(mode==='preset' && !visible('edit-custom-colors'),'next editor leaked previous style');
        assert(calls.every(c=>c==='get_color_palette'||c==='get_color_favorites'),'draft triggered mutation or audio IPC');
        const original={type:'audio',path:'aplausos.wav',label:'Original',color_bg:'#FF0000',color_text:'#FFFFFF',
          style:{gradient:'vertical',bg2:'#123456',text2:'#FFEEDD',exact:true}};
        for(const target of ['grid','fixed']){
          await openEditModal(1,original,()=>{},target);
          assert(mode==='vertical','editor did not load persisted style');
          set('edit-bg-hex','#no');await document.getElementById('btn-save-edit').onclick();
          assert(writes.length===(target==='grid'?0:1) && alerts.length>0,'invalid style reached backend');
          set('edit-bg-hex','#112233');set('edit-fill','horizontal','change');
          document.getElementById('btn-cancel-edit').click();
          assert(original.style.gradient==='vertical','cancel changed source object');
          await openEditModal(1,original,()=>{},target);assert(mode==='vertical','canceled style leaked');
          set('edit-fill','horizontal','change');set('edit-bg2-hex','#ABCDEF');
          await document.getElementById('btn-save-edit').onclick();
          const saved=writes.at(-1);
          assert(saved.cmd===(target==='grid'?'update_button_data':'update_fixed_button'),'wrong save command');
          assert(saved.args.style.gradient==='horizontal' && saved.args.style.bg2==='#abcdef' && saved.args.style.exact,'style missing from save IPC');
        }
        await openEditModal(1,original,()=>{},'grid');
        set('edit-bg-hex','#598042');set('edit-bg2-hex','#021927');
        set('edit-favorite-name','Verde y noche');document.getElementById('edit-favorite-add').click();await wait();
        assert(savedFavorites.length===1 && savedFavorites[0].style.gradient==='vertical','favorite not saved');
        set('edit-fill','solid','change');set('edit-bg-hex','#FF0000');
        set('edit-fill','favorites','change');set('edit-favorite-list','Verde y noche','change');await wait();
        assert(currentEditStyle().gradient==='vertical' && document.getElementById('edit-bg-color').value==='#598042','favorite not applied');
        assert(visible('edit-favorite-choices') && !visible('edit-custom-colors'),'favorites controls');
        document.getElementById('btn-cancel-edit').click();await openEditModal(1,original,()=>{},'grid');
        set('edit-fill','favorites','change');set('edit-favorite-list','Verde y noche','change');await wait();
        assert(document.getElementById('edit-bg-color').value==='#598042','favorite not available when reopening');
        const appearance=JSON.stringify(currentEditStyle()),bg=document.getElementById('edit-bg-color').value;
        await document.getElementById('btn-select-file').onclick();
        assert(document.getElementById('edit-filepath').value==='C:/Music/otro.mp4','new path not selected');
        assert(bg===document.getElementById('edit-bg-color').value && JSON.stringify(currentEditStyle())===appearance,'sound change changed colors');
        assert(writes.length===2,'choosing audio persisted a draft');
        assert(!calls.some(c=>c==='assign_file_to_button'||c==='assign_file_to_fixed_button'),'editor assigned audio before accepting');
        for(const type of ['random_folder','time','temperature','humidity','audio']) {
          const typeEl=document.getElementById('edit-type');typeEl.value=type;typeEl.onchange();
          assert(bg===document.getElementById('edit-bg-color').value && JSON.stringify(currentEditStyle())===appearance,'type change changed colors');
        }
        document.getElementById('edit-favorite-remove').click();await wait();
        assert(savedFavorites.length===0 && JSON.stringify(currentEditStyle())===appearance,'deleting favorite changed button draft');
        const box=document.querySelector('#edit-modal > .modal-content'),bodyEl=box.querySelector('.modal-body');
        const rect=box.getBoundingClientRect(),footer=box.querySelector('.modal-footer').getBoundingClientRect();
        assert(rect.top>=0 && rect.bottom<=innerHeight+1 && rect.left>=0 && rect.right<=innerWidth+1,'dialog outside viewport');
        assert(footer.bottom<=innerHeight+1 && footer.top>=0,'actions outside viewport');
        set('edit-fill','diagonal_inv','change');document.getElementById('edit-favorite-save').open=true;
        bodyEl.scrollTop=bodyEl.scrollHeight;await wait();
        assert(bodyEl.clientHeight>0 && bodyEl.scrollTop>0,'body cannot scroll to remaining controls');
        assert(box.querySelector('.modal-header').getBoundingClientRect().top>=0,'header scrolled away');
        document.body.dataset.buttonColorsTest='passed';
      })().catch(e=>{document.body.dataset.buttonColorsTest='failed: '+e.message;});
    `;
    html=html.replace('</body>',`<script>addEventListener('error',e=>document.body.dataset.buttonColorsTest='failed: '+e.message);</script><script>${script}</script></body>`);
    const fixture=join(folder,'colors.html');writeFileSync(fixture,html);
    const result=spawnSync(browser,['--headless','--disable-gpu','--no-first-run','--no-default-browser-check',
        `--window-size=${width},${height}`,'--disable-background-networking','--allow-file-access-from-files',
        `--user-data-dir=${resolve(folder,'browser')}`,'--virtual-time-budget=4000','--dump-dom',pathToFileURL(fixture).href],
        {encoding:'utf8',timeout:45000,maxBuffer:3*1024*1024});
    assert.ifError(result.error);
    const outcome=result.stdout?.match(/data-button-colors-test="([^"]*)"/)?.[1];
    assert.equal(outcome,'passed',outcome || result.stderr);
});
