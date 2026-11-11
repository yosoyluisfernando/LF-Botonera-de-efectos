import assert from 'node:assert/strict';

export async function dismissStartup(evaluate, until) {
    let quiet = 0;
    await until(async () => {
        const settled = await evaluate(`(() => {
        const ids = ['preload-dialog-skip', 'release-notes-ok', 'donation-later', 'nc-cancel'];
        const visible = ids.map(id => document.getElementById(id)).filter(el => el?.getClientRects().length);
        visible.forEach(el => el.click());
        return visible.length === 0 && !document.querySelector('.modal-overlay:not(.hidden)');
        })()`);
        quiet = settled ? quiet + 1 : 0;
        return quiet >= 3;
    }, 'Los avisos de inicio no se cerraron');
}

export async function verifyAndroidToolbar(evaluate, until) {
    const title = await evaluate(`(() => {
        const button = document.getElementById('btn-profiles');
        return { text: button.textContent, name: button.getAttribute('aria-label'),
            remote: document.getElementById('btn-remote-mirror').textContent };
    })()`);
    assert.equal(title.text, '👤');
    assert.ok(title.name.includes('Perfil'));
    assert.equal(title.remote, '🎮');
    await evaluate("document.getElementById('btn-remote-mirror').click()");
    assert.ok(await evaluate("document.querySelector('.remote-dialog').open"));
    await evaluate("document.querySelector('.remote-dialog > button').click()");
    for (const enabled of [false, true]) {
        await evaluate("document.getElementById('btn-settings').click()");
        await until(() => evaluate("!document.getElementById('settings-modal').classList.contains('hidden')"),
            'No se abrieron los ajustes');
        await evaluate("document.querySelector('[data-target=s-remote]').click()");
        await evaluate("document.getElementById('remote-client-enabled').checked = " + enabled);
        await evaluate("document.getElementById('btn-save-settings').click()");
        await until(() => evaluate("document.getElementById('settings-modal').classList.contains('hidden')"),
            'No se guardó la visibilidad del acceso remoto');
        assert.equal(await evaluate("!!document.getElementById('btn-remote-mirror').getClientRects().length"), enabled);
        assert.equal(await evaluate("window.__TAURI__.core.invoke('get_config').then(c => c.remote_client_enabled)"), enabled);
    }
    console.log('Perfiles y control remoto con iconos, desactivación guardada y acceso oculto.');
}
