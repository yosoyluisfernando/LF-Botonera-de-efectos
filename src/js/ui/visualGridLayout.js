/** Calcula la geometría vertical de cada categoría de una cuadrícula virtual. */
export function buildVisualGroupLayouts(groups, columns, headerHeight, itemHeight) {
    let start = 0;
    let y = 0;
    return groups.map(group => {
        const rows = Math.ceil(group.count / columns);
        const layout = {
            ...group,
            start,
            end: start + group.count,
            headerY: y,
            itemsY: y + headerHeight,
            endY: y + headerHeight + rows * itemHeight,
        };
        start = layout.end;
        y = layout.endY;
        return layout;
    });
}
