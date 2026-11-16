import { placeMenu } from '../util/menuPosition.js';

export function wireConsolePlacementMenu(trigger, menu, changePlacement) {
    const items = [...menu.querySelectorAll('[data-mode]')];
    const close = () => {
        menu.classList.add('hidden');
        trigger.setAttribute('aria-expanded', 'false');
    };
    const open = (x, y) => {
        placeMenu(menu, x, y);
        trigger.setAttribute('aria-expanded', 'true');
        items[0].focus();
    };
    trigger.addEventListener('contextmenu', event => {
        event.preventDefault();
        open(event.clientX, event.clientY);
    });
    trigger.addEventListener('keydown', event => {
        if (event.key !== 'ContextMenu' && !(event.shiftKey && event.key === 'F10')) return;
        event.preventDefault();
        const rect = trigger.getBoundingClientRect();
        open(rect.left, rect.bottom);
    });
    items.forEach(item => item.addEventListener('click', () => {
        close();
        trigger.focus();
        return changePlacement(item.dataset.mode).catch(console.error);
    }));
    menu.addEventListener('keydown', event => {
        const index = items.indexOf(document.activeElement);
        switch (event.key) {
            case 'ArrowDown': items[(index + 1) % items.length].focus(); break;
            case 'ArrowUp': items[(index + items.length - 1) % items.length].focus(); break;
            case 'Home': items[0].focus(); break;
            case 'End': items.at(-1).focus(); break;
            case 'Enter':
            case ' ': document.activeElement.click(); break;
            case 'Escape': close(); trigger.focus(); break;
            case 'Tab': close(); trigger.focus(); return;
            default: return;
        }
        event.preventDefault();
        event.stopPropagation();
    });
    document.addEventListener('pointerdown', event => {
        if (!menu.classList.contains('hidden') && !menu.contains(event.target)) close();
    });
}
