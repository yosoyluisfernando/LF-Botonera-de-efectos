import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, existsSync, mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
const root = fileURLToPath(new URL('../', import.meta.url));
const browser = process.env.LF_TEST_BROWSER || 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe';
const body = name => readFileSync(join(root, 'src/js/ui', name), 'utf8')
    .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');
for (const width of [640, 1366]) for (const theme of ['light', 'dark']) {
    test(`tab outputs: selector, warnings and console at ${width}px ${theme}`, { skip: !existsSync(browser) }, () => {
        const folder = mkdtempSync(join(tmpdir(), 'lf-tab-outputs-'));
        try {
            let html = readFileSync(join(root, 'src/index.html'), 'utf8')
                .replace(/<script\b[^>]*>[\s\S]*?<\/script>/g, '')
                .replace(/href="(\/css\/[^"?]+)(?:\?[^" ]*)?"/g, (_, path) =>
                    `href="${pathToFileURL(join(root, 'src', path.slice(1))).href}"`);
            for (const name of ['console', 'consoleMeter', 'consoleFader', 'consoleDock', 'tabOutputs']) {
                html = html.replace('</head>', `<link rel="stylesheet" href="${pathToFileURL(join(root, 'src/css', name + '.css')).href}"></head>`);
            }
            const script = `
                const dictionary = ${readFileSync(join(root, 'src/public/i18n/es.json'), 'utf8')};
                const t = key => key.split('.').reduce((v, k) => v?.[k], dictionary) ?? key;
                let android = false;
                const isAndroid = () => android;
                const calls = [], handlers = {};
                const listen = async (name, callback) => { handlers[name] = callback; };
                const activeMeterTransition = () => 20;
                const attachPalette = async () => {}, refreshColorInputs = async () => {};
                const appAlert = async message => { throw Error(message); };
                const issue = { paleta_id: 'p_tab', nombre: 'Música', device: 'USB ausente', reason: 'missing' };
                let outputStatus = { devices: ['default', 'USB'], issues: [] };
                const fixed = (bus, in_program = true) => ({ bus, fader: 1, max: 1, in_program });
                const view = { strips: [fixed('efectos'), fixed('panel'), fixed('reproductor'),
                    ...Array.from({length: 8}, (_, i) => ({ ...fixed('salida'+i, false),
                        device: i ? 'USB '+i : '<img src=x>', available: i > 0, follows_master: true }))],
                    program: fixed('programa'), cue: fixed('cue', false) };
                const invoke = async (cmd, args) => {
                    calls.push({cmd, args});
                    if (cmd === 'get_tab_output_devices') return [{id: 'wasapi:usb', name: 'USB', usable: true}];
                    if (cmd === 'get_tab_output_status') return outputStatus;
                    if (cmd === 'get_console_view') return view;
                    if (cmd === 'get_volume_controls') return {faders: Object.fromEntries([...view.strips, view.program, view.cue].map(s => [s.bus, {value:s.fader,max:s.max}]))};
                    if (cmd === 'update_paleta_meta' || cmd === 'create_paleta' || cmd === 'set_bus_fader') return;
                    throw Error('Unexpected IPC '+cmd);
                };
                ${body('volumeSync.js')}
                ${body('volumeSlider.js')}
                ${body('consoleFaders.js')}
                ${body('consoleView.js')}
                ${body('tabOutputs.js')}
                ${body('tabModal.js')}
                const assert = (v, m) => { if (!v) throw Error(m); };
                (async () => {
                    document.documentElement.dataset.theme = '${theme}';
                    document.getElementById('loading-screen').classList.add('hidden');
                    document.getElementById('app-section').classList.remove('hidden');
                    const config = {active_profile_id: 'p', profiles: [{id: 'p', paletas: []}]};
                    const tab = {id: 'p_tab', nombre: 'Música', rows: 5, cols: 5, audio_out: 'USB ausente'};
                    await openTabModal(config, tab, () => {});
                    let select = document.getElementById('tab-audio-out');
                    assert(select.value === 'USB ausente', 'lost unavailable device');
                    assert(select.selectedOptions[0].textContent.includes('desconectada'), 'missing device not labelled');
                    select.value = 'wasapi:usb';
                    await document.getElementById('btn-save-tab').onclick();
                    assert(calls.find(c => c.cmd === 'update_paleta_meta').args.audioOut === 'USB', 'output not saved with tab');
                    assert(calls.find(c => c.cmd === 'update_paleta_meta').args.audioOutId === 'wasapi:usb', 'stable identity not saved');
                    await fillTabOutput({...tab, audio_out: 'Nombre anterior', audio_out_id: 'wasapi:usb'});
                    assert(select.value === 'wasapi:usb' && selectedTabOutput(tab) === 'USB', 'rename lost stable device');
                    await fillTabOutput({...tab, audio_out: 'USB', audio_out_id: 'wasapi:missing'});
                    assert(select.value === 'wasapi:missing' && selectedTabOutputId(tab) === 'wasapi:missing', 'missing ID fell back to same-name replacement');
                    android = true;
                    assert(selectedTabOutput(tab) === undefined, 'Android overwrites desktop assignment');
                    android = false;
                    await openTabModal(config, {...tab, obs: true}, () => {});
                    assert(document.getElementById('tab-output-row').hidden, 'OBS tab exposes audio output');
                    document.getElementById('tab-modal').classList.add('hidden');
                    await initTabOutputs();
                    outputStatus = {devices: ['default'], issues: [issue]};
                    handlers['tab-outputs-changed']({payload: outputStatus});
                    const notice = document.querySelector('.tab-output-notice');
                    assert(!notice.hidden && notice.textContent.includes('Música'), 'missing persistent output warning');
                    await notice.querySelector('button').onclick();
                    assert(calls.some(c => c.cmd === 'get_tab_output_status' && c.args.retry), 'retry does not reach Rust');
                    paintOutputStatus({devices: ['default'], issues: []});
                    assert(notice.hidden, 'reconnected warning not hidden');
                    document.getElementById('console-modal').classList.remove('hidden');
                    await renderConsole();
                    assert(document.querySelectorAll('.console-strip').length === 13, 'missing output channels');
                    const strips = [...document.querySelectorAll('#console-strips .console-strip')];
                    assert(strips[3].textContent.includes('<img src=x>') && !strips[3].querySelector('img'), 'device name interpreted as markup');
                    assert(strips[3].classList.contains('is-unavailable'), 'unavailable output not marked');
                    assert(!strips[4].classList.contains('is-direct'), 'master-controlled output labelled direct');
                    updateConsoleTick({buses: {salidas: Array.from({length: 8}, () => ({l: 0.5, r: 0.25, db: -6}))}});
                    assert(strips[4].querySelector('.strip-db').textContent.includes('-6.0'), 'independent meter not painted');
                    const fader = strips[4].querySelector('.fader'); fader.value = '0.4';
                    fader.dispatchEvent(new Event('input'));
                    const feedback = fader.closest('.fader-track');
                    assert(feedback.classList.contains('show-value') && feedback.querySelector('.fader-value').textContent === '40%', 'independent output missing live percentage');
                    assert(getComputedStyle(feedback.querySelector('.fader-value')).display === 'block', 'percentage CSS hidden');
                    const masterFader = document.querySelector('.fader.is-master');
                    masterFader.max = '1.5';
                    masterFader.dispatchEvent(new Event('input'));
                    const masterTrack = masterFader.closest('.fader-track');
                    const marker = masterTrack.querySelector('.fader-unity');
                    const top = parseFloat(getComputedStyle(marker).top);
                    assert(top > 0 && Math.abs(top - (7.5 + (masterTrack.clientHeight - 15) / 3)) < 2, '100% marker does not match unity position');
                    assert(getComputedStyle(masterTrack, '::before').backgroundImage.includes('linear-gradient'), 'boost rail not painted');
                    fader.dispatchEvent(new Event('change'));
                    await new Promise(resolve => setTimeout(resolve, 0));
                    assert(calls.some(c => c.cmd === 'set_bus_fader' && c.args.bus === 'salida1' && c.args.device === 'USB 1' && c.args.persist), 'output fader not persisted');
                    assert(!calls.some(c => /play|stop|queue/.test(c.cmd)), 'UI changed transport');
                    document.body.dataset.outputTest = 'passed';
                })().catch(e => { document.body.dataset.outputTest = 'failed: '+e.message; });
            `;
            html = html.replace('</body>', `<script>${script}</script></body>`);
            const fixture = join(folder, 'outputs.html'); writeFileSync(fixture, html);
            const result = spawnSync(browser, ['--headless', '--disable-gpu', '--no-first-run', '--no-default-browser-check',
                `--window-size=${width},768`, '--disable-background-networking', '--allow-file-access-from-files',
                `--user-data-dir=${resolve(folder, 'browser')}`, '--virtual-time-budget=5000', '--dump-dom', pathToFileURL(fixture).href],
                {encoding: 'utf8', timeout: 45000, maxBuffer: 3*1024*1024});
            assert.ifError(result.error);
            const outcome = result.stdout?.match(/data-output-test="([^"]*)"/)?.[1];
            assert.equal(outcome, 'passed', outcome || result.stderr);
        } finally { rmSync(folder, {recursive: true, force: true}); }
    });
}
