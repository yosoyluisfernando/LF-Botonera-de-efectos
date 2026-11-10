import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { setTimeout as sleep } from 'node:timers/promises';
import { writeFileSync } from 'node:fs';

const adb = (...args) => execFileSync(process.argv[2], args, { encoding: 'utf8', timeout: 20000 }).trim();
const pkg = 'io.github.yosoyluisfernando.lfbotonera';
const port = 9222;

async function until(fn, message) {
    for (let attempt = 0; attempt < 20; attempt++) {
        try {
            const result = await fn();
            if (result) return result;
        } catch (error) {
            if (attempt === 19) throw error;
        }
        await sleep(800);
    }
    throw new Error(message);
}

async function connect() {
    const pid = adb('shell', 'pidof', pkg).split(' ')[0];
    adb('forward', `tcp:${port}`, `localabstract:webview_devtools_remote_${pid}`);
    const url = await until(async () => {
        const pages = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
        return pages.find(page => page.type === 'page')?.webSocketDebuggerUrl;
    }, 'No se encontró el WebView de depuración');
    const ws = new WebSocket(url);
    await new Promise((resolve, reject) => {
        ws.addEventListener('open', resolve, { once: true });
        ws.addEventListener('error', reject, { once: true });
    });
    const pending = new Map();
    let id = 0;
    ws.addEventListener('message', event => {
        const message = JSON.parse(event.data);
        pending.get(message.id)?.(message);
        pending.delete(message.id);
    });
    const evaluate = (expression) => new Promise((resolve, reject) => {
        const seq = ++id;
        const timeout = setTimeout(() => {
            pending.delete(seq);
            reject(new Error(`Tiempo agotado: ${expression}`));
        }, 45000);
        pending.set(seq, (message) => {
            clearTimeout(timeout);
            const reply = message.result;
            if (message.error || reply?.exceptionDetails) {
                reject(new Error(JSON.stringify(message.error ?? reply.exceptionDetails)));
            } else {
                resolve(reply.result.value);
            }
        });
        ws.send(JSON.stringify({ id: seq, method: 'Runtime.evaluate', params: {
            expression, awaitPromise: true, returnByValue: true,
        } }));
    });
    return { evaluate, close: () => ws.close() };
}

const { evaluate, close } = await connect();
try {
    await until(() => evaluate(`(() => {
        if (document.querySelector('.grid-item')?.getClientRects().length) return true;
        ['preload-dialog-skip','wizard-start-btn'].map(id => document.getElementById(id))
            .find(button => button?.getClientRects().length)?.click();
        return false;
    })()`), 'No aparece la rejilla');
    const home = adb('shell', 'run-as', pkg, 'pwd');
    const path = home + '/files/LF-Latency.wav';
    await evaluate(`window.__TAURI__.core.invoke('assign_file_to_button', {index: 1, path: ${JSON.stringify(path)}})`);
    await evaluate('setTimeout(() => location.reload(), 100); true');
    const position = await until(() => evaluate(`(() => {
        const dismiss = ['release-notes-ok','donation-later','preload-dialog-skip']
            .map(id => document.getElementById(id)).find(button => button?.getClientRects().length);
        if (dismiss) { dismiss.click(); return null; }
        const cell = document.querySelector('.grid-item[data-id]');
        if (!cell?.getClientRects().length) return null;
        const r = cell.getBoundingClientRect();
        const x = r.left + r.width / 2, y = r.top + r.height / 2;
        if (!cell.contains(document.elementFromPoint(x,y))) return null;
        return {id: cell.dataset.id, x: Math.round(x * devicePixelRatio), y: Math.round(y * devicePixelRatio)};
    })()`), 'La celda no es visible');
    await evaluate(`(async () => {
        const id = ${JSON.stringify(position.id)};
        const rows = window.lfLatency = [];
        let row;
        function cell(event) { return event.target.closest('.grid-item')?.dataset.id === id; }
        document.addEventListener('pointerup', event => {
            if (cell(event)) { row = {up: performance.now()}; rows.push(row); }
        }, true);
        document.addEventListener('click', event => {
            if (row && cell(event)) row.click = performance.now() - row.up;
        }, true);
        await window.__TAURI__.event.listen('audio-tick', ({payload}) => {
            if (row && row.tick == null && payload.buttons.some(button => button.id === id))
                row.tick = performance.now() - row.up;
        });
        await window.__TAURI__.event.listen('meter-tick', ({payload}) => {
            if (row && row.meter == null && payload.master_level_l > 0)
                row.meter = performance.now() - row.up;
        });
        return true;
    })()`);
    for (let n=0; n<9; n++) {
        adb('shell','input','tap', String(position.x),String(position.y));
        await sleep(2400);
    }
    const data = await evaluate('window.lfLatency');
    assert.equal(data.length,9);
    assert.ok(data.every(row => Number.isFinite(row.click) && Number.isFinite(row.tick)));
    const summarize = (rows, key) => {
        const s = rows.map(row=>row[key]).filter(Number.isFinite).sort((a,b)=>a-b);
        return s.length ? {n:s.length, median:+s[Math.floor(s.length/2)].toFixed(1), min:+s[0].toFixed(1), max:+s.at(-1).toFixed(1)} : null;
    };
    const output = {apk:process.argv[3], cold:data[0], warm:Object.fromEntries(
        ['click','tick','meter'].map(key=>[key,summarize(data.slice(1),key)])), samples:data};
    console.log('LF_LATENCY',JSON.stringify(output));
    writeFileSync(`android-smoke/latencia-${process.argv[3]}.json`, JSON.stringify(output,null,2));
} finally { close(); }

