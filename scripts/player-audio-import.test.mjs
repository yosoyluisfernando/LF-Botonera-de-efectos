import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const code = readFileSync(new URL('../src/js/ui/touchTrackAction.js', import.meta.url), 'utf8')
    .replace(/^import .*;\r?\n/gm, '').replace(/export /g, '');

function fixture(path, failure) {
    const calls = [], errors = [];
    let click, refreshed = 0;
    const button = { disabled: false, addEventListener: (_, callback) => { click = callback; } };
    const context = vm.createContext({
        document: { getElementById: () => button },
        invoke: async (command, args) => {
            assert.equal(button.disabled, true, 'picker/import must prevent duplicate clicks');
            calls.push({ command, args });
            if (failure === command) throw Error('failed');
            return command === 'pick_audio_files' ? path : undefined;
        },
        alertIpcError: async error => { errors.push(error.message); },
        refresh: async () => { refreshed++; },
    });
    vm.runInContext(code + '\ninitPlayerAudioImport(refresh);', context);
    return { button, calls, errors, click: () => click(), refreshed: () => refreshed };
}

test('player + appends selected audio through existing commands without replacing the queue', async () => {
    const f = fixture(['C:/Music/song.mp3', 'C:/Music/second.flac']);
    await f.click();
    assert.deepEqual(f.calls.map(c => c.command), ['pick_audio_files', 'player_add_tracks']);
    assert.deepEqual(Array.from(f.calls[1].args.paths), ['C:/Music/song.mp3', 'C:/Music/second.flac']);
    assert.equal(f.refreshed(), 1);
    assert.equal(f.button.disabled, false);
});

test('canceling the picker leaves the queue untouched', async () => {
    const f = fixture([]);
    await f.click();
    assert.deepEqual(f.calls.map(c => c.command), ['pick_audio_files']);
    assert.equal(f.refreshed(), 0);
    assert.equal(f.button.disabled, false);
});

test('audio import failure reports the error and restores the add button', async () => {
    const f = fixture(['C:/Music/song.mp3'], 'player_add_tracks');
    await f.click();
    assert.deepEqual(f.errors, ['failed']);
    assert.equal(f.refreshed(), 0);
    assert.equal(f.button.disabled, false);
});
