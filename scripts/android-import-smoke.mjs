import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { setTimeout as sleep } from 'node:timers/promises';

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

function tapDocument(text) {
    adb('shell', 'uiautomator', 'dump', '/sdcard/lf-picker.xml');
    const xml = adb('exec-out', 'cat', '/sdcard/lf-picker.xml');
    const node = xml.match(/<node\b[^>]*>/g)?.find(el =>
        el.includes(`text="${text}"`) || el.includes(`content-desc="${text}"`));
    const bounds = node?.match(/bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]"/);
    if (!bounds) return false;
    const [, left, top, right, bottom] = bounds.map(Number);
    adb('shell', 'input', 'tap', String((left + right) >> 1), String((top + bottom) >> 1));
    return true;
}

async function verifyLayout(evaluate) {
    for (const [rotation, landscape] of [['0', false], ['1', true]]) {
        adb('shell', 'settings', 'put', 'system', 'user_rotation', rotation);
        const layout = await until(async () => {
            const bounds = await evaluate(`(() => {
                const bounds = id => document.getElementById(id).getBoundingClientRect();
                return { landscape: innerWidth > innerHeight, stop: bounds('pb-btn-stop-all').bottom,
                    height: innerHeight, clock: bounds('clock-widget').height };
            })()`);
            return bounds.landscape === landscape && bounds.stop > 0 ? bounds : false;
        }, 'La botonera no apareció en la orientación solicitada');
        assert.equal(layout.clock, 0);
        assert.ok(layout.stop <= layout.height + 1, JSON.stringify(layout));
        console.log('Transporte visible', landscape ? 'horizontal' : 'vertical', layout);
    }
}

const { evaluate, close } = await connect();
try {
    // Completa el primer inicio sin modificar el modelo ni saltarse el selector nativo.
    for (let attempt = 0; attempt < 12; attempt++) {
        await evaluate(`[...document.querySelectorAll('button')].find(button =>
            button.getClientRects().length && /^(Comenzar|Start|Aceptar|OK|Ahora no|Más tarde)$/i.test(button.textContent.trim()))?.click()`);
        await sleep(400);
    }
    await until(() => evaluate("document.querySelector('.grid-item') !== null"), 'No se dibujó la rejilla');
    const assignment = evaluate("window.__TAURI__.core.invoke('assign_file_to_button', { index: 1, path: null })");
    assignment.catch(() => {});
    await until(() => adb('shell', 'dumpsys', 'activity', 'activities').includes('documentsui'), 'No abrió el selector SAF');
    if (!tapDocument('LF-Smoke.wav')) {
        assert.ok(tapDocument('Show roots') || tapDocument('Open navigation drawer') || tapDocument('Browse'), 'No se encontró el menú del selector');
        await until(() => tapDocument('Downloads'), 'No se encontró la carpeta Downloads');
        await until(() => tapDocument('LF-Smoke.wav'), 'No se encontró el audio de prueba');
    }
    const grid = await assignment;
    const button = grid.buttons.find(item => item.index === 1);
    assert.ok(button?.path.includes('/imports/'), JSON.stringify(grid));
    assert.ok(button.duration > 0);
    const source = adb('shell', 'sha256sum', '/sdcard/Download/LF-Smoke.wav').split(' ')[0];
    assert.equal(adb('shell', 'run-as', pkg, 'sha256sum', button.path).split(' ')[0], source);
    await evaluate(`window.__TAURI__.core.invoke('play_button', { id: ${JSON.stringify(button.id)} })`);
    console.log('Importación SAF, copia verificada y disparo aceptado', button.id, button.duration);
    adb('shell', 'settings', 'put', 'system', 'accelerometer_rotation', '0');
    await verifyLayout(evaluate);
} finally {
    close();
}
