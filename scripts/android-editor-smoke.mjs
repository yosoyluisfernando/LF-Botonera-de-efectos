import assert from 'node:assert/strict';

export async function verifyTouchEditor(evaluate, until, path) {
    await evaluate('setTimeout(() => location.reload(), 100); true');
    await until(() => evaluate(`(() => {
        const cell = document.querySelector('.grid-item[data-id]');
        if (!cell?.getClientRects().length) return false;
        cell.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true, clientX: 40, clientY: 90 }));
        document.getElementById('menu-editar-pista').click();
        return true;
    })()`), 'No se encontró el efecto en la rejilla');
    await until(() => evaluate(`!document.getElementById('track-editor-modal').classList.contains('hidden')
        && !document.getElementById('te-save').disabled`), 'No se abrió el editor de pista');
    const changed = await evaluate(`(() => {
        const wave = document.getElementById('te-wave-inner');
        const rect = wave.getBoundingClientRect();
        for (const [fraction,button] of [[0.25,'te-fijar-inicio'],[0.75,'te-fijar-fin']]) {
            const options = {pointerType:'touch',isPrimary:true,pointerId:1,
                clientX:rect.left + rect.width*fraction,clientY:rect.top + 30};
            wave.dispatchEvent(new PointerEvent('pointerdown',options));
            wave.dispatchEvent(new PointerEvent('pointerup',options));
            document.getElementById(button).click();
        }
        const gain = document.getElementById('te-gain');
        const before = Number(gain.value);
        document.querySelector('[data-te-step="te-gain"][data-step="-10"]').click();
        document.querySelector('[data-te-step="te-zoom"][data-step="1"]').click();
        return { gain: Number(gain.value) - before, zoom: Number(document.getElementById('te-zoom').value),
            stop: document.getElementById('te-save').getBoundingClientRect().bottom, height: innerHeight };
    })()`);
    assert.ok(Math.abs(changed.gain + 1) < 0.01);
    assert.equal(changed.zoom,1.5);
    assert.ok(changed.stop <= changed.height);
    await evaluate("document.getElementById('te-save').click(); true");
    await until(() => evaluate("document.getElementById('track-editor-modal').classList.contains('hidden')"), 'No se guardó el recorte');
    const meta = await evaluate(`window.__TAURI__.core.invoke('get_track_meta', {path:${JSON.stringify(path)}})`);
    assert.ok(Math.abs(meta.cue_start_s - 0.5) < 0.03 && Math.abs(meta.cue_end_s - 1.5) < 0.03, JSON.stringify(meta));
    console.log('Editor táctil: inicio, fin, ganancia y zoom verificados', changed);
}
