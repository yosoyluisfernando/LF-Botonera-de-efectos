import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

function paths(value, prefix = '') {
    return Object.entries(value).flatMap(([key, item]) => {
        const path = prefix ? `${prefix}.${key}` : key;
        return typeof item === 'object' ? paths(item, path) : [path];
    }).sort();
}
test('mobile controls and errors resolve through t() in all four languages', async () => {
    let reference;
    for (const language of ['es', 'en', 'pt-BR', 'pt-PT']) {
        const data = JSON.parse(await readFile(
            new URL(`../src/public/i18n/${language}.json`, import.meta.url), 'utf8'));
        const keys = paths(data);
        if (reference) assert.deepEqual(keys, reference);
        reference = keys;
        for (const key of ['back', 'activate_track', 'hint', 'add_audio', 'settings_back']) {
            assert.ok(data.mobile[key]?.length > 3);
            assert.ok(!data.mobile[key].includes('?'), `${language}: ${key}`);
        }
        for (const key of ['platform_unavailable', 'mobile_prelisten_unavailable',
            'mobile_folder_unavailable', 'mobile_export_unavailable']) {
            assert.equal(typeof data.errors[key], 'string');
            assert.ok(!data.errors[key].includes('?'), `${language}: ${key}`);
        }
    }
});
