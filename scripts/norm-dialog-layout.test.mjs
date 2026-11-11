import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, existsSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = fileURLToPath(new URL('../', import.meta.url));
const browser = process.env.LF_TEST_BROWSER || (process.platform === 'win32'
    ? 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe' : '/usr/bin/google-chrome');
const source = readFileSync(join(root, 'src/js/ui/normConfig.js'), 'utf8')
    .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');
const numberInputs = readFileSync(join(root, 'src/js/util/numberInputs.js'), 'utf8')
    .replace(/export /g, '');
const locales = Object.fromEntries(['es','en','pt-BR','pt-PT'].map(language => [language,
    JSON.parse(readFileSync(join(root, 'src/public/i18n', language+'.json'), 'utf8'))]));

for (const [width, height] of [[1366,768], [800,480], [390,640]]) {
    test(`normalization options scroll within ${width}x${height} without losing actions`, {
        skip: !existsSync(browser),
    }, () => {
        const folder = mkdtempSync(join(tmpdir(), 'lf-norm-dialog-'));
        let html = readFileSync(join(root, 'src/index.html'), 'utf8')
            .replace(/<script\b[^>]*>[\s\S]*?<\/script>/g, '')
            .replace(/href="(\/css\/[^"?]+)(?:\?[^" ]*)?"/g,
                (_, path) => 'href="'+pathToFileURL(join(root, 'src', path.slice(1))).href+'"');
        const script = `
          const locales=${JSON.stringify(locales)};
          let language='es';
          const t=key=>key.split('.').reduce((value,part)=>value?.[part],locales[language]) ?? key;
          const invoke=async()=>({used_mb:0,max_mb:100,count:0});
          const norm=(()=>{${source};return {open,close};})();
          (()=>{${numberInputs};initNumberInputs();})();
          const assert=(condition,message)=>{if(!condition)throw Error(message);};
          (async()=>{
            for(const lang of Object.keys(locales)) {
              language=lang;
              for(const platform of ['desktop','android']) {
                document.body.dataset.platform=platform;
                for(const design of ['default','modern']) {
                  if(platform==='android' && design==='modern')continue;
                  if(design==='default')delete document.documentElement.dataset.desktopStyle;
                  else document.documentElement.dataset.desktopStyle=design;
                  for(const theme of ['light','dark']) {
                    document.documentElement.dataset.theme=theme;
                    norm.open({mode:'lufs'}, {enabled:true,detect_start:true,detect_end:true},
                      {firstTime:true,waveformCache:{max_mb:100,max_age_days:30}});
                    await new Promise(resolve=>setTimeout(resolve,0));
                    const modal=document.getElementById('norm-config-modal');
                    const content=modal.querySelector('.modal-content');
                    const body=modal.querySelector('.modal-body');
                    const header=modal.querySelector('.modal-header');
                    const footer=modal.querySelector('.modal-footer');
                    const r=content.getBoundingClientRect();
                    const context=lang+'/'+platform+'/'+design+'/'+theme;
                    const input=document.getElementById('nc-target');input.blur();
                    const value=input.value;
                    const wheel=new WheelEvent('wheel',{deltaY:100,bubbles:true,cancelable:true});
                    input.dispatchEvent(wheel);
                    assert(!wheel.defaultPrevented && input.value===value,context+': scrolling edits unfocused field');
                    input.focus();
                    const adjust=new WheelEvent('wheel',{deltaY:100,bubbles:true,cancelable:true});
                    input.dispatchEvent(adjust);
                    assert(adjust.defaultPrevented && input.value!==value,context+': focused adjustment broken');
                    input.blur();
                    assert(r.top>=-0.5 && r.bottom<=innerHeight+0.5,context+': clipped vertically');
                    assert(r.left>=-0.5 && r.right<=innerWidth+0.5,context+': clipped horizontally');
                    assert(body.clientHeight>0,context+': body has no usable height');
                    assert(body.scrollWidth<=body.clientWidth+1,context+': horizontal form overflow');
                    const beforeHeader=header.getBoundingClientRect().top;
                    const beforeFooter=footer.getBoundingClientRect().bottom;
                    body.scrollTop=body.scrollHeight;
                    const last=document.getElementById('nc-wave-clear').getBoundingClientRect();
                    const b=body.getBoundingClientRect();
                    assert(last.top>=b.top-1 && last.bottom<=b.bottom+1,context+': last option unreachable');
                    assert(header.getBoundingClientRect().top===beforeHeader,context+': header scrolled away');
                    assert(footer.getBoundingClientRect().bottom===beforeFooter,context+': actions scrolled away');
                    for(const id of ['nc-save','nc-cancel']) {
                      const action=document.getElementById(id).getBoundingClientRect();
                      assert(action.top>=0 && action.bottom<=innerHeight,context+': action outside screen');
                    }
                    assert(document.scrollingElement.scrollTop===0,context+': underlying page scrolled');
                    norm.close();
                  }
                }
              }
            }
            document.body.dataset.normLayout='passed';
          })().catch(error=>{document.body.dataset.normLayout='failed: '+error.message;});
        `;
        html = html.replace('</body>', '<script>'+script+'</script></body>');
        const file = join(folder, 'normalization.html');
        writeFileSync(file, html);
        const result = spawnSync(browser, ['--headless','--disable-gpu','--no-first-run',
            '--no-default-browser-check','--disable-background-networking','--allow-file-access-from-files',
            '--user-data-dir='+join(folder,'browser'),`--window-size=${width},${height}`,
            '--virtual-time-budget=5000','--dump-dom',pathToFileURL(file).href],
        {encoding:'utf8',timeout:60000,maxBuffer:4*1024*1024,windowsHide:true});
        assert.ifError(result.error);
        const outcome=result.stdout?.match(/data-norm-layout="([^"]*)"/)?.[1];
        assert.equal(outcome,'passed',outcome || result.stderr || 'No browser result');
    });
}
