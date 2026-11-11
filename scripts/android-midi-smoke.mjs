import assert from 'node:assert/strict';

export async function verifyMidiUsb(evaluate, until, adb, pkg) {
    await evaluate(`(() => {
        document.querySelector('[data-target="s-atajos"]').click();
        const panel = document.getElementById('s-atajos');
        const keyboard = document.getElementById('android-keyboard-shortcuts');
        if (!keyboard || keyboard.open || panel.lastElementChild !== keyboard) {
            throw Error('Los atajos del teclado deben estar plegados al final');
        }
        if (document.getElementById('android-midi-add').tagName !== 'SECTION') {
            throw Error('La búsqueda MIDI debe estar visible sin desplegar');
        }
        for (const id of ['android-midi-usb', 'android-midi-bluetooth']) {
            if (!document.getElementById(id).getClientRects().length) {
                throw Error('Búsqueda MIDI no visible: ' + id);
            }
        }
        document.getElementById('android-midi-usb').click();
    })()`);
    await until(() => evaluate(`(() => {
        const add = document.getElementById('android-midi-add');
        return add.getClientRects().length && document.getElementById('android-midi-status').textContent;
    })()`), 'No respondió la comprobación USB de MIDI');
    const keyboards = await evaluate(`window.__TAURI__.core.invoke('keyboard_devices')`);
    assert.ok(Array.isArray(keyboards), 'El servicio de teclados no devolvió una lista');
    for (const keyboard of keyboards) {
        assert.equal(typeof keyboard.id, 'string');
        assert.equal(typeof keyboard.name, 'string');
    }
    assert.ok(await evaluate(`(() => {
        const disclosure = document.getElementById('android-keyboard-shortcuts');
        disclosure.open = true;
        return disclosure.contains(document.getElementById('android-keyboard-device-list')) &&
            document.getElementById('android-keyboard-bluetooth').getClientRects().length > 0;
    })()`), 'Falta el acceso Bluetooth del teclado de escritura');
    await evaluate(`document.getElementById('android-keyboard-shortcuts').open = false`);
    const permissions = adb('shell', 'dumpsys', 'package', pkg);
    for (const permission of ['BLUETOOTH_SCAN', 'BLUETOOTH_CONNECT']) {
        const entry = permissions.split('\n').find(line => line.includes(permission + ': granted='));
        assert.match(entry ?? '', /granted=false/);
        assert.doesNotMatch(entry, /USER_SET|USER_FIXED/);
    }
}
