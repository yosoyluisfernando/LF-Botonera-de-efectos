import assert from 'node:assert/strict';

export async function verifyAndroidBack({ evaluate, until, adb, pkg }) {
    const back = () => adb('shell', 'input', 'keyevent', '4');
    const hidden = id => evaluate(`document.getElementById('${id}').classList.contains('hidden')`);
    await evaluate(`document.getElementById('btn-settings').click()`);
    await until(async () => !(await hidden('settings-modal')), 'No abrió Ajustes para probar Atrás');
    await evaluate(`document.querySelector('[data-target="s-about"]').click()`);
    const info = await evaluate(`window.__TAURI__.core.invoke('get_distribution_info')`);
    assert.equal(info.platform, 'android');
    assert.equal(info.channel, 'direct');
    assert.equal(info.updateManager, 'github_releases');
    assert.equal(info.version, '1.6.0');
    assert.equal(await evaluate(`document.getElementById('app-platform').textContent`), 'Android');
    back();
    await until(() => evaluate(`document.getElementById('settings-modal').dataset.androidView === 'sections'`),
        'Atrás no volvió de Acerca de a las secciones');
    assert.equal(await hidden('settings-modal'), false);
    back();
    await until(() => hidden('settings-modal'), 'Atrás no volvió de Ajustes a la rejilla');

    await evaluate(`(() => {
        const cell = document.querySelector('.grid-item[data-id]') || document.querySelector('.grid-item');
        cell.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true, clientX: 50, clientY: 50 }));
        document.getElementById('menu-editar').click();
    })()`);
    await until(async () => !(await hidden('edit-modal')), 'No abrió el editor para probar Atrás');
    await evaluate(`document.getElementById('edit-visual-open').click()`);
    await until(() => evaluate(`!!document.querySelector('.visual-selector-overlay:not(.hidden)')`),
        'No abrió el selector de identificador');
    back();
    await until(() => evaluate(`!document.querySelector('.visual-selector-overlay:not(.hidden)')`),
        'Atrás no cerró el selector de identificador');
    assert.equal(await hidden('edit-modal'), false);
    back();
    await until(() => hidden('edit-modal'), 'Atrás no cerró el editor');

    await evaluate(`document.getElementById('btn-fixed-panel').click()`);
    await until(async () => !(await hidden('fixed-panel')), 'No abrió el panel fijo');
    back();
    await until(() => hidden('fixed-panel'), 'Atrás no cerró el panel fijo');
    assert.equal(await evaluate(`window.__TAURI__.core.invoke('get_fixed_panel').then(panel => panel.settings.visible)`), false);

    await evaluate(`document.getElementById('btn-remote-mirror').click()`);
    await until(() => evaluate(`document.querySelector('.remote-dialog').open`), 'No abrió el control remoto');
    back();
    await until(() => evaluate(`!document.querySelector('.remote-dialog').open`), 'Atrás no cerró el control remoto');

    back();
    await until(() => evaluate(`!!document.querySelector('#app-dialog-modal:not(.hidden)')`),
        'Atrás en la rejilla no pidió confirmar la salida');
    back();
    await until(() => hidden('app-dialog-modal'), 'Otro Atrás no canceló la confirmación de salida');
    assert.ok(adb('shell', 'pidof', pkg));
    console.log('Atrás Android: Acerca de, Ajustes, editor anidado, panel fijo, control remoto y cancelación de salida correctos.');

    back();
    await until(() => evaluate(`!!document.querySelector('#app-dialog-modal:not(.hidden)')`),
        'No volvió a pedir confirmar la salida');
    await evaluate(`document.querySelector('.app-dialog-ok').click()`);
    await until(() => !adb('shell', 'dumpsys', 'activity', 'activities').split('\n')
        .some(line => /topResumedActivity|mResumedActivity/.test(line) && line.includes(pkg + '/')),
    'La confirmación no cerró la tarea Android');
    console.log('Salida Android confirmada sin cierre accidental.');
}
