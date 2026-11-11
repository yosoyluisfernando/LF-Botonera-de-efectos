import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';

function capture(name) {
    writeFileSync('android-smoke/' + name + '.png',
        execFileSync(process.argv[2], ['exec-out', 'screencap', '-p'], { timeout: 15000 }));
}

export async function verifyWelcome(evaluate, until) {
    await until(() => evaluate("document.getElementById('wizard-weather')?.getClientRects().length"),
        'Bienvenida no apareció');
    assert.equal(await evaluate(`(() => {
        const label = document.querySelector('.wizard-option label');
        const control = document.getElementById('wizard-weather');
        const visible = document.querySelector('.wizard-switch').getBoundingClientRect();
        if (label.getBoundingClientRect().height < 48 || visible.width < 44) return false;
        if (control.checked) label.click();
        label.querySelector('span').click();
        return control.checked;
    })()`), true);
    capture('bienvenida');
}

export async function verifyPanelGeometry(evaluate, landscape) {
    const sizes = await evaluate(`(() => {
        const box = selector => document.querySelector(selector).getBoundingClientRect();
        return { panel: box('#fixed-panel').width, shell: box('#workspace-shell').width,
            grid: box('.workspace-main').width, bottom: box('#bottom-bar').height,
            toolbar: box('.tabs-container').height,
            close: box('#fixed-panel-close').height, console: box('#btn-console').height };
    })()`);
    assert.equal(sizes.console, 0);
    if (landscape) {
        assert.ok(sizes.grid > 0 && sizes.panel < sizes.shell && sizes.bottom > 0, JSON.stringify(sizes));
        assert.equal(sizes.close, 0);
    } else {
        assert.ok(Math.abs(sizes.panel - sizes.shell) <= 1, JSON.stringify(sizes));
        assert.equal(sizes.grid, 0);
        assert.equal(sizes.bottom, 0);
        assert.equal(sizes.toolbar, 0);
        assert.ok(sizes.close >= 44);
    }
    capture(landscape ? 'reproductor-horizontal' : 'reproductor-vertical');
}

export async function verifyLocutions({ evaluate, until, adb, pkg, tapDocument }) {
    const hour = adb('shell', 'date', '+%H');
    const packs = [
        { key: 'time', label: 'LF-Hora', file: 'HRS' + hour + '.wav' },
        { key: 'temp', label: 'LF-Temperatura', file: 'TMP025.wav' },
        { key: 'hum', label: 'LF-Humedad', file: 'HUM082.wav' },
    ];
    const source = '/sdcard/Download/LF-Smoke.wav';
    const digest = adb('shell', 'sha256sum', source).split(' ')[0];

    for (const { label, file } of packs) {
        const folder = '/sdcard/Download/' + label;
        adb('shell', 'mkdir', '-p', folder);
        adb('shell', 'cp', source, folder + '/' + file);
        adb('shell', 'am', 'broadcast', '-a', 'android.intent.action.MEDIA_SCANNER_SCAN_FILE',
            '-d', 'file://' + folder + '/' + file);
    }

    await evaluate("document.getElementById('btn-settings').click()");
    await until(() => evaluate("!document.getElementById('settings-modal').classList.contains('hidden')"),
        'No se abrieron los ajustes');
    await evaluate("document.querySelector('[data-target=s-main]').click()");
    assert.equal(await evaluate("document.getElementById('config-show-console').getClientRects().length"), 0);
    await evaluate("document.querySelector('[data-target=s-locuciones]').click()");
    await until(() => evaluate("document.getElementById('loc-module').checked"),
        'Bienvenida no guardó Hora y clima');
    await evaluate(`for (const id of ['loc-time-on', 'loc-weather-on']) {
        const control = document.getElementById(id);
        if (!control.checked) control.click();
    }`);

    const paths = {};
    for (const { key, label, file } of packs) {
        await evaluate("document.getElementById('loc-" + key + "-browse').click()");
        await until(() => tapDocument(label) || (tapDocument('Downloads'), false)
            || (tapDocument('Show roots', 'Open navigation drawer'), false),
        'No se encontró la carpeta ' + label);
        await until(() => tapDocument('Use this folder', 'USE THIS FOLDER'),
            'Android no ofreció acceso a ' + label);
        await until(() => tapDocument('Allow', 'ALLOW'), 'Android no confirmó el acceso a ' + label);
        const path = await until(() => evaluate("document.getElementById('loc-" + key + "-folder').value"),
            'No se importó la carpeta ' + label);
        paths[key + '_folder'] = path;
        assert.ok(path.includes('/locutions/') && path.endsWith('/' + label), path);
        assert.equal(adb('shell', 'run-as', pkg, 'sha256sum', path + '/' + file).split(' ')[0], digest);
    }

    await evaluate("document.getElementById('loc-time-test').click()");
    await until(() => evaluate("document.getElementById('loc-time-status').dataset.kind === 'ok'"),
        'El motor no encontró o no pudo reproducir la locución de hora');
    capture('carpetas-locuciones');
    await evaluate("document.getElementById('btn-save-settings').click()");
    await until(() => evaluate("document.getElementById('settings-modal').classList.contains('hidden')"),
        'Los ajustes de Hora y clima no se guardaron');
    const saved = await evaluate("window.__TAURI__.core.invoke('get_config').then(config => config.locutions)");
    for (const [key, path] of Object.entries(paths)) assert.equal(saved[key], path);
    console.log('Hora, temperatura y humedad importadas con nombres y contenido intactos. Prueba de hora aceptada.');
}
