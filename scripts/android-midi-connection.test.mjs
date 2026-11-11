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
const source = readFileSync(join(root, 'src/js/ui/settingsMidi.js'), 'utf8')
    .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');
const locales = Object.fromEntries(['es', 'en', 'pt-BR', 'pt-PT'].map(language => [language,
    JSON.parse(readFileSync(join(root, 'src/public/i18n', language + '.json'), 'utf8'))]));

for (const platform of ['android', 'desktop']) {
    test(`${platform} MIDI connection actions persist selection and reflect backend states`, {
        skip: !existsSync(browser),
    }, () => {
        const folder = mkdtempSync(join(tmpdir(), 'lf-midi-connection-'));
        const script = `
            const platform=${JSON.stringify(platform)}, locales=${JSON.stringify(locales)};
            let language='es', devices=[];
            const saved=[], alerts=[];
            const isAndroid=()=>platform==='android';
            const t=key=>key.split('.').reduce((value,part)=>value?.[part],locales[language])??key;
            const assert=(condition,message)=>{if(!condition)throw Error(language+': '+message);};
            const appAlert=async error=>alerts.push(error);
            const invoke=async(command,args)=>{
                if(command==='midi_devices')return devices;
                if(command==='midi_set_config') {
                    saved.push(structuredClone(args));
                    devices=devices.map(device=>({...device,selected:args.inputIds.includes(device.id),
                        connection_state:args.inputIds.includes(device.id)?'connecting':'disconnected'}));
                    return {midi:{enabled:args.enabled,inputs:args.inputIds}};
                }
                throw Error('Unexpected IPC '+command);
            };
            ${source}
            const enabled=document.getElementById('config-midi-enabled');
            const host=document.getElementById('config-midi-devices');
            const id='ble:AA:BB:CC:DD:EE:FF';
            const render=async(state,selected=false)=>{
                devices=[{id,label:'Bluetooth Controller '+('M'.repeat(90)),selected,available:true,
                    connected:state==='connected',connection_state:state}];
                enabled.checked=true;
                await loadMidiDevices();
                return host.firstElementChild;
            };
            const button=key=>[...host.querySelectorAll('button')].find(action=>action.textContent===t(key));
            const press=async(key,expectedIds)=>{
                const action=button(key), count=saved.length;
                assert(action&&!action.disabled,'missing enabled '+key);
                action.click();
                assert([...host.querySelectorAll('button')].every(item=>item.disabled),'actions not disabled while saving');
                await new Promise(resolve=>setTimeout(resolve,0));
                assert(saved.length===count+1,'click did not save exactly once');
                const last=saved.at(-1);
                assert(last.enabled===true,'click changed MIDI activation');
                assert(JSON.stringify(last.inputIds)===JSON.stringify(expectedIds),'wrong saved input IDs for '+key);
                assert(host.querySelector('[data-midi-device]').checked===expectedIds.includes(id),'selection not refreshed');
            };
            (async()=>{
                for(const lang of Object.keys(locales)) {
                    language=lang;
                    let row=await render('disconnected');
                    const input=row.querySelector('input');
                    assert(row.textContent.includes(t('midi.bluetooth_transport')),'Bluetooth transport missing');
                    if(platform==='desktop') {
                        assert(row.tagName==='LABEL','desktop label replaced');
                        assert(!input.hidden&&input.checkVisibility(),'desktop checkbox hidden');
                        assert(!row.querySelector('button'),'desktop checkbox replaced with actions');
                        input.checked=true;input.dispatchEvent(new Event('change'));
                        await new Promise(resolve=>setTimeout(resolve,0));
                        assert(saved.at(-1).inputIds.includes(id),'desktop checkbox did not save');
                        enabled.checked=false;updateDeviceSelectionState();
                        assert(host.querySelector('input').disabled,'desktop checkbox active with MIDI off');
                        continue;
                    }
                    assert(row.tagName==='DIV'&&input.hidden,'Android checkbox still visible');
                    assert(button('midi.connect').getAttribute('aria-label').includes(devices[0].label),'action lacks device name');
                    assert(row.scrollWidth<=document.documentElement.clientWidth,'long device name overflows phone');
                    await press('midi.connect',[id]);
                    assert(host.textContent.includes(t('midi.connecting')),'connecting state not shown after Connect');
                    await press('midi.disconnect',[]);
                    await render('connecting',true);
                    assert(host.textContent.includes(t('midi.connecting')),'connecting status missing');
                    await press('midi.disconnect',[]);
                    await render('failed',true);
                    assert(host.textContent.includes(t('midi.connection_failed')),'connection error missing');
                    assert(button('midi.disconnect'),'failed connection cannot be cancelled');
                    await press('midi.retry',[id]);
                    await render('failed',true);
                    await press('midi.disconnect',[]);
                    await render('connected',true);
                    assert(host.textContent.includes(t('midi.connected')),'connected status missing');
                    await press('midi.disconnect',[]);
                    await render('failed',true);
                    enabled.checked=false;updateDeviceSelectionState();
                    assert([...host.querySelectorAll('[data-midi-device],[data-midi-action]')].every(item=>item.disabled),'actions active with MIDI off');
                    const count=saved.length;button('midi.retry').click();button('midi.disconnect').click();
                    await new Promise(resolve=>setTimeout(resolve,0));
                    assert(saved.length===count,'disabled action changed selection');
                }
                assert(alerts.length===0,'unexpected save error');
                document.body.dataset.midiConnection='passed';
            })().catch(error=>{document.body.dataset.midiConnection='failed: '+error.message;});
        `;
        const file = join(folder, 'connection.html');
        const css = pathToFileURL(join(root, 'src/css/midi.css')).href;
        writeFileSync(file, '<!doctype html><html><head><meta charset="utf-8"><link rel="stylesheet" href="'
            + css + '"></head><body><input type="checkbox" id="config-midi-enabled">'
            + '<div id="config-midi-devices"></div><script>' + script + '</script></body></html>');
        const result = spawnSync(browser, ['--headless', '--disable-gpu', '--no-first-run',
            '--no-default-browser-check', '--disable-background-networking', '--allow-file-access-from-files',
            '--user-data-dir=' + join(folder, 'browser'), '--window-size=390,640',
            '--virtual-time-budget=5000', '--dump-dom', pathToFileURL(file).href],
        { encoding: 'utf8', timeout: 60000, maxBuffer: 4 * 1024 * 1024, windowsHide: true });
        assert.ifError(result.error);
        const outcome = result.stdout?.match(/data-midi-connection="([^"]*)"/)?.[1];
        assert.equal(outcome, 'passed', outcome || result.stderr || 'No browser result');
    });
}
