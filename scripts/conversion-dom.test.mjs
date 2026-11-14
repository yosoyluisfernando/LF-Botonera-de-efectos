import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, existsSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
const root = fileURLToPath(new URL('../', import.meta.url));
const browser = process.env.LF_TEST_BROWSER || 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe';
const body = name => readFileSync(join(root, 'src/js/ui', name), 'utf8')
    .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');
for (const width of [390, 1366]) test(`conversion progress survives hiding and redrawing at ${width}px`, {skip:!existsSync(browser)}, () => {
    const folder = mkdtempSync(join(tmpdir(), 'lf-conversion-dom-'));
    const dictionary = readFileSync(join(root, 'src/public/i18n/es.json'), 'utf8');
    const css = ['theme.css','main.css','modal.css','audioConversion.css'].map(name =>
        `<link rel="stylesheet" href="${pathToFileURL(join(root,'src/css',name)).href}">`).join('');
    const script = `
      const dictionary=${dictionary};
      const t=key=>key.split('.').reduce((value,part)=>value?.[part],dictionary) ?? key;
      const calls=[];const invoke=async command=>{calls.push(command);return [];};
      const showNotice=()=>{};
      ${body('conversionState.js')}
      ${body('conversionButtons.js')}
      ${body('conversionModal.js')}
      ${body('audioConversion.js')}
      const assert=(value,message)=>{if(!value)throw Error(message);};
      const fixture=()=>'<div class="grid-item" data-audio-path="sample.mp4"><span class="timer">3.0s</span></div><button class="fixed-panel-item" data-audio-path="sample.mp4"><span class="timer">3.0s</span></button>';
      document.getElementById('cells').innerHTML=fixture();
      (async()=>{
        await paintConversion({path:'sample.mp4',status:'queued',revision:1,percent:null});
        assert(document.querySelector('.conversion-text').textContent==='En cola…','queue state missing');
        await paintConversion({path:'sample.mp4',status:'converting',revision:2,percent:23});
        const modal=document.getElementById('conversion-modal');
        assert(!modal.classList.contains('hidden'),'conversion modal did not open');
        assert(document.querySelectorAll('.conversion-text').length===2,'grid and fixed buttons not updated');
        modal.querySelector('button').click();
        assert(modal.classList.contains('hidden'),'hide failed');
        await paintConversion({path:'sample.mp4',status:'converting',revision:3,percent:61});
        assert(modal.classList.contains('hidden'),'progress reopened hidden modal');
        assert(document.querySelector('.conversion-text').textContent==='Conversión 61%','percentage missing');
        document.getElementById('cells').innerHTML=fixture();paintConversionButtons();
        assert(document.querySelector('progress').value===61,'redraw lost progress');
        await paintConversion({path:'sample.mp4',status:'queued',revision:1,percent:null});
        assert(document.querySelector('.conversion-text').textContent==='Conversión 61%','old snapshot replaced progress');
        document.getElementById('conversion-launcher').click();
        assert(!modal.classList.contains('hidden'),'cannot reopen modal');
        const rect=modal.querySelector('.conversion-dialog').getBoundingClientRect();
        assert(rect.left>=0 && rect.right<=innerWidth+1,'modal overflows viewport');
        await paintConversion({path:'sample.mp4',status:'ready',revision:4,percent:100});
        assert(!document.querySelector('.conversion-status'),'ready badge remains');
        assert(document.querySelector('.timer').textContent==='3.0s','playback timer was modified');
        assert(!calls.some(cmd=>/cancel|stop/.test(cmd)),'hiding sent a transport command');
        await paintConversion({path:'bad<img>.aac',status:'failed',error:'audio_convert_failed',revision:5,percent:null});
        assert(!modal.querySelector('img'),'filename interpreted as HTML');
        document.body.dataset.conversionTest='passed';
      })().catch(error=>{document.body.dataset.conversionTest='failed: '+error.message;});
    `;
    const fixture = join(folder,'conversion.html');
    writeFileSync(fixture,`<!doctype html><html><head><meta charset="utf-8">${css}</head><body><div id="cells"></div><script>${script}</script></body></html>`);
    const result=spawnSync(browser,['--headless','--disable-gpu','--no-first-run','--no-default-browser-check',
        `--window-size=${width},768`,'--disable-background-networking','--allow-file-access-from-files',
        `--user-data-dir=${resolve(folder,'browser')}`,'--virtual-time-budget=2000','--dump-dom',pathToFileURL(fixture).href],
        {encoding:'utf8',timeout:45000,maxBuffer:2*1024*1024});
    assert.ifError(result.error);
    assert.equal(result.stdout?.match(/data-conversion-test="([^"]*)"/)?.[1],'passed',result.stderr);
});
