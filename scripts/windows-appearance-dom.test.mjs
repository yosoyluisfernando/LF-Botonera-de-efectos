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
      ${moduleBody('desktopAppearance.js')}
      ${moduleBody('settingsAppearance.js')}
      ${moduleBody('settingsPlayback.js')}
      const assert = (value,message) => { if(!value) throw Error(message); };
      (async () => {
        document.body.dataset.platform='desktop';
        document.documentElement.dataset.theme='dark';
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
          select.value='default';select.dispatchEvent(new Event('change'));
          await new Promise(resolve=>setTimeout(resolve,0));
          assert(!document.documentElement.dataset.desktopStyle,'Default not restored');
        }
        document.body.dataset.appearanceTest='passed';
      })().catch(error=>{document.body.dataset.appearanceTest='failed: '+error.message;});
    `;
    html = html.replace('</body>', `<script>${script}</script></body>`);
    const fixture = join(folder, 'principal.html');
    writeFileSync(fixture, html);
    const result = spawnSync(edge, ['--headless', '--disable-gpu', '--no-first-run',
        '--no-default-browser-check', '--disable-background-networking', '--allow-file-access-from-files',
        `--user-data-dir=${resolve(folder, 'browser')}`, '--virtual-time-budget=3000', '--dump-dom',
        pathToFileURL(fixture).href], { encoding:'utf8', timeout:45000, maxBuffer:4*1024*1024 });
    assert.ifError(result.error);
    assert.match(result.stdout || '', /data-appearance-test="passed"/,
        (result.stdout?.match(/data-appearance-test="[^"]*"/)?.[0] || result.stderr || 'No browser result'));
});
