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
const source = readFileSync(join(root, 'src/js/ui/settingsAndroid.js'), 'utf8')
    .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');
const keyboards = readFileSync(join(root, 'src/js/ui/settingsKeyboardDevices.js'), 'utf8')
    .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');
const locales = Object.fromEntries(['es', 'en', 'pt-BR', 'pt-PT'].map(language => [language,
    JSON.parse(readFileSync(join(root, 'src/public/i18n', language + '.json'), 'utf8'))]));

for (const platform of ['desktop', 'android']) {
    test(`${platform} MIDI settings keep keyboard controls accessible and preserve device nodes`, {
        skip: !existsSync(browser),
    }, () => {
        const folder = mkdtempSync(join(tmpdir(), 'lf-midi-settings-'));
        let html = readFileSync(join(root, 'src/index.html'), 'utf8')
            .replace(/<script\b[^>]*>[\s\S]*?<\/script>/g, '')
            .replace(/href="(\/css\/[^"?]+)(?:\?[^" ]*)?"/g,
                (_, path) => 'href="' + pathToFileURL(join(root, 'src', path.slice(1))).href + '"');
        const script = `
            const platform=${JSON.stringify(platform)}, locales=${JSON.stringify(locales)};
            let language='es';
            const isAndroid=()=>platform==='android';
            const t=key=>key.split('.').reduce((value,part)=>value?.[part],locales[language])??key;
            let keyboardDevices=[], keyboardEvents, queryCount=0, settingsError=false, queryError=false;
            const invoke=async command=>{
                if(command==='keyboard_devices') {
                    queryCount++;if(queryError)throw Error('unavailable');return keyboardDevices;
                }
                if(command==='keyboard_open_bluetooth_settings') {
                    if(settingsError)throw Error('unavailable');return;
                }
                throw Error('Unexpected IPC '+command);
            };
            const listen=async(name,callback)=>{keyboardEvents=callback;return ()=>{};};
            ${keyboards}
            ${source}
            const assert=(condition,message)=>{if(!condition)throw Error(message);};
            (async()=>{
                document.body.dataset.platform=platform;
                const panel=document.getElementById('s-atajos'), original=panel.innerHTML;
                const button=document.getElementById('android-midi-bluetooth');
                let clicks=0;button.addEventListener('click',()=>clicks++);
                initAndroidSettingsNavigation();
                assert(document.getElementById('android-midi-bluetooth')===button,'device button replaced');
                button.click();assert(clicks===1,'device listener lost');
                if(platform==='desktop') {
                    assert(panel.innerHTML===original,'desktop settings changed');
                    assert(document.querySelector('[data-target="s-atajos"]').dataset.i18n==='settings.tab_keys','desktop title changed');
                } else {
                    const keyboard=document.getElementById('android-keyboard-shortcuts');
                    assert(!keyboard.open,'keyboard opens by default');
                    const physical=document.getElementById('android-keyboard-devices');
                    assert(keyboard.contains(physical),'physical keyboards outside disclosure');
                    assert(physical.nextElementSibling.id==='settings-keyboard-main','physical devices not before shortcut fields');
                    const keyboardList=document.getElementById('android-keyboard-device-list');
                    await new Promise(resolve=>setTimeout(resolve,0));
                    assert(queryCount===1,'physical devices not queried on init');
                    assert(keyboardList.textContent===t('keyboard.empty'),'empty keyboard state missing');
                    keyboardDevices=[{id:'kbd1',name:'Bluetooth <Keyboard & USB>'}];
                    window.dispatchEvent(new Event('focus'));
                    await new Promise(resolve=>setTimeout(resolve,0));
                    assert(keyboardList.textContent===keyboardDevices[0].name,'return to app did not refresh real keyboard name');
                    assert(keyboardList.children.length===1,'device name interpreted as HTML');
                    keyboardEvents({payload:[]});
                    assert(keyboardList.textContent===t('keyboard.empty'),'disconnected keyboards not cleared');
                    keyboardEvents({payload:[{id:'usb1',name:'USB Keyboard'}]});
                    assert(keyboardList.textContent==='USB Keyboard','keyboard hotplug not shown');
                    const pair=document.getElementById('android-keyboard-bluetooth');
                    pair.click();await new Promise(resolve=>setTimeout(resolve,0));
                    assert(!pair.disabled,'keyboard settings action stayed disabled');
                    settingsError=true;pair.click();await new Promise(resolve=>setTimeout(resolve,0));
                    assert(physical.querySelector('[role="status"]').textContent===t('keyboard.settings_unavailable'),'missing Bluetooth settings error');
                    queryError=true;window.dispatchEvent(new Event('focus'));
                    await new Promise(resolve=>setTimeout(resolve,0));
                    assert(keyboardList.textContent===t('keyboard.devices_unavailable'),'keyboard query error hidden');
                    queryError=false;

                    assert(panel.lastElementChild===keyboard,'keyboard is not last');
                    assert(panel.firstElementChild.id==='settings-midi-section','MIDI is not first');
                    for(const input of panel.querySelectorAll('.key-input'))
                        assert(keyboard.contains(input),'keyboard field escaped disclosure');
                    for(const id of ['config-global-keys','btn-enter-mapping','shortcuts-orphan-section'])
                        assert(keyboard.contains(document.getElementById(id)),id+' escaped disclosure');
                    assert(keyboard.querySelector('.reserved-shortcuts'),'reserved keyboard shortcuts outside disclosure');
                    assert(document.getElementById('android-midi-add').tagName==='SECTION','discovery remains collapsed');
                    assert(!keyboard.contains(button),'Bluetooth search inside keyboard disclosure');
                    document.getElementById('settings-modal').classList.remove('hidden');
                    document.getElementById('settings-modal').dataset.androidView='detail';
                    document.getElementById('s-principal')?.classList.add('hidden');
                    panel.classList.remove('hidden');
                    document.getElementById('android-midi-add').classList.remove('hidden');
                    assert(button.getClientRects().length,'Bluetooth search hidden');
                    const key=document.getElementById('config-key-stop');
                    assert(!key.checkVisibility(),'keyboard fields visible when collapsed');
                    const beforeExpand=queryCount;
                    keyboard.open=true;
                    await new Promise(resolve=>setTimeout(resolve,0));
                    assert(queryCount>beforeExpand,'expanding keyboard disclosure did not refresh devices');
                    assert(key.checkVisibility(),'keyboard fields unreachable when expanded');
                    for(const lang of Object.keys(locales)) {
                        language=lang;
                        for(const el of document.querySelectorAll('[data-i18n]')) el.textContent=t(el.dataset.i18n);
                        assert(keyboard.querySelector('summary').textContent===locales[lang].settings.keyboard_shortcuts,'untranslated disclosure');
                        assert(document.querySelector('[data-target="s-atajos"]').textContent===locales[lang].settings.tab_midi_keys,'untranslated section');
                        for(const key of ['connect','disconnect','connecting','retry','connection_failed','bluetooth_hint'])
                            assert(t('midi.'+key)!=='midi.'+key,'missing translation '+lang+'/'+key);
                        for(const key of ['title','empty','bluetooth_settings','bluetooth_hint','settings_unavailable','devices_unavailable'])
                            assert(t('keyboard.'+key)!=='keyboard.'+key,'missing keyboard translation '+lang+'/'+key);
                    }
                }
                const ids=[...document.querySelectorAll('[id]')].map(el=>el.id);
                assert(ids.length===new Set(ids).size,'duplicate IDs');
                document.body.dataset.midiLayout='passed';
            })().catch(error=>{document.body.dataset.midiLayout='failed: '+error.message;});
        `;
        html = html.replace('</body>', '<script>' + script + '</script></body>');
        const file = join(folder, 'midi-settings.html');
        writeFileSync(file, html);
        const result = spawnSync(browser, ['--headless', '--disable-gpu', '--no-first-run',
            '--no-default-browser-check', '--disable-background-networking', '--allow-file-access-from-files',
            '--user-data-dir=' + join(folder, 'browser'), '--window-size=390,640',
            '--virtual-time-budget=5000', '--dump-dom', pathToFileURL(file).href],
        { encoding: 'utf8', timeout: 60000, maxBuffer: 4 * 1024 * 1024, windowsHide: true });
        assert.ifError(result.error);
        const outcome = result.stdout?.match(/data-midi-layout="([^"]*)"/)?.[1];
        assert.equal(outcome, 'passed', outcome || result.stderr || 'No browser result');
    });
}
