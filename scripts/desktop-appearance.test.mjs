import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const source = readFileSync(new URL('../src/js/ui/desktopAppearance.js', import.meta.url), 'utf8')
    .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');

function windowFixture({ enabled = true, platform = 'desktop', theme = 'dark' } = {}) {
    const cache = new Map();
    const events = [];
    const calls = [];
    const dataset = { theme };
    let remote;
    let failSave = false;
    const context = vm.createContext({
        __LF_DESKTOP_THEMES__: enabled,
        platformCapabilities: () => ({ platform }),
        document: { documentElement: { dataset }, dispatchEvent: e => events.push(e) },
        localStorage: {
            getItem: k => { calls.push('cache-read'); return cache.get(k); },
            setItem: (k, v) => cache.set(k, v), removeItem: k => cache.delete(k),
        },
        CustomEvent: class { constructor(type, options) { Object.assign(this, { type }, options); } },
        listen: async (name, fn) => { calls.push(name); remote = fn; },
        invoke: async (cmd, args) => {
            calls.push(cmd);
            if (cmd === 'set_desktop_appearance' && failSave) throw Error('disk full');
            return { available: true, style: args?.style || 'default' };
        },
    });
    vm.runInContext(source, context);
    return { context, cache, events, calls, dataset,
        receive: style => remote({ payload: { available: true, style } }),
        rejectSave: () => { failSave = true; } };
}

test('switching design redraws the open waveform with the resolved color mode', async () => {
    for (const theme of ['light', 'dark']) {
        const w = windowFixture({ theme });
        await w.context.initializeDesktopAppearance();
        await w.context.selectDesktopAppearance('modern');
        assert.equal(w.dataset.desktopStyle, 'modern');
        assert.equal(w.events.at(-1).type, 'lf-theme-change');
        assert.equal(w.events.at(-1).detail.theme, theme);
        assert.equal(w.dataset.theme, theme);
        assert.equal(w.cache.get('lf-botonera-desktop-style'), 'modern');
        await w.context.selectDesktopAppearance('default');
        assert.equal(w.dataset.desktopStyle, undefined);
        assert.equal(w.cache.size, 0);
    }
});

test('a rejected save preserves the active style and the first-paint cache', async () => {
    const w = windowFixture();
    await w.context.initializeDesktopAppearance();
    await w.context.selectDesktopAppearance('modern');
    const count = w.events.length;
    w.rejectSave();
    await assert.rejects(w.context.selectDesktopAppearance('default'), /disk full/);
    assert.equal(w.context.desktopAppearance().style, 'modern');
    assert.equal(w.dataset.desktopStyle, 'modern');
    assert.equal(w.cache.get('lf-botonera-desktop-style'), 'modern');
    assert.equal(w.events.length, count);
});

test('persisted changes reach independent windows and clear Modern on Default', async () => {
    const w = windowFixture();
    await w.context.initializeDesktopAppearance();
    w.receive('modern');
    assert.equal(w.dataset.desktopStyle, 'modern');
    w.receive('default');
    assert.equal(w.dataset.desktopStyle, undefined);
    assert.equal(w.events.at(-1).type, 'lf-theme-change');
});

test('excluded builds never read the appearance cache or request its commands', async () => {
    for (const platform of ['android', 'ios']) {
        const w = windowFixture({ enabled: false, platform });
        w.cache.set('lf-botonera-desktop-style', 'modern');
        w.context.initializeAppearanceCache();
        await w.context.initializeDesktopAppearance();
        assert.deepEqual(w.calls, []);
        assert.equal(w.dataset.desktopStyle, undefined);
    }
});

test('a browser preview never activates the native desktop setting', async () => {
    const w = windowFixture({ platform: 'browser' });
    await w.context.initializeDesktopAppearance();
    assert.deepEqual(w.calls, []);
    assert.equal(w.context.desktopAppearance().available, false);
});

// La capacidad identifica la familia de interfaz, no el sistema operativo.
test('the runtime fixture agrees with the actual Rust platform contract', () => {
    const rust = readFileSync(new URL('../src-tauri/src/core/platform.rs', import.meta.url), 'utf8');
    assert.match(rust, /platform: if android \{ "android" \} else \{ "desktop" \}/);
});
