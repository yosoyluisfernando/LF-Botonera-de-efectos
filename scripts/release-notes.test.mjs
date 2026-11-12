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
const source = readFileSync(join(root, 'src/js/ui/startupPrompts.js'), 'utf8')
    .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');
const androidNotes = readFileSync(join(root, 'CHANGELOG.android.md'), 'utf8')
    .split('## [1.6.0]')[1].split('\n').slice(1).join('\n').split('\n## ')[0];
const desktopNotes = readFileSync(join(root, 'CHANGELOG.md'), 'utf8')
    .split('## [1.6.0]')[1].split('\n').slice(1).join('\n').split('\n## ')[0];

for (const width of [390, 768, 1366]) {
    test(`release notes flow as complete paragraphs at ${width}px`, { skip: !existsSync(browser) }, () => {
        const folder = mkdtempSync(join(tmpdir(), 'lf-release-notes-'));
        const sample = '### Heading\r\n- First **bold** line\r\n  continues with `code`.\r\n- Second item.\r\n\r\nA paragraph\r\ncontinues here.\r\n\r\n---\r\n### Next\r\n- <script>safe</script>';
        const script = `
            const t=key=>key,isAndroid=()=>false,links=[];
            window.__TAURI__={opener:{openUrl:async url=>{links.push(url);}}};
            ${source}
            const host=document.getElementById('notes');
            const assert=(value,message)=>{if(!value)throw Error(message);};
            try {
                host.innerHTML=_renderNotes(${JSON.stringify(sample)});
                const paragraphs=[...host.querySelectorAll('p')];
                assert(paragraphs.length===4,'wrapped source became extra paragraphs');
                assert(paragraphs[0].textContent==='First bold line continues with code.','list continuation split');
                assert(paragraphs[0].querySelector('strong') && paragraphs[0].querySelector('code'),'inline formatting lost');
                assert(paragraphs[2].textContent==='A paragraph continues here.','paragraph continuation split');
                assert(paragraphs[3].textContent==='<script>safe</script>' && !host.querySelector('script'),'unsafe content rendered');
                assert(host.querySelectorAll('h4').length===2,'headings or separator changed');
                host.innerHTML=_renderNotes(${JSON.stringify(androidNotes)});
                assert(!host.querySelector('p:not(.release-note-item)'),'Android list continuations became standalone paragraphs');
                assert([...host.querySelectorAll('p')].some(p=>p.textContent.includes(
                    'GitHub Releases. La app identifica Android y muestra el canal responsable')),'distribution paragraph split');
                assert(host.scrollWidth<=host.clientWidth+1,'notes overflow horizontally');
                host.innerHTML=_renderNotes(${JSON.stringify(desktopNotes)});
                assert(!host.querySelector('p:not(.release-note-item)'),'desktop list continuations became standalone paragraphs');
                assert([...host.querySelectorAll('p')].some(p=>p.textContent.includes(
                    'MIDI sin cables: conecta controladores compatibles por Bluetooth LE en Windows y Linux.')),'desktop paragraph split');
                assert(host.querySelectorAll('h4').length===1,'unpublished features presented as corrections');
                assert(host.scrollWidth<=host.clientWidth+1,'desktop notes overflow horizontally');
                _ensureReleaseModal();
                _prompts={currentVersion:'1.6.0',releaseNotes:${JSON.stringify(desktopNotes)}};
                _showReleaseNotes();
                const body=document.getElementById('release-notes-body');
                assert(body.lastElementChild.classList.contains('release-notes-support'),'support is not at the end');
                body.querySelectorAll('[data-community-link]').forEach(button=>button.click());
                assert(links.length===2 && links[0].startsWith('https://apps.microsoft.com/')
                    && links[1].includes('github.com/yosoyluisfernando/'),'support links are incorrect');
                document.body.dataset.notesResult='passed';
            } catch(error) {document.body.dataset.notesResult='failed: '+error.message;}
        `;
        const html = '<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="'
            + pathToFileURL(join(root, 'src/css/updateNotice.css')).href
            + '"><div id="notes" class="release-notes-body"></div><script>'
            + script.replace(/<\/script/gi, '<\\/script') + '</script>';
        const file = join(folder, 'notes.html');
        writeFileSync(file, html);
        const result = spawnSync(browser, ['--headless', '--disable-gpu', '--no-first-run',
            '--no-default-browser-check', '--disable-background-networking', '--allow-file-access-from-files',
            '--user-data-dir=' + join(folder, 'browser'), `--window-size=${width},800`,
            '--virtual-time-budget=3000', '--dump-dom', pathToFileURL(file).href],
        { encoding: 'utf8', timeout: 60000, maxBuffer: 2 * 1024 * 1024, windowsHide: true });
        assert.ifError(result.error);
        const outcome = result.stdout?.match(/data-notes-result="([^"]*)"/)?.[1];
        assert.equal(outcome, 'passed', outcome || result.stderr || 'No browser result');
    });
}
