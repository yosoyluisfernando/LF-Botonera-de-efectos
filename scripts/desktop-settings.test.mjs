import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, existsSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { spawnSync } from 'node:child_process';
import vm from 'node:vm';

const root = fileURLToPath(new URL('../', import.meta.url));
const source = name => readFileSync(join(root, 'src/js/ui', name), 'utf8')
    .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');
const browser = process.env.LF_TEST_BROWSER || (process.platform === 'win32'
    ? 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe' : '/usr/bin/google-chrome');

test('About uses backend platform/channel and never invents GitHub on IPC failure', async () => {
    const elements = {};
    let info;
    const context = vm.createContext({
        document: { getElementById: id => elements[id] ||= {} },
        t: key => key,
        invoke: async () => { if (!info) throw Error('offline'); return info; },
    });
    vm.runInContext(source('aboutInfo.js'), context);
    for (const [platform, channel, updateManager] of [
        ['windows', 'direct', 'github_releases'], ['windows', 'store', 'microsoft_store'],
        ['linux', 'direct', 'github_releases'],
    ]) {
        info = { version: '1.6.0-beta.5', platform, channel, updateManager };
        await vm.runInContext('loadAboutInfo()', context);
        assert.equal(elements['app-platform'].textContent, 'settings.about_platform_' + platform);
        assert.equal(elements['app-update-manager'].textContent, 'settings.about_updates_' + updateManager);
    }
    info = null;
    await vm.runInContext('loadAboutInfo()', context);
    for (const element of Object.values(elements))
        assert.equal(element.textContent, 'settings.about_info_unavailable');
});

test('desktop About remains last and Remote controls recover from status failures', {
    skip: !existsSync(browser),
}, () => {
    const folder = mkdtempSync(join(tmpdir(), 'lf-desktop-settings-'));
    let html = readFileSync(join(root, 'src/index.html'), 'utf8')
        .replace(/<script\b[^>]*>[\s\S]*?<\/script>/g, '')
        .replace('href="/css/settingsModal.css"', 'href="' + pathToFileURL(join(root, 'src/css/settingsModal.css')).href + '"');
    const script = `
      const t=key=>key,isAndroid=()=>false;
      const initObsAppearance=()=>{},loadObsAppearance=()=>{},obsAppearance=()=>({}),applyObsAppearance=()=>{};
      const mountClient=()=>()=>{};
      const platformCapabilities=()=>({midi:true}),listen=async()=>()=>{};
      const initMidiField=()=>{},setMidiField=()=>{},getMidiField=()=>({}),appAlert=async()=>{};
      let available=false,enabled=false;
      const invoke=async(command)=>{
        if(!available)throw Error('transport_unavailable');
        if(command==='remote_start')enabled=true;
        if(command==='remote_stop')enabled=false;
        return {enabled,url:'http://192.168.1.5:17865',token:'test',suggested_ip:'192.168.1.5'};
      };
      ${source('settingsObs.js')}
      ${source('settingsRemote.js')}
      ${source('settingsPlayback.js')}
      ${source('settingsMidi.js')}
      const assert=(condition,message)=>{if(!condition)throw Error(message);};
      const tick=()=>new Promise(resolve=>setTimeout(resolve,10));
      (async()=>{
        initPlaybackPanel();initSettingsObs();initSettingsRemote({});await tick();
        initSettingsMidi();
        const discovery=document.getElementById('android-midi-add');
        assert(!discovery.open,'desktop alternative Bluetooth section opens by default');
        assert(discovery.querySelector('summary').textContent==='midi.bluetooth_alternative','desktop pairing flow unclear');
        assert(document.querySelector('[data-i18n="midi.desktop_pairing_hint"]'),'missing system pairing guidance');
        assert(!document.getElementById('config-midi-refresh').classList.contains('hidden'),'native refresh hidden');
        assert(document.querySelector('.settings-tabs').lastElementChild.dataset.target==='s-about','About not last');
        const warning=document.querySelector('#s-remote > [data-i18n="remote.encryption_warning"]');
        assert(warning && !warning.hidden && warning===warning.parentElement.lastElementChild,'unencrypted warning hidden when inactive');
        const toggle=document.getElementById('remote-host-enabled'),options=document.getElementById('remote-host-options');
        assert(!toggle.checked && options.disabled && options.inert,'remote controls active while off');
        assert(getComputedStyle(options).opacity==='0.45','disabled section is not dimmed like preload');
        assert(!toggle.disabled,'failure permanently disabled checkbox');
        toggle.checked=true;toggle.dispatchEvent(new Event('change'));
        assert(!options.disabled && !options.inert,'remote options stay locked after checking');
        document.getElementById('remote-ip').value='192.168.1.5';
        available=true;await saveSettingsRemote();
        assert(toggle.checked&&enabled,'saving did not activate server');
        available=false;toggle.checked=false;toggle.dispatchEvent(new Event('change'));await tick();
        assert(!toggle.disabled && toggle.checked,'failed stop did not restore server state');
        available=true;toggle.checked=false;toggle.dispatchEvent(new Event('change'));await tick();
        assert(!enabled && options.disabled && options.inert,'unchecking did not stop server');
        const obs=document.getElementById('obs-enabled'),obsOptions=document.getElementById('obs-options');
        assert(obsOptions.disabled && obsOptions.inert,'OBS options active while off');
        obs.checked=true;obs.dispatchEvent(new Event('change'));
        assert(!obsOptions.disabled&&!obsOptions.inert,'OBS options remain blocked');
        const progress=document.getElementById('playback-progress-enabled'),seek=document.getElementById('playback-progress-options');
        assert(seek.disabled&&seek.inert,'seek options active while off');
        progress.checked=true;progress.dispatchEvent(new Event('change'));
        assert(!seek.disabled&&!seek.inert,'seek options remain blocked');
        available=false;
        document.getElementById('obs-test').click();await tick();
        assert(!document.getElementById('obs-test').disabled,'OBS test button locked');
        assert(document.getElementById('obs-test-result').textContent==='errors.obs_connection_failed','unknown OBS error exposed');
        document.body.dataset.result='passed';
      })().catch(error=>{document.body.dataset.result=error.message;});
    `;
    html += '<script>' + script.replace(/<\/script/gi, '<\\/script') + '</script>';
    const file = join(folder, 'settings.html');
    writeFileSync(file, html);
    const result = spawnSync(browser, ['--headless', '--disable-gpu', '--no-first-run',
        '--no-default-browser-check', '--disable-background-networking',
        '--user-data-dir=' + join(folder, 'browser'), '--virtual-time-budget=3000',
        '--dump-dom', pathToFileURL(file).href],
    { encoding: 'utf8', timeout: 60000, maxBuffer: 2 * 1024 * 1024, windowsHide: true });
    assert.ifError(result.error);
    assert.equal(result.stdout?.match(/data-result="([^"]*)"/)?.[1], 'passed');
});

test('remote client defaults off and switching it off cancels polling immediately', {
    skip: !existsSync(browser),
}, () => {
    const folder = mkdtempSync(join(tmpdir(), 'lf-remote-off-'));
    const client = readFileSync(join(root, 'src/js/remote/client.js'), 'utf8')
        .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');
    const script = `
      const t=key=>key,isAndroid=()=>true,renderSnapshot=()=>{};
      let requests=0;
      const invoke=async()=>{requests++;return {};};
      ${client}
      ${source('settingsRemote.js')}
      const assert=(value,message)=>{if(!value)throw Error(message);};
      const tick=ms=>new Promise(resolve=>setTimeout(resolve,ms));
      (async()=>{
        initSettingsRemote({});
        const enabled=document.getElementById('remote-client-enabled');
        const shortcut=document.getElementById('btn-remote-mirror');
        assert(!enabled.checked && shortcut.classList.contains('hidden'),'remote defaults on');
        await tick(500);assert(requests===0,'disabled client sends requests');
        enabled.checked=true;enabled.dispatchEvent(new Event('change'));
        shortcut.click();
        const form=document.querySelector('.remote-login');
        form.elements.address.value='192.168.1.5:17865';form.elements.token.value='key';
        form.dispatchEvent(new Event('submit',{cancelable:true}));await tick(10);
        assert(requests===1,'connection did not start');
        enabled.checked=false;enabled.dispatchEvent(new Event('change'));
        assert(!document.querySelector('.remote-dialog').open,'disable left dialog open');
        const count=requests;await tick(700);
        assert(requests===count,'disabled client kept polling');
        assert(shortcut.classList.contains('hidden'),'disabled shortcut remains visible');
        assert(document.querySelector('#s-remote').lastElementChild.dataset.i18n==='remote.encryption_warning','warning not last');
        document.body.dataset.result='passed';
      })().catch(error=>{document.body.dataset.result=error.message;});
    `;
    const html = '<!doctype html><meta charset="utf-8"><div class="settings-tabs"><button data-target="s-about"></button></div>'
        + '<div class="settings-content"></div><button id="btn-settings"></button><script>'
        + script.replace(/<\/script/gi, '<\\/script') + '</script>';
    const file = join(folder, 'remote.html');writeFileSync(file, html);
    const result = spawnSync(browser, ['--headless', '--disable-gpu', '--no-first-run',
        '--no-default-browser-check', '--disable-background-networking',
        '--user-data-dir=' + join(folder, 'browser'), '--virtual-time-budget=3000',
        '--dump-dom', pathToFileURL(file).href],
    { encoding: 'utf8', timeout: 60000, maxBuffer: 2 * 1024 * 1024, windowsHide: true });
    assert.ifError(result.error);
    assert.equal(result.stdout?.match(/data-result="([^"]*)"/)?.[1], 'passed');
});
