/**
 * Archivo: i18n.js
 * Propósito: Carga los archivos JSON de idioma y aplica traducciones al DOM.
 * Cumple Regla 6: Internacionalización Inmediata — cero strings hardcoded en la UI.
 */

let currentLangData = {};

/**
 * Carga el JSON del idioma y aplica las traducciones a todos los elementos
 * data-i18n, data-i18n-placeholder y data-i18n-title del DOM.
 * @param {string} lang Código de idioma (ej: 'es', 'en').
 */
export async function loadLanguage(lang = 'es') {
    try {
        const response = await fetch(`/i18n/${lang}.json`);
        if (!response.ok) throw new Error(`Archivo de idioma no encontrado: ${lang}`);
        currentLangData = await response.json();
        applyTranslations();
    } catch (error) {
        console.error("Error cargando idioma:", error);
    }
}

/**
 * Traduce una clave puntual sin modificar el DOM.
 * @param {string} key Clave anidada con notación de punto.
 * @returns {string} Texto traducido, o la propia clave si no se encuentra.
 */
export function t(key) {
    return getNestedValue(currentLangData, key) ?? key;
}

/** Aplica traducciones a todos los elementos con atributos data-i18n*. */
function applyTranslations() {
    document.querySelectorAll('[data-i18n]').forEach(el => {
        const val = getNestedValue(currentLangData, el.getAttribute('data-i18n'));
        if (val) el.textContent = val;
    });
    document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
        const val = getNestedValue(currentLangData, el.getAttribute('data-i18n-placeholder'));
        if (val) el.placeholder = val;
    });
    document.querySelectorAll('[data-i18n-title]').forEach(el => {
        const val = getNestedValue(currentLangData, el.getAttribute('data-i18n-title'));
        if (val) el.title = val;
    });
}

/** Navega un objeto anidado usando notación de punto (ej: "app.title"). */
function getNestedValue(obj, path) {
    return path.split('.').reduce((acc, part) => acc?.[part], obj);
}
