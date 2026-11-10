import assert from 'node:assert/strict';

export async function verifyAndroidMenus(evaluate) {
    const result = await evaluate(`(() => {
        const rect = id => document.getElementById(id).getBoundingClientRect().toJSON();
        const cell = document.querySelector('.grid-item');
        cell.dispatchEvent(new MouseEvent('contextmenu', {
            bubbles: true, cancelable: true, clientX: innerWidth - 2, clientY: innerHeight - 2
        }));
        const menu = document.getElementById('context-menu');
        const menuRect = rect('context-menu');
        const area = getComputedStyle(document.querySelector('.android-safe-area-probe'));
        const right = Number.parseFloat(area.paddingRight) || 0;
        const bottom = Number.parseFloat(area.paddingBottom) || 0;
        const left = Number.parseFloat(area.paddingLeft) || 0;
        const reserve = right || bottom || left ? 0 : 48;
        menu.classList.add('hidden');
        document.getElementById('master-volume-btn').click();
        const result = {menu:menuRect, count:rect('playback-countdown'),
            stop:rect('pb-btn-stop-all'), panel:rect('master-volume-panel'), slider:rect('master-volume-slider'),
            right: innerWidth - Math.max(right, innerWidth > innerHeight ? reserve : 0),
            bottom: innerHeight - Math.max(bottom,reserve), height:innerHeight, width:innerWidth};
        document.getElementById('master-volume-btn').click();
        return result;
    })()`);
    assert.ok(result.menu.height > 0 && result.menu.right <= result.right - 7
        && result.menu.bottom <= result.bottom - 7, JSON.stringify(result));
    assert.ok(result.panel.top >= result.count.bottom && result.panel.top >= result.stop.bottom - 1
        && result.panel.bottom <= result.height, JSON.stringify(result));
    assert.ok(result.slider.width > 100 && result.slider.right <= result.width, JSON.stringify(result));
    console.log('Menú seguro y volumen visible', result.width, result.height);
}
