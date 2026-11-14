import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';
const source=readFileSync(new URL('../src/js/ui/consoleWindow.js',import.meta.url),'utf8')
    .replace(/^import .*;\r?\n/gm,'').replace(/export /g,'');
function fixture({existing=true,mode='window',android=false,fail=false}={}){
    const calls=[];let current=existing;
    class WebviewWindow {
        static async getByLabel(label){calls.push(['find',label]);if(fail)throw Error('native unavailable');return current?{
            unminimize:async()=>calls.push(['restore']),show:async()=>calls.push(['show']),setFocus:async()=>calls.push(['focus'])
        }:null;}
        constructor(label,options){calls.push(['create',label,options]);current=true;}
    }
    const context=vm.createContext({
        isAndroid:()=>android,invoke:async(cmd)=>{calls.push(['ipc',cmd]);return {console_mode:mode};},
        listen:async()=>()=>{},t:key=>key,renderConsole:async()=>calls.push(['render']),clearConsole:()=>{},updateConsoleTick:()=>{},
        document:{body:{classList:{contains:()=>false}},getElementById:id=>({classList:{remove:()=>calls.push(['modal',id])}})},
        window:{__TAURI__:{webviewWindow:{WebviewWindow}}},console:{error:()=>{}},
    });
    vm.runInContext(source,context);
    return {calls,open:()=>vm.runInContext('openPreferred()',context)};
}
test('existing console is restored, shown and focused on every click',async()=>{
    const f=fixture();await f.open();await f.open();
    assert.deepEqual(f.calls.filter(c=>['restore','show','focus'].includes(c[0])).map(c=>c[0]),['restore','show','focus','restore','show','focus']);
    assert.equal(f.calls.filter(c=>c[0]==='create').length,0);
    assert.equal(f.calls.filter(c=>c[0]==='render'||c[0]==='modal').length,0);
});
test('closed console is created once and subsequent clicks focus it',async()=>{
    const f=fixture({existing:false});await f.open();await f.open();
    assert.equal(f.calls.filter(c=>c[0]==='create').length,1);
    assert.equal(f.calls.find(c=>c[0]==='create')[1],'audio-console');
    assert.ok(f.calls.some(c=>c[0]==='focus'));
});
test('modal preference retains the existing modal path',async()=>{
    const f=fixture({mode:'modal'});await f.open();
    assert.ok(f.calls.some(c=>c[0]==='render'));assert.ok(!f.calls.some(c=>c[0]==='find'));
});
test('native-window failure retains modal fallback',async()=>{
    const f=fixture({fail:true});await f.open();assert.ok(f.calls.some(c=>c[0]==='modal'));
});
test('Android does not open desktop console windows',async()=>{
    const f=fixture({android:true});await f.open();assert.equal(f.calls.length,0);
});
