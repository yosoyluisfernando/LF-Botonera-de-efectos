import test from 'node:test';
import assert from 'node:assert/strict';
import { initTabWheel } from '../src/js/ui/tabWheel.js';
function bar(cycle) {
    const element = { dataset: {}, clientHeight: 50, addEventListener(type, callback, options) {
        assert.equal(type, 'wheel'); assert.equal(options.passive, false); this.wheel = callback;
    }};
    initTabWheel(element, cycle);
    return element;
}
function event(deltaY, extra = {}) {
    return { deltaY, deltaX: 0, deltaMode: 0, prevented: false,
        preventDefault() { this.prevented = true; }, ...extra };
}
test('wheel selects direction while leaving horizontal scroll and modifiers untouched', async () => {
    for (const [delta, expected] of [[80, 1], [-80, -1]]) {
        const calls = []; const element = bar(async value => calls.push(value));
        const e = event(delta); await element.wheel(e);
        assert.deepEqual(calls, [expected]); assert.ok(e.prevented);
    }
    for (const extra of [{shiftKey:true}, {ctrlKey:true}, {altKey:true}, {deltaX:100}]) {
        const element = bar(() => assert.fail('must not switch'));
        const e = event(80, extra); await element.wheel(e); assert.equal(e.prevented, false);
    }
});
test('small trackpad deltas accumulate and reversing direction discards old movement', async () => {
    const calls=[];const element=bar(async value=>calls.push(value));
    await element.wheel(event(30));await element.wheel(event(-20));
    assert.deepEqual(calls, []);await element.wheel(event(-20));assert.deepEqual(calls, [-1]);
});
test('line-based wheels normalize units and an in-flight change is not queued again', async () => {
    let finish; const pending=new Promise(resolve=>{finish=resolve;});let calls=0;
    const element=bar(async()=>{calls++;await pending;});
    const first=element.wheel(event(3,{deltaMode:1}));
    await element.wheel(event(120));assert.equal(calls,1);finish();await first;
});
test('initializing twice leaves one listener and keeps the original callback', async()=>{
    let count=0;const element=bar(async()=>{count++;});
    initTabWheel(element,()=>assert.fail('duplicate handler'));await element.wheel(event(80));
    assert.equal(count,1);
});
