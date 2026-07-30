/** Arranque exclusivo de la ventana Biblioteca. */
import { invoke, listen, waitForTauri } from './bridge/api.js';
import { loadLanguage, t } from './util/i18n.js';
import { applyTheme } from './ui/theme.js';
import { initLibraryIndexProgress } from './ui/libraryIndexProgress.js';
import { initLibraryRootsModal } from './ui/libraryRootsModal.js';
import { initBackupRestore } from './ui/backupRestoreModal.js';
import {
    initLibraryDisplayMode, refreshLibraryDisplayMode,
} from './ui/libraryDisplayMode.js';
import {
    initLibraryWindowTree, refreshLibraryWindowTree,
} from './ui/libraryWindowTree.js';
import {
    initLibraryWindowTable, refreshLibraryWindowTable,
    repaintLibraryWindowTable, showCatalog, showDirectory,
} from './ui/libraryWindowTable.js';

start().catch(showFatal);

async function start() {
    await waitForTauri();
    const config = await invoke('get_config');
    applyTheme(config.theme || 'dark');
    await loadLanguage(config.language || 'es');
    document.title = t('library.window_title');
    await listen('theme-changed', event =>
        applyTheme(event.payload?.theme || 'dark'));
    await listen('library-metadata-changed', () =>
        refreshLibraryWindowTable().catch(showStatusError));
    document.addEventListener('contextmenu', event => event.preventDefault(), true);
    initLibraryWindowTable();
    initLibraryWindowTree({
        onCatalog: scope => showCatalog(scope).catch(showStatusError),
        onDirectory: path => showDirectory(path).catch(showStatusError),
    });
    initLibraryIndexProgress();
    initLibraryRootsModal({
        onComplete: async () => {
            await Promise.all([
                refreshLibraryWindowTree(),
                showCatalog({}),
            ]);
        },
    });
    await initBackupRestore({
        buttonId: 'library-backup-open',
        checkResult: true,
    });
    initLibraryDisplayMode(() => repaintLibraryWindowTable());
    const panel = await invoke('get_fixed_panel');
    refreshLibraryDisplayMode(panel.settings);
    wireToolbar();
    await Promise.all([refreshLibraryWindowTree(), showCatalog({})]);
    document.getElementById('loading-screen')?.classList.add('hidden');
    document.getElementById('library-window').classList.remove('hidden');
}

function wireToolbar() {
    document.getElementById('library-refresh').addEventListener('click', async () => {
        await Promise.all([
            refreshLibraryWindowTree(),
            refreshLibraryWindowTable(),
        ]).catch(showStatusError);
    });
    document.getElementById('library-show-all').addEventListener('click', () => {
        document.getElementById('library-window-search').value = '';
        showCatalog({}).catch(showStatusError);
    });
    document.querySelector('#library-roots-modal .close-btn')
        .addEventListener('click', () =>
            document.getElementById('library-roots-hide').click());
}

function showStatusError(error) {
    document.getElementById('library-window-status').textContent = String(error);
}

async function showFatal(error) {
    await loadLanguage('es').catch(() => {});
    const loading = document.getElementById('loading-screen');
    if (loading) loading.textContent = `${t('errors.fatal_ipc')}: ${error}`;
}
