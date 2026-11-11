import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { spawn } from 'node:child_process';

const browser = process.env.LF_TEST_BROWSER || (process.platform==='win32'
    ? 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe' : '/usr/bin/google-chrome');
const root = process.env.LF_TEST_REPO || fileURLToPath(new URL('../', import.meta.url));
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));

test('stationary pointer stays on the grid and fixed-panel button edges', {
    skip: !existsSync(browser), timeout:45000,
}, async () => {
    const folder = mkdtempSync(join(tmpdir(), 'lf-hover-'));
    const profile = join(folder, 'browser');
    let html = readFileSync(join(root, 'src/index.html'), 'utf8')
        .replace(/<script\b[^>]*>[\s\S]*?<\/script>/g, '')
        .replace(/href="(\/css\/[^"?]+)(?:\?[^" ]*)?"/g,
            (_, path) => 'href="'+pathToFileURL(join(root, 'src', path.slice(1))).href+'"');
    const fixture = join(folder, 'edges.html');
    writeFileSync(fixture, html);
    const child = spawn(browser, ['--headless','--disable-gpu','--no-first-run',
        '--no-default-browser-check','--disable-background-networking','--allow-file-access-from-files',
        '--remote-debugging-port=0','--user-data-dir='+profile,'about:blank'],
    { stdio:'ignore', windowsHide:true });
    let ws;
    try {
        const portFile = join(profile, 'DevToolsActivePort');
        const deadline = Date.now()+15000;
        while (!existsSync(portFile) && Date.now()<deadline) await pause(50);
        assert.ok(existsSync(portFile), 'Browser debugger did not start');
        const port = readFileSync(portFile,'utf8').split('\n')[0];
        const pages = await (await fetch('http://127.0.0.1:'+port+'/json/list')).json();
        ws = new WebSocket(pages.find(page=>page.type==='page').webSocketDebuggerUrl);
        await new Promise((resolve,reject)=>{ws.onopen=resolve;ws.onerror=reject;});
        let next = 0;
        const pending = new Map();
        ws.onmessage = event => {
            const msg = JSON.parse(event.data), waiter = pending.get(msg.id);
            if (!waiter) return;
            pending.delete(msg.id);
            if (msg.error) waiter.reject(Error(msg.error.message)); else waiter.resolve(msg.result);
        };
        const send = (method, params={}) => new Promise((resolve,reject)=>{
            const id=++next;pending.set(id,{resolve,reject});ws.send(JSON.stringify({id,method,params}));
        });
        const evaluate = async expression => {
            const result = await send('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});
            assert.equal(result.exceptionDetails,undefined,result.exceptionDetails?.text);
            return result.result.value;
        };
        await send('Emulation.setDeviceMetricsOverride',{width:1366,height:768,deviceScaleFactor:1,mobile:false});
        await send('Page.navigate',{url:pathToFileURL(fixture).href});
        while (!await evaluate('document.readyState === "complete"')) await pause(50);
        await evaluate(`(() => {
          document.documentElement.dataset.desktopStyle='modern';document.body.dataset.platform='desktop';
          document.getElementById('app-section').classList.remove('hidden');
          document.getElementById('loading-screen').classList.add('hidden');
          const grid=document.getElementById('main-grid');
          grid.style.gridTemplateColumns='repeat(3,1fr)';grid.style.gridAutoRows='100px';
          grid.innerHTML='<div class="grid-item" id="probe-grid">Sound</div>';
          const panel=document.getElementById('fixed-panel');panel.classList.remove('hidden');panel.dataset.view='buttons';
          document.getElementById('fixed-panel-items').innerHTML='<div class="fixed-panel-item" id="probe-fixed" style="height:100px">Sound</div>';
          window.counts={};for(const id of ['probe-grid','probe-fixed']) {
            counts[id]={enter:0,leave:0};const el=document.getElementById(id);
            el.addEventListener('mouseenter',()=>counts[id].enter++);
            el.addEventListener('mouseleave',()=>counts[id].leave++);
          }
        })()`);
        for (const theme of ['light','dark']) {
            await evaluate('document.documentElement.dataset.theme='+JSON.stringify(theme));
            for (const id of ['probe-grid','probe-fixed']) {
                await send('Input.dispatchMouseEvent',{type:'mouseMoved',x:0,y:0});
                await pause(200);
                const point = await evaluate(`(() => {
                  counts[${JSON.stringify(id)}]={enter:0,leave:0};
                  const r=document.getElementById(${JSON.stringify(id)}).getBoundingClientRect();
                  return {x:r.left+r.width/2,y:r.bottom-0.5};
                })()`);
                assert.ok(point.y>0 && point.y<768,'Probe must be visible');
                await send('Input.dispatchMouseEvent',{type:'mouseMoved',...point});
                await pause(600);
                const underPointer=await evaluate('document.getElementById('+JSON.stringify(id)+').contains(document.elementFromPoint('+point.x+','+point.y+'))');
                assert.ok(underPointer,theme+' hover moved button away from stationary pointer');
                let state = await evaluate('counts['+JSON.stringify(id)+']');
                assert.deepEqual(state,{enter:1,leave:0},theme+' hover at '+id+' oscillates');
                await send('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...point});
                await pause(250);
                state = await evaluate('counts['+JSON.stringify(id)+']');
                assert.deepEqual(state,{enter:1,leave:0},theme+' press at '+id+' moves hit target');
                await send('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...point});
            }
        }
        await send('Browser.close');
    } finally {
        ws?.close();child.kill();
    }
});
