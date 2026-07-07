/**
 * Archivo: editTypes.js
 * Proposito: metadatos que Rust entrega para dibujar tipos del editor.
 */

const FALLBACK = {
    id: 'audio',
    label_key: 'edit_modal.type_audio',
    placeholder_key: 'edit_modal.placeholder_audio',
    is_folder: false,
    is_locution: false,
    can_prelisten: true,
    default_folder: '',
};

let _state = { selected_type: 'audio', options: [FALLBACK] };

/** Guarda los tipos permitidos devueltos por Rust para esta apertura del editor. */
export function setTypeState(state) {
    _state = state?.options?.length ? state : { selected_type: 'audio', options: [FALLBACK] };
}

export const selectedType = () => _state.selected_type || 'audio';
export const typeOptions = () => _state.options ?? [FALLBACK];
export const typeMeta = type => typeOptions().find(opt => opt.id === type) ?? FALLBACK;
export const isLocution = type => !!typeMeta(type).is_locution;
export const isFolderType = type => !!typeMeta(type).is_folder;
export const canPrelisten = type => !!typeMeta(type).can_prelisten;
export const placeholderKey = type => typeMeta(type).placeholder_key;
export const defaultFolder = type => typeMeta(type).default_folder ?? '';
export const labelKey = type => typeMeta(type).label_key;
