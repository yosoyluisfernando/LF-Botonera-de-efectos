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
const source = filename => readFileSync(join(root, filename), 'utf8')
    .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');
const locales = Object.fromEntries(['es', 'en', 'pt-BR', 'pt-PT'].map(language => [language,
    JSON.parse(readFileSync(join(root, 'src/public/i18n', language + '.json'), 'utf8'))]));

for (const platform of ['desktop', 'android']) {
    test(`${platform} shortcut labels and editor device availability preserve assignments`, {
        skip: !existsSync(browser),
    }, () => {
        const folder = mkdtempSync(join(tmpdir(), 'lf-shortcut-controls-'));
        let html = readFileSync(join(root, 'src/index.html'), 'utf8')
            .replace(/<script\b[^>]*>[\s\S]*?<\/script>/g, '')
            .replace(/href="(\/css\/[^"?]+)(?:\?[^" ]*)?"/g,
                (_, path) => 'href="' + pathToFileURL(join(root, 'src', path.slice(1))).href + '"');
        const script = `
          const platform=${JSON.stringify(platform)},locales=${JSON.stringify(locales)};
          let language='es';
          const t=key=>key.split('.').reduce((value,part)=>value?.[part],locales[language])??key;
          const isAndroid=()=>platform==='android';
          let availability={keyboard_available:false,midi_available:false},queryError=false,cancellations=0;
          const events={}, saves=[];
          const invoke=async(command,args)=>{
            if(command==='get_shortcut_editor_state') {
              if(queryError)throw Error('unavailable');return {...availability};
            }
            if(command==='set_shortcut_label_visibility') {saves.push(args);return;}
            throw Error('Unexpected IPC '+command);
          };
          const listen=async(name,callback)=>{events[name]=callback;return ()=>{};};
          const cancelMidiCapture=async()=>{cancellations++;};
          const hasActiveMidiCapture=()=>false;
          ${source('src/js/ui/settingsShortcutLabels.js')}
          ${source('src/js/ui/editShortcutSections.js')}
          const assert=(condition,message)=>{if(!condition)throw Error(message);};
          const tick=()=>new Promise(resolve=>setTimeout(resolve,0));
          (async()=>{
            document.body.dataset.platform=platform;
            initShortcutLabels({});
            for(const kind of ['keyboard','midi'])
              assert(document.getElementById('config-show-'+kind+'-shortcut-labels').checked,kind+' default hidden');
            const grid=document.getElementById('main-grid');
            grid.innerHTML='<div class="grid-item"><span data-shortcut-kind="keyboard">Ctrl+F2</span><span data-shortcut-kind="midi">Launchpad</span></div>';
            const badge=kind=>grid.querySelector('[data-shortcut-kind="'+kind+'"]');
            for(const lang of Object.keys(locales)) {
              language=lang;
              for(const [keyboard,midi] of [[true,true],[false,true],[true,false],[false,false]]) {
                document.getElementById('config-show-keyboard-shortcut-labels').checked=keyboard;
                document.getElementById('config-show-midi-shortcut-labels').checked=midi;
                await saveShortcutLabels();
                assert(JSON.stringify(saves.at(-1))===JSON.stringify({keyboard,midi}),'incorrect preference save');
                assert((getComputedStyle(badge('keyboard')).display!=='none')===keyboard,'keyboard badge visibility');
                assert((getComputedStyle(badge('midi')).display!=='none')===midi,'MIDI badge visibility');
                assert(badge('keyboard').textContent==='Ctrl+F2'&&badge('midi').textContent==='Launchpad','hiding removed assignment text');
              }
            }
            const modal=document.getElementById('edit-modal');modal.classList.remove('hidden');
            document.getElementById('edit-shortcut').value='Ctrl+F2';
            document.getElementById('edit-midi').value='Launchpad';
            const keyboardGroup=document.getElementById('edit-keyboard-shortcut');
            const midiGroup=document.getElementById('edit-midi-shortcut');
            await prepareEditShortcutSections();
            if(platform==='desktop') {
              assert(!document.getElementById('edit-keyboard-disclosure'),'desktop editor was folded');
              assert(!document.getElementById('edit-shortcut').disabled,'desktop keyboard disabled');
              assert(!document.getElementById('edit-midi-capture').disabled,'desktop MIDI disabled');
            } else {
              const keyboard=document.getElementById('edit-keyboard-disclosure');
              const midi=document.getElementById('edit-midi-disclosure');
              assert(keyboard.contains(keyboardGroup)&&midi.contains(midiGroup),'editor fields replaced');
              assert(!keyboard.open&&!midi.open,'absent device opened editor');
              assert(document.getElementById('edit-shortcut').disabled,'absent keyboard captures');
              assert(document.getElementById('edit-midi-capture').disabled,'disabled MIDI captures');
              availability.keyboard_available=true;events['keyboard-devices-changed']();await tick();
              assert(keyboard.open&&!midi.open,'keyboard connection opened wrong section');
              keyboard.open=false;
              availability.midi_available=true;events['midi-devices-changed']();await tick();
              assert(!keyboard.open&&midi.open,'manual collapse lost or connected MIDI closed');
              availability={keyboard_available:false,midi_available:false};
              events['midi-devices-changed']();await tick();
              assert(!keyboard.open&&!midi.open&&cancellations>0,'disconnect not folded/cancelled');
              assert(document.getElementById('edit-shortcut').value==='Ctrl+F2','keyboard assignment lost');
              assert(document.getElementById('edit-midi').value==='Launchpad','MIDI assignment lost');
              assert(!document.getElementById('edit-midi-clear').disabled,'cannot clear stored MIDI');
              queryError=true;await prepareEditShortcutSections();
              assert(document.getElementById('edit-midi-capture').disabled,'failed query enabled capture');
            }
            document.body.dataset.shortcutControls='passed';
          })().catch(error=>{document.body.dataset.shortcutControls='failed: '+error.message;});
        `;
        html = html.replace('</body>', '<script>' + script + '</script></body>');
        const file = join(folder, 'shortcuts.html');
        writeFileSync(file, html);
        const result = spawnSync(browser, ['--headless', '--disable-gpu', '--no-first-run',
            '--no-default-browser-check', '--disable-background-networking', '--allow-file-access-from-files',
            '--user-data-dir=' + join(folder, 'browser'), '--window-size=390,640',
            '--virtual-time-budget=5000', '--dump-dom', pathToFileURL(file).href],
        { encoding: 'utf8', timeout: 60000, maxBuffer: 4 * 1024 * 1024, windowsHide: true });
        assert.ifError(result.error);
        const outcome = result.stdout?.match(/data-shortcut-controls="([^"]*)"/)?.[1];
        assert.equal(outcome, 'passed', outcome || result.stderr || 'No browser result');
    });
}
