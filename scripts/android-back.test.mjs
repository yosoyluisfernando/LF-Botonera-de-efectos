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
const code = filename => readFileSync(join(root, 'src/js/ui', filename), 'utf8')
    .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');

test('Android Back unwinds real UI surfaces and requires explicit confirmation to exit', {
    skip: !existsSync(browser),
}, () => {
    const folder = mkdtempSync(join(tmpdir(), 'lf-android-back-'));
    let html = readFileSync(join(root, 'src/index.html'), 'utf8')
        .replace(/<script\b[^>]*>[\s\S]*?<\/script>/g, '')
        .replace(/href="(\/css\/[^"?]+)(?:\?[^" ]*)?"/g,
            (_, path) => 'href="'+pathToFileURL(join(root, 'src', path.slice(1))).href+'"');
    const script = `
        const assert=(condition,message)=>{if(!condition)throw Error(message);};
        const t=key=>key, calls=[]; let android=true,capture=false,mapping=false,backHandler;
        const isAndroid=()=>android;
        const invoke=async command=>{calls.push(command);};
        const listen=async(name,callback)=>{backHandler=callback;return ()=>{};};
        const hasActiveMidiCapture=()=>capture;
        const cancelMidiCapture=async()=>{capture=false;calls.push('cancel-capture');};
        const isMapping=()=>mapping,exitMapping=()=>{mapping=false;calls.push('exit-mapping');};
        const toggleFixedPanel=async()=>{document.getElementById('fixed-panel').classList.add('hidden');calls.push('close-fixed');};
        ${code('appDialog.js')}
        ${code('androidBack.js')}
        (async()=>{
            document.body.dataset.platform='android';
            await initAndroidBack();await initAndroidBack();
            assert(calls.filter(x=>x==='android_back_attach').length===1,'native handler attached twice');
            const show=id=>{const el=document.getElementById(id);el.classList.remove('hidden');return el;};
            const hide=id=>document.getElementById(id)?.classList.add('hidden');
            const tick=()=>new Promise(resolve=>setTimeout(resolve,0));
            await handleAndroidBack();assert(!document.getElementById('app-dialog-modal'),'Back escaped loading screen');
            hide('loading-screen');show('app-section');hide('fixed-panel');
            const settings=show('settings-modal');settings.dataset.androidView='detail';
            const sections=document.createElement('button');sections.className='android-settings-back';
            sections.onclick=()=>{settings.dataset.androidView='sections';};settings.prepend(sections);
            settings.querySelector('[data-close]').onclick=()=>hide('settings-modal');
            await handleAndroidBack();assert(settings.dataset.androidView==='sections','settings detail did not return to sections');
            await handleAndroidBack();assert(settings.classList.contains('hidden'),'settings did not return to main');
            show('settings-modal');settings.dataset.androidView='detail';
            const editor=show('track-editor-modal');let halted=0;
            document.getElementById('te-close').onclick=()=>{halted++;hide('track-editor-modal');};
            const color=show('color-picker-modal');let resolved=0;
            document.getElementById('color-picker-cancel').onclick=()=>{resolved++;hide('color-picker-modal');};
            await handleAndroidBack();assert(resolved===1&&!editor.classList.contains('hidden'),'color picker skipped its cancellation handler');
            await handleAndroidBack();assert(halted===1&&!settings.classList.contains('hidden'),'track editor did not run cleanup');
            hide('settings-modal');
            const edit=show('edit-modal');document.getElementById('btn-cancel-edit').onclick=()=>hide('edit-modal');
            capture=true;await handleAndroidBack();assert(!capture&&!edit.classList.contains('hidden'),'capture Back closed editor');
            await handleAndroidBack();assert(edit.classList.contains('hidden'),'editor cancel was bypassed');
            const busy=document.createElement('div');busy.id='backup-modal';busy.className='modal-overlay';busy.style.zIndex='9999';
            busy.innerHTML='<button id="backup-modal-close" disabled>close</button>';document.body.append(busy);
            await handleAndroidBack();assert(!document.getElementById('app-dialog-modal'),'busy modal escaped to exit');busy.remove();
            const dialog=document.createElement('dialog');document.body.append(dialog);let disconnected=0;
            dialog.addEventListener('close',()=>disconnected++);dialog.showModal();
            await handleAndroidBack();await tick();assert(!dialog.open&&disconnected===1,'remote dialog close cleanup skipped');
            mapping=true;await handleAndroidBack();assert(!mapping,'mapping stayed active');
            show('fixed-panel');await handleAndroidBack();assert(calls.includes('close-fixed'),'fixed panel did not use persisted close');
            const first=handleAndroidBack();await tick();
            assert(!document.getElementById('app-dialog-modal').classList.contains('hidden'),'root Back did not ask');
            await handleAndroidBack();await first;
            assert(!calls.includes('android_exit_app'),'second Back exited instead of dismissing confirmation');
            const second=handleAndroidBack();await tick();
            document.querySelector('.app-dialog-ok').click();await second;
            assert(calls.filter(x=>x==='android_exit_app').length===1,'explicit confirmation did not exit exactly once');
            document.body.dataset.backTest='passed';
        })().catch(error=>{document.body.dataset.backTest='failed: '+error.message;});
    `;
    html=html.replace('</body>','<script>'+script+'</script></body>');
    const file=join(folder,'back.html');writeFileSync(file,html);
    const result=spawnSync(browser,['--headless','--disable-gpu','--no-first-run',
        '--no-default-browser-check','--disable-background-networking','--allow-file-access-from-files',
        '--user-data-dir='+join(folder,'browser'),'--window-size=390,640','--virtual-time-budget=5000',
        '--dump-dom',pathToFileURL(file).href],
    {encoding:'utf8',timeout:60000,maxBuffer:4*1024*1024,windowsHide:true});
    assert.ifError(result.error);
    const outcome=result.stdout?.match(/data-back-test="([^"]*)"/)?.[1];
    assert.equal(outcome,'passed',outcome||result.stderr||'No browser result');
});
