import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, existsSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
const root=fileURLToPath(new URL('../',import.meta.url));
const browser=process.env.LF_TEST_BROWSER || 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe';
const body=name=>readFileSync(join(root,'src/js/ui',name),'utf8').replace(/^import .*;\r?\n/gm,'').replace(/export /g,'');
for(const width of [768,1366]) test(`button finder left click and fixed search right click at ${width}px`,{skip:!existsSync(browser)},()=>{
    const folder=mkdtempSync(join(tmpdir(),'lf-button-search-'));
    let html=readFileSync(join(root,'src/index.html'),'utf8')
        .replace(/<script\b[^>]*>[\s\S]*?<\/script>/g,'')
        .replace(/href="(\/css\/[^"?]+)(?:\?[^" ]*)?"/g,(_,path)=>`href="${pathToFileURL(join(root,'src',path.slice(1))).href}"`);
    const locale=readFileSync(join(root,'src/public/i18n/es.json'),'utf8');
    const script=`
      const dictionary=${locale};const t=key=>key.split('.').reduce((v,k)=>v?.[k],dictionary)??key;
      const isAndroid=()=>false,placeMenu=()=>{},alertIpcError=e=>{throw e;};
      const calls=[];let refreshes=0;
      const cfg={active_profile_id:'profile',profiles:[{id:'profile',active_paleta_id:'a'}]};
      let settings={scope:'profile',view:'buttons',visible:false,side:'left',columns:3,row_mode:'limited',rows:4,width:360,modes_position:'bottom'};
      const hit={paleta_id:'b',paleta_nombre:'Cortinas',tab_position:2,button_id:'b_btn_5',index:5,row:1,col:5,label:'Aplausos',file_name:'aplausos'};
      const invoke=async(cmd,args)=>{
        calls.push({cmd,args});
        if(cmd==='search_buttons')return [hit];
        if(cmd==='set_active_paleta'){cfg.profiles[0].active_paleta_id=args.paletaId;return;}
        if(cmd==='get_fixed_panel')return {settings:{...settings},buttons:[]};
        if(cmd==='set_fixed_panel_settings'){
          settings={...settings,view:args.view,visible:args.visible??settings.visible};
          return {settings:{...settings},buttons:[]};
        }
        throw Error('Unexpected IPC: '+cmd);
      };
      ${body('fixedPanelViews.js')}
      ${body('buttonSearch.js')}
      const assert=(v,m)=>{if(!v)throw Error(m);};const wait=ms=>new Promise(r=>setTimeout(r,ms));
      document.getElementById('loading-screen').classList.add('hidden');
      document.getElementById('app-section').classList.remove('hidden');
      initFixedPanelViews(state=>{
        refreshFixedPanelView(state.settings);
        document.getElementById('fixed-panel').classList.toggle('hidden',!state.settings.visible);
        document.getElementById('fixed-panel').dataset.view=state.settings.view;
      });
      initButtonSearch(()=>cfg,async()=>{
        refreshes++;document.getElementById('main-grid').innerHTML='<div class="grid-item" data-id="b_btn_5">Aplausos</div>';
      });
      (async()=>{
        const button=document.getElementById('btn-button-search');button.click();
        const popover=document.querySelector('.button-search-panel');
        assert(!popover.classList.contains('hidden'),'left click did not open Danilo finder');
        assert(!calls.some(c=>c.cmd==='set_fixed_panel_settings'),'left click opened fixed panel');
        const input=popover.querySelector('input');input.value='aplausos';input.dispatchEvent(new Event('input'));
        await wait(120);
        assert(popover.querySelector('.button-search-row .name').textContent==='Aplausos','result missing');
        input.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true}));await wait(20);
        assert(cfg.profiles[0].active_paleta_id==='b' && refreshes===1,'result did not navigate');
        assert(document.querySelector('[data-id="b_btn_5"]').classList.contains('search-found'),'result not highlighted');
        const right=async()=>{
          const event=new MouseEvent('contextmenu',{bubbles:true,cancelable:true});button.dispatchEvent(event);
          await wait(20);assert(event.defaultPrevented,'native context menu not prevented');
        };
        for(const initial of [{visible:false,view:'buttons'},{visible:false,view:'search'},{visible:true,view:'player'},{visible:true,view:'buttons'},{visible:true,view:'search'}]){
          settings={...settings,...initial};refreshFixedPanelView(settings);button.click();
          await right();
          assert(settings.visible && settings.view==='search','right click did not open search');
          assert(popover.classList.contains('hidden'),'right click left button finder open');
          const args=calls.filter(c=>c.cmd==='set_fixed_panel_settings').at(-1).args;
          assert(args.scope==='profile' && args.side==='left' && args.columns===3 && args.rows===4 && args.width===360 && args.rowMode==='limited' && args.modesPosition==='bottom','panel preferences changed');
          assert(document.activeElement.id==='library-search-input','catalog input not focused');
        }
        assert(refreshes===1,'right click changed active palette');
        assert(!calls.some(c=>/play|stop|pause|clear|queue/.test(c.cmd)),'search changed audio transport');
        document.body.dataset.buttonSearchTest='passed';
      })().catch(e=>{document.body.dataset.buttonSearchTest='failed: '+e.message;});
    `;
    html=html.replace('</body>',`<script>${script}</script></body>`);
    const fixture=join(folder,'search.html');writeFileSync(fixture,html);
    const result=spawnSync(browser,['--headless','--disable-gpu','--no-first-run','--no-default-browser-check',
        `--window-size=${width},768`,'--disable-background-networking','--allow-file-access-from-files',
        `--user-data-dir=${resolve(folder,'browser')}`,'--virtual-time-budget=4000','--dump-dom',pathToFileURL(fixture).href],
        {encoding:'utf8',timeout:45000,maxBuffer:3*1024*1024});
    assert.ifError(result.error);
    const outcome=result.stdout?.match(/data-button-search-test="([^"]*)"/)?.[1];
    assert.equal(outcome,'passed',outcome || result.stderr);
});
