import test from 'node:test';
import assert from 'node:assert/strict';

let sequence = 0;
async function setup(t) {
    const listeners = new Map();
    const timers = new Map();
    let timerId = 0;
    t.mock.method(globalThis, 'setTimeout', fn => { timers.set(++timerId, fn); return timerId; });
    t.mock.method(globalThis, 'clearTimeout', id => timers.delete(id));
    const surface = { addEventListener(name, fn) {
        const handlers = listeners.get(name) ?? [];
        handlers.push(fn); listeners.set(name, handlers);
    } };
    globalThis.document = surface;
    globalThis.window = surface;
    globalThis.MouseEvent = class { constructor(type, props) { Object.assign(this, props, { type }); } };
    const target = {
        menus: 0,
        closest(selector) { return selector.includes('.grid-item') ? this : null; },
        contains(element) { return element === this; },
        dispatchEvent(event) { if (event.type === 'contextmenu') this.menus++; },
    };
    const { initTouchControls } = await import(`../src/js/ui/touchControls.js?test=${++sequence}`);
    initTouchControls();
    function send(type, props = {}) {
        const event = { target, pointerId: 1, pointerType: 'touch', isPrimary: true,
            clientX: 40, clientY: 50, preventDefault() { this.prevented = true; },
            stopImmediatePropagation() { this.stopped = true; }, ...props };
        for (const fn of listeners.get(type) ?? []) fn(event);
        return event;
    }
    function hold() { for (const [id, fn] of timers) { timers.delete(id); fn(); } }
    t.after(() => {
        delete globalThis.document; delete globalThis.window; delete globalThis.MouseEvent;
    });
    return { send, hold, target };
}

test('tap remains a click; it does not open a context menu', async t => {
    const { send, hold, target } = await setup(t);
    send('pointerdown'); send('pointerup'); hold();
    assert.equal(target.menus, 0);
    assert.equal(send('click').stopped, undefined);
});
test('holding opens one menu and consumes the click that could play sound', async t => {
    const { send, hold, target } = await setup(t);
    send('pointerdown'); hold(); send('pointerup');
    assert.equal(target.menus, 1);
    assert.equal(send('click').stopped, true);
    send('pointerdown'); send('pointerup');
    assert.equal(send('click').stopped, undefined);
});
for (const [label, type, props] of [
    ['scroll movement', 'pointermove', { clientX: 80 }],
    ['scroll event', 'scroll', {}],
    ['cancel', 'pointercancel', {}],
    ['second finger', 'pointerdown', { isPrimary: false, pointerId: 2 }],
    ['focus loss', 'blur', {}],
]) {
    test(`${label} cancels the pending menu`, async t => {
        const { send, hold, target } = await setup(t);
        send('pointerdown'); send(type, props); hold();
        assert.equal(target.menus, 0);
    });
}
test('mouse input does not acquire touch behavior', async t => {
    const { send, hold, target } = await setup(t);
    send('pointerdown', { pointerType: 'mouse' }); hold();
    assert.equal(target.menus, 0);
});
test('native touch menu is suppressed while the app menu is pending', async t => {
    const { send, hold, target } = await setup(t);
    send('pointerdown');
    assert.equal(send('contextmenu', { isTrusted: true }).stopped, true);
    hold(); assert.equal(target.menus, 1);
});
