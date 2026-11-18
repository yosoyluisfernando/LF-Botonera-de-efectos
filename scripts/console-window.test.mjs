import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';
const source=['consoleDock','consolePlacementMenu','consoleWindow'].map(name=>
    readFileSync(new URL(`../src/js/ui/${name}.js`,import.meta.url),'utf8'))
    .join('\n')
    .replace(/^import .*;\r?\n/gm,'').replace(/export /g,'');
function fixture({existing=true,mode='window',android=false,fail=false,creationFails=false}={}){
    const calls=[];let current=existing;
    const elements=new Map();
    const document={addEventListener:()=>{},activeElement:null};
    const element=id=>{
        if(elements.has(id))return elements.get(id);
        const classes=new Set(['console-modal','console-dock','console-placement-menu'].includes(id)?['hidden']:[]);
        const listeners=new Map();
        const e={
            dataset:{},attributes:new Map(),style:{setProperty:(...args)=>calls.push(['style',...args])},
            setAttribute(key,value){this.attributes.set(key,value);},
            getAttribute(key){return this.attributes.get(key);},
            classList:{add:value=>classes.add(value),remove:value=>classes.delete(value),contains:value=>classes.has(value)},
            addEventListener:(event,fn)=>listeners.set(event,fn),fire:(event,payload)=>listeners.get(event)(payload),
            focus(){document.activeElement=this;},append(child){child.parentElement=this;},
            querySelectorAll:()=>['left','right','window','modal'].map(mode=>element(mode)),
        };
        elements.set(id,e);return e;
    };
    const view=element('view');
    element('console-modal').append(view);
    for(const mode of ['left','right','window','modal'])element(mode).dataset.mode=mode;
    class WebviewWindow {
        static async getByLabel(label){calls.push(['find',label]);if(fail)throw Error('native unavailable');return current?{
            unminimize:async()=>calls.push(['restore']),show:async()=>calls.push(['show']),
            setFocus:async()=>calls.push(['focus']),close:async()=>{current=false;calls.push(['close']);},
        }:null;}
        constructor(label,options){calls.push(['create',label,options]);current=!creationFails;}
        async once(event,handler){
            if(event===(creationFails?'tauri://error':'tauri://created'))handler({payload:'native result'});
        }
    }
    Object.assign(document,{body:element('body'),getElementById:element,querySelector:()=>view});
    const context=vm.createContext({
        isAndroid:()=>android,invoke:async(cmd,args)=>{
            calls.push(['ipc',cmd,args]);
            if(cmd==='set_console_mode'&&args.width===undefined)mode=args.mode;
            return {console_mode:mode,console:{dock_width:420}};
        },
        listen:async()=>()=>{},t:key=>key,renderConsole:async()=>calls.push(['render']),
        initVolumeSync:async()=>{},
        clearConsole:()=>calls.push(['clear']),updateConsoleTick:()=>{},
        document,placeMenu:menu=>menu.classList.remove('hidden'),
        window:{__TAURI__:{webviewWindow:{WebviewWindow}}},
        console:{error:()=>{}},
    });
    vm.runInContext(source,context);
    const run=script=>vm.runInContext(script,context);
    return {calls,element,view,run,open:()=>run('openPreferred()'),async select(mode){
        const select=element('console-placement');select.value=mode;await select.fire('change');
    }};
}
test('existing console is restored, shown and focused on every click',async()=>{
    const f=fixture();await f.open();await f.open();
    assert.deepEqual(f.calls.filter(c=>['restore','show','focus'].includes(c[0])).map(c=>c[0]),['restore','show','focus','restore','show','focus']);
    assert.equal(f.calls.filter(c=>c[0]==='create').length,0);
    assert.equal(f.calls.filter(c=>c[0]==='render').length,0);
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
    for(const options of [{fail:true},{existing:false,creationFails:true}]){
        const f=fixture(options);await f.open();
        assert.ok(!f.element('console-modal').classList.contains('hidden'));
    }
});
test('Android does not open desktop console windows',async()=>{
    const f=fixture({android:true});await f.open();assert.equal(f.calls.length,0);
});
test('placement controls have console translations in all four locales',()=>{
    for(const language of ['es','en','pt-BR','pt-PT']){
        const path=new URL(`../src/public/i18n/${language}.json`,import.meta.url);
        const {console:labels}=JSON.parse(readFileSync(path,'utf8'));
        for(const key of ['placement','dock_left','dock_right','popout','dock','dock_width']){
            assert.ok(labels[key]?.trim(),`${language}: missing console.${key}`);
        }
    }
});
test('a primary click toggles a dock while preserving its side and width',async()=>{
    const f=fixture({mode:'right',existing:false});f.run('wire()');
    const dock=f.element('console-dock');
    const trigger=f.element('btn-console');
    await trigger.fire('click');
    assert.equal(f.view.parentElement,dock);
    assert.equal(dock.dataset.side,'right');
    assert.equal(f.element('console-dock-width').getAttribute('aria-valuenow'),420);
    assert.ok(f.element('console-modal').classList.contains('hidden'));
    await trigger.fire('click');
    assert.ok(dock.classList.contains('hidden'));
    await trigger.fire('click');
    assert.ok(!dock.classList.contains('hidden'));
    trigger.fire('contextmenu',{preventDefault(){}});
    assert.equal(trigger.getAttribute('aria-expanded'),'true');
    await f.element('left').fire('click');
    assert.equal(trigger.getAttribute('aria-expanded'),'false');
    assert.equal(dock.dataset.side,'left');
    await f.element('console-dock-width').fire('keydown',{key:'ArrowRight',preventDefault(){}});
    assert.equal(f.calls.at(-1)[2].width,436);
    await f.element('modal').fire('click');
    assert.equal(f.view.parentElement,f.element('console-modal'));
    assert.ok(dock.classList.contains('hidden'));
    assert.equal(f.calls.filter(c=>c[0]==='render').length,2);
    await f.select('right');
    assert.equal(f.view.parentElement,dock);
});
test('the toolbar menu returns a floating console to a dock',async()=>{
    const f=fixture();f.run('wire()');
    await f.element('btn-console').fire('click');
    assert.ok(f.calls.some(c=>c[0]==='focus'));
    await f.element('left').fire('click');
    assert.equal(f.calls.filter(c=>c[0]==='close').length,1);
    assert.equal(f.element('console-dock').dataset.side,'left');
    assert.ok(!f.element('console-dock').classList.contains('hidden'));
});
