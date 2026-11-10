import assert from 'node:assert/strict';

export async function verifyMidiUsb(evaluate, until, adb, pkg) {
    await evaluate(`(() => {
        document.querySelector('[data-target="s-atajos"]').click();
        document.getElementById('android-midi-add').open = true;
        document.getElementById('android-midi-usb').click();
    })()`);
    await until(() => evaluate(`(() => {
        const add = document.getElementById('android-midi-add');
        return add.getClientRects().length && document.getElementById('android-midi-status').textContent;
    })()`), 'No respondió la comprobación USB de MIDI');
    const permissions = adb('shell', 'dumpsys', 'package', pkg);
    for (const permission of ['BLUETOOTH_SCAN', 'BLUETOOTH_CONNECT']) {
        const entry = permissions.split('\n').find(line => line.includes(permission + ': granted='));
        assert.match(entry ?? '', /granted=false/);
        assert.doesNotMatch(entry, /USER_SET|USER_FIXED/);
    }
}
