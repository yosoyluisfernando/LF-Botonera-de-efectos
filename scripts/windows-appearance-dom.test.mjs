import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, existsSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { spawnSync } from 'node:child_process';

const edge = process.env.LF_TEST_BROWSER || 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe';
const root = fileURLToPath(new URL('../', import.meta.url));
const moduleBody = name => readFileSync(join(root, 'src/js/ui', name), 'utf8')
    .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');

test('Principal mounts and switches its designs in a real Windows browser DOM', {
    skip: process.platform !== 'win32' || !existsSync(edge),
}, () => {
    const folder = mkdtempSync(join(tmpdir(), 'lf-appearance-dom-'));
    let html = readFileSync(join(root, 'src/index.html'), 'utf8')
        .replace(/<script\b[^>]*>[\s\S]*?<\/script>/g, '')
        .replace(/href="(\/css\/[^"?]+)(?:\?[^" ]*)?"/g,
            (_, path) => `href="${pathToFileURL(join(root, 'src', path.slice(1))).href}"`);
    const locale = readFileSync(join(root, 'src/public/i18n/es.json'), 'utf8');
    const script = `
      const __LF_WINDOWS_THEMES__ = true;
      const dictionary = ${locale};
      const t = key => key.split('.').reduce((v,k) => v?.[k], dictionary) ?? key;
      const platformCapabilities = () => ({platform:'desktop'});
      const listen = async () => {};
      const invoke = async (cmd,args) => ({available:true,style:args?.style || 'default'});
      const appAlert = async message => { throw Error(message); };
      const isAndroid = () => false;
      const midiDisplay = () => '';
      ${readFileSync(join(root, 'src/js/util/colorAdapter.js'), 'utf8').replace(/export /g, '')}
      const {updateProfiles} = (() => { ${moduleBody('profiles.js')} return {updateProfiles}; })();
      const {updateTabs,updateTabPlayback} = (() => { ${moduleBody('tabs.js')} return {updateTabs,updateTabPlayback}; })();
      ${moduleBody('desktopAppearance.js')}
      ${moduleBody('settingsAppearance.js')}
      ${moduleBody('settingsPlayback.js')}
      const assert = (value,message) => { if(!value) throw Error(message); };
      (async () => {
        document.body.dataset.platform='desktop';
        document.getElementById('app-section').classList.remove('hidden');
        document.getElementById('loading-screen').classList.add('hidden');
        const config={active_profile_id:'profile1',profiles:[
          {id:'profile1',name:'Perfil 1',bg:'#008c3a',text:'#ffffff',active_paleta_id:'tab1',paletas:[
            {id:'tab1',nombre:'Principal',tab_bg:'#0088ff',tab_text:'#ffffff'},
            {id:'tab2',nombre:'Segunda',tab_bg:'#ff8800',tab_text:'#ffffff'}]},
          {id:'profile2',name:'Otro perfil',bg:'#111111',text:'#ffffff',paletas:[]}]};
        updateProfiles(config,()=>{});updateTabs(config,()=>{});
        const saved=JSON.stringify(config);
        const contrast=(a,b)=>{
          const lum=c=>c.match(/[\\d.]+/g).slice(0,3).map(Number).map(v=>{
            v/=255;return v<=0.04045?v/12.92:((v+0.055)/1.055)**2.4;
          }).reduce((sum,v,i)=>sum+v*[0.2126,0.7152,0.0722][i],0);
          const x=lum(a)+0.05,y=lum(b)+0.05;return Math.max(x,y)/Math.min(x,y);
        };
        document.documentElement.dataset.theme='dark';
        document.getElementById('master-volume-panel').classList.remove('hidden');
        const original = ['config-theme','config-language','config-button-text-size',
          'config-show-console','config-show-fixed-panel','config-out-main','config-out-pre','player-device']
          .map(id => document.getElementById(id));
        await initializeDesktopAppearance();
        initPlaybackPanel();
        initSettingsAppearance();
        const panel=document.getElementById('s-main');
        assert(panel.querySelectorAll('fieldset').length===3,'Principal groups missing');
        assert(original.every(node=>panel.contains(node)),'An existing control was lost');
        const select=document.getElementById('config-desktop-style');
        assert(select && select.options.length===2,'Design list missing');
        assert(getComputedStyle(panel).display==='grid','Desktop layout not applied');
        panel.classList.add('hidden');
        assert(getComputedStyle(panel).display==='none','Settings navigation broken');
        panel.classList.remove('hidden');
        const counter=document.createElement('span');counter.className='te-time-text';panel.append(counter);
        for(const mode of ['dark','light']) {
          document.documentElement.dataset.theme=mode;
          select.value='modern';select.dispatchEvent(new Event('change'));
          await new Promise(resolve=>setTimeout(resolve,0));
          assert(document.documentElement.dataset.desktopStyle==='modern','Selection not applied');
          assert(!select.disabled,'Selection stayed disabled');
          const color=getComputedStyle(counter).backgroundColor;
          assert(color===(mode==='dark'?'rgb(124, 58, 237)':'rgb(109, 40, 217)'), 'Counter contrast missing');
          const profile=document.getElementById('btn-profiles');
          const activeTab=document.querySelector('#tabs-list .active');
          const modeButton=getComputedStyle(document.getElementById('pb-btn-normal'));
          for(const node of [profile,activeTab]) {
            assert(getComputedStyle(node).backgroundImage===modeButton.backgroundImage,'Header accent differs');
            assert(getComputedStyle(node).color==='rgb(255, 255, 255)','Header text lost contrast');
          }
          const menu=document.getElementById('profile-context-menu');menu.classList.remove('hidden');
          for(const li of menu.querySelectorAll('#profile-list li, .danger')) {
            assert(contrast(getComputedStyle(li).color,getComputedStyle(document.body).backgroundColor)>=4.5,
              'Saved profile name unreadable');
          }
          const vu=document.getElementById('vu-meter').getBoundingClientRect();
          const controls=document.getElementById('playback-modes').getBoundingClientRect();
          const bar=document.getElementById('bottom-bar').getBoundingClientRect();
          assert(controls.top-vu.bottom>=7.5,'VU collides with lower buttons');
          assert(controls.bottom<=bar.bottom-8,'Lower buttons clipped');
          updateTabPlayback({buttons:[{id:'tab2_btn_0'}]});
          const playing=document.querySelector('.tab-playing');
          assert(getComputedStyle(playing).backgroundImage==='none','Playing tab lost its signal color');
          assert(getComputedStyle(playing).color==='rgb(255, 255, 255)','Playing tab text changed');
          updateTabPlayback({buttons:[]});
          assert(JSON.stringify(config)===saved,'Personal colors were mutated');
          select.value='default';select.dispatchEvent(new Event('change'));
          await new Promise(resolve=>setTimeout(resolve,0));
          assert(!document.documentElement.dataset.desktopStyle,'Default not restored');
          assert(getComputedStyle(profile).backgroundColor==='rgb(0, 140, 58)','Default profile color changed');
        }
        document.body.dataset.appearanceTest='passed';
      })().catch(error=>{document.body.dataset.appearanceTest='failed: '+error.message;});
    `;
    html = html.replace('</body>', `<script>${script}</script></body>`);
    const fixture = join(folder, 'principal.html');
    writeFileSync(fixture, html);
    const result = spawnSync(edge, ['--headless', '--disable-gpu', '--no-first-run', '--window-size=1366,768',
        '--no-default-browser-check', '--disable-background-networking', '--allow-file-access-from-files',
        `--user-data-dir=${resolve(folder, 'browser')}`, '--virtual-time-budget=3000', '--dump-dom',
        pathToFileURL(fixture).href], { encoding:'utf8', timeout:45000, maxBuffer:4*1024*1024 });
    assert.ifError(result.error);
    const outcome=result.stdout?.match(/data-appearance-test="([^"]*)"/)?.[1];
    assert.equal(outcome, 'passed', outcome || result.stderr || 'No browser result');
});
