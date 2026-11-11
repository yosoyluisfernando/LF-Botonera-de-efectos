import assert from 'node:assert/strict';
import { dismissStartup } from './android-toolbar-smoke.mjs';

export async function verifyExportRoundtrip({ evaluate, until, adb, pkg, tapDocument }, original) {
    const invoke = (command, args = {}) => evaluate(
        'window.__TAURI__.core.invoke(' + JSON.stringify(command) + ',' + JSON.stringify(args) + ')');
    const config = await invoke('get_config');
    const firstProfile = config.active_profile_id;


    async function exportFile(command) {
        const operation = invoke(command);
        operation.catch(() => {});
        await until(() => tapDocument('Save', 'SAVE') || (tapDocument('Download', 'Downloads'), false),
            'El selector no ofreció guardar la exportación');
        await operation;
    }

    async function importFile(command, name) {
        const operation = invoke(command);
        operation.catch(() => {});
        await until(() => tapDocument(name, name.replace(/\.(bdelf|bdeplf)$/, '')) || (tapDocument('Download', 'Downloads'), false)
            || (tapDocument('Show roots', 'Open navigation drawer', 'Browse'), false),
            'No se encontró la exportación ' + name);
        return operation;
    }

    async function verifyPlayback(label) {
        const config = await invoke('get_config');
        const profile = config.profiles.find(p => p.id === config.active_profile_id);
        const palette = profile.paletas.find(p => p.botones.some(b => b.path === original.path));
        assert.ok(palette, 'El perfil importado perdió el MP3');
        await invoke('set_active_paleta', { profileId: profile.id, paletaId: palette.id });
        const grid = await invoke('get_grid_state');
        const button = grid.buttons.find(b => b.index === 1);
        assert.equal(button.path, original.path);
        assert.ok(button.duration > 0, JSON.stringify(button));
        await invoke('stop_all_audio');
        await evaluate(`(async () => {
            window.__roundtrip = { id: ${JSON.stringify(button.id)}, playing: false, peak: 0 };
            window.__roundtripAudio = await window.__TAURI__.event.listen('audio-tick', ({payload}) => {
                if (payload.buttons.some(b => b.id === window.__roundtrip.id)) window.__roundtrip.playing = true;
            });
            window.__roundtripMeter = await window.__TAURI__.event.listen('meter-tick', ({payload}) => {
                window.__roundtrip.peak = Math.max(window.__roundtrip.peak, payload.master_level_l, payload.master_level_r);
            });
        })()`);
        await invoke('play_button', { id: button.id });
        const evidence = await until(() => evaluate(
            'window.__roundtrip.playing && window.__roundtrip.peak > 0.001 ? window.__roundtrip : false'),
            'El audio importado no produjo señal en la salida: ' + label);
        await evaluate("window.__roundtripAudio(); window.__roundtripMeter()");
        const meta = await invoke('get_track_meta', { path: button.path });
        assert.equal(meta.cue_start_s, originalMeta.cue_start_s);
        assert.equal(meta.cue_end_s, originalMeta.cue_end_s);
        assert.equal(meta.gain_db, originalMeta.gain_db);
        console.log('Reproducción real tras importación', label, evidence);
        const updated = await invoke('get_config');
        const ids = updated.profiles.flatMap(p => p.paletas.flatMap(t => t.botones.map(b => b.id)));
        assert.equal(ids.length, new Set(ids).size, 'Identificadores de botones repetidos');
        return grid;
    }

    await dismissStartup(evaluate, until);
    const assignment = invoke('assign_file_to_button', { index: 1, path: null });
    assignment.catch(() => {});
    await until(() => tapDocument('LF-Roundtrip.mp3', 'LF-Roundtrip')
        || (tapDocument('Download', 'Downloads'), false)
        || (tapDocument('Show roots', 'Open navigation drawer', 'Browse'), false),
        'No se encontró el MP3 de prueba');
    original = (await assignment).buttons.find(b => b.index === 1);
    await invoke('analyze_track', { path: original.path });
    await invoke('set_track_cue', { path: original.path, startS: 0.5, endS: 1.5 });
    await invoke('set_track_gain', { path: original.path, gainDb: -1 });
    const originalMeta = await invoke('get_track_meta', { path: original.path });
    await exportFile('export_tab');
    await importFile('import_tab', 'Principal.bdelf');
    await verifyPlayback('pestaña');
    await exportFile('export_profile');
    await invoke('set_master_volume', { volume: 0 });
    await importFile('import_profile', 'Perfil 1.bdeplf');
    assert.equal((await invoke('get_console_view')).program.fader, 1);
    await verifyPlayback('perfil completo');
    await importFile('import_tab', 'Principal.bdelf');
    const current = await invoke('get_config');
    const active = current.profiles.find(p => p.id === current.active_profile_id);
    await invoke('delete_paleta', {
        profileId: active.id, paletaId: active.active_paleta_id,
    });
    await importFile('import_tab', 'Principal.bdelf');
    await verifyPlayback('reimportación después de borrar una pestaña');
    await invoke('delete_profile', { id: firstProfile });
    await verifyPlayback('después de borrar el perfil original');
    await evaluate('setTimeout(() => location.reload(), 100); true');
    await until(() => evaluate("!!document.querySelector('.grid-item[data-id]')?.getClientRects().length"),
        'La rejilla no reapareció después de recargar');
    await dismissStartup(evaluate, until);
    await verifyPlayback('después de recargar');
    console.log('Exportación e importación de pestaña y perfil con rutas, recortes y señal comprobados.');
}
