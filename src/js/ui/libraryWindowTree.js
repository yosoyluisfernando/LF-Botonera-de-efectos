/** Árbol lateral de categorías indexadas y almacenamiento local. */
import { invoke } from '../bridge/api.js';
import { t } from '../util/i18n.js';

let onCatalog = null;
let onDirectory = null;
let active = null;
const ICON_KIND = new Map([
    ['♫', 'music'], ['◈', 'effects'], ['▣', 'root'], ['■', 'folder'], ['▰', 'drive'],
]);
export function initLibraryWindowTree(callbacks) {
    onCatalog = callbacks.onCatalog;
    onDirectory = callbacks.onDirectory;
}

export async function refreshLibraryWindowTree() {
    const target = document.getElementById('library-tree');
    const [roots, drives] = await Promise.all([
        invoke('library_list_roots'),
        invoke('library_storage_roots'),
    ]);
    target.replaceChildren(
        category('effects', roots),
        category('music', roots),
        separator(),
        ...drives.map(drive => driveNode(drive)),
    );
}

function category(collection, roots) {
    const wrapper = document.createElement('div');
    const children = childContainer();
    const row = treeRow({
        label: t(`library.${collection}`),
        icon: collection === 'music' ? '♫' : '◈',
        expandable: true,
        select: () => onCatalog({ collection }),
        expand: () => toggle(children, row),
    });
    roots.filter(root => root.collection === collection)
        .forEach(root => children.appendChild(rootNode(root)));
    wrapper.append(row, children);
    return wrapper;
}

function rootNode(root) {
    const wrapper = document.createElement('div');
    const children = childContainer(true);
    const label = root.path.replace(/[\\/]+$/, '').split(/[\\/]/).at(-1) || root.path;
    const row = treeRow({
        label,
        title: root.path,
        icon: '▣',
        expandable: true,
        select: () => onCatalog({
            collection: root.collection,
            rootId: root.id,
            relativePrefix: '',
        }),
        expand: async () => {
            if (!children.dataset.loaded) {
                await loadCatalogFolders(children, root, '');
            }
            toggle(children, row);
        },
    });
    wrapper.append(row, children);
    return wrapper;
}

async function loadCatalogFolders(target, root, parent) {
    const folders = await invoke('library_folder_children', {
        rootId: root.id,
        relativePath: parent || null,
    });
    target.replaceChildren(...folders.map(folder => catalogFolder(root, folder)));
    target.dataset.loaded = 'true';
}

function catalogFolder(root, folder) {
    const wrapper = document.createElement('div');
    const children = childContainer(true);
    const row = treeRow({
        label: folder.name,
        icon: '■',
        expandable: true,
        select: () => onCatalog({
            collection: root.collection,
            rootId: root.id,
            relativePrefix: folder.relative_path,
        }),
        expand: async () => {
            if (!children.dataset.loaded) {
                await loadCatalogFolders(children, root, folder.relative_path);
            }
            toggle(children, row);
        },
    });
    wrapper.append(row, children);
    return wrapper;
}

function driveNode(drive) {
    const wrapper = document.createElement('div');
    const children = childContainer(true);
    const row = treeRow({
        label: drive.name,
        title: drive.path,
        icon: '▰',
        expandable: true,
        select: () => onDirectory(drive.path),
        expand: async () => {
            if (!children.dataset.loaded) await loadDiskFolders(children, drive.path);
            toggle(children, row);
        },
    });
    wrapper.append(row, children);
    return wrapper;
}

async function loadDiskFolders(target, path) {
    const entries = await invoke('library_read_directory', { path });
    target.replaceChildren(...entries.filter(entry => entry.is_directory)
        .map(entry => diskFolder(entry)));
    target.dataset.loaded = 'true';
}

function diskFolder(entry) {
    const wrapper = document.createElement('div');
    const children = childContainer(true);
    const row = treeRow({
        label: entry.name,
        title: entry.path,
        icon: '■',
        expandable: true,
        select: () => onDirectory(entry.path),
        expand: async () => {
            if (!children.dataset.loaded) await loadDiskFolders(children, entry.path);
            toggle(children, row);
        },
    });
    wrapper.append(row, children);
    return wrapper;
}

function treeRow(options) {
    const row = document.createElement('button');
    row.type = 'button';
    row.className = 'library-tree-row';
    row.setAttribute('role', 'treeitem');
    row.title = options.title || options.label;
    row.innerHTML = `<span class="library-tree-caret">${options.expandable ? '▸' : ''}</span>
      <span class="library-tree-icon ${ICON_KIND.get(options.icon)}"
            aria-hidden="true">${options.icon}</span>
      <span class="library-tree-label"></span>`;
    row.querySelector('.library-tree-label').textContent = options.label;
    row.addEventListener('click', () => {
        setActive(row);
        options.select();
    });
    row.addEventListener('dblclick', event => {
        if (!options.expandable) return;
        event.preventDefault();
        options.expand?.().catch(console.error);
    });
    row.querySelector('.library-tree-caret').addEventListener('click', event => {
        event.stopPropagation();
        options.expand?.().catch(console.error);
    });
    row.addEventListener('keydown', event => {
        if (event.key === 'ArrowRight') options.expand?.().catch(console.error);
    });
    return row;
}

function toggle(children, row) {
    const opening = children.classList.contains('hidden');
    children.classList.toggle('hidden', !opening);
    row.setAttribute('aria-expanded', String(opening));
    row.querySelector('.library-tree-caret').textContent = opening ? '▾' : '▸';
}

function setActive(row) {
    active?.classList.remove('active');
    active = row;
    active.classList.add('active');
}

function childContainer(hidden = false) {
    const element = document.createElement('div');
    element.className = `library-tree-children${hidden ? ' hidden' : ''}`;
    element.setAttribute('role', 'group');
    return element;
}

function separator() {
    const element = document.createElement('div');
    element.className = 'library-tree-separator';
    return element;
}
