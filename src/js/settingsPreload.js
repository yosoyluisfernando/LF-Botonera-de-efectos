/**
 * Archivo: settingsPreload.js
 * Propósito: control remoto del formulario de PRECARGA (Regla 4). NO contiene
 * lógica de precarga: solo (1) pinta los controles, (2) los rellena con la vista
 * que devuelve Rust y (3) manda las elecciones a Rust, que valida, convierte
 * (días→horas), aplica defaults y persiste. Reutilizable en el panel de Ajustes
 * y en el diálogo de primer arranque (consultas acotadas por contenedor para no
 * colisionar ids). Justificación del JS: en Tauri la UI es el webview; pintar un
 * formulario es trabajo intrínseco del frontend, no lógica de negocio.
 */
import '../css/preload.css';
import { invoke } from './api.js';
import { t } from './i18n.js';

const q = (root, name) => root.querySelector(`[data-pl="${name}"]`);

/** Inyecta el formulario (textos i18n) en `root` y conecta el mostrar/ocultar
 *  del TTL según la estrategia (pura presentación, no lógica). */
export function renderPreloadForm(root) {
    root.innerHTML = _template();
    const strat = q(root, 'strategy');
    const enabled = q(root, 'enabled');
    const sync = () => {
        q(root, 'options').disabled = !enabled.checked;
        q(root, 'ttl-row').classList.toggle('hidden', strat.value !== 'on_play');
    };
    enabled.addEventListener('change', sync);
    strat.addEventListener('change', sync);
    sync();
}

/** Rellena el formulario con la vista que entrega Rust (sin cálculos en JS). */
export function fillPreloadForm(root, view) {
    q(root, 'enabled').checked = !!view.enabled;
    q(root, 'ram').value = String(view.ram_budget_mb);
    q(root, 'dur').value = String(view.max_duration_s);
    q(root, 'strategy').value = view.strategy;
    q(root, 'evict-value').value = view.evict_value;
    q(root, 'evict-unit').value = view.evict_unit;
    q(root, 'options').disabled = !view.enabled;
    q(root, 'ttl-row').classList.toggle('hidden', view.strategy !== 'on_play');
}

/** Envía las elecciones a Rust, que valida y persiste (sin validar en JS). */
export function savePreloadFrom(root) {
    return invoke('set_preload_config', {
        enabled: q(root, 'enabled').checked,
        ramBudgetMb: parseInt(q(root, 'ram').value, 10),
        maxDurationS: parseInt(q(root, 'dur').value, 10),
        strategy: q(root, 'strategy').value,
        evictValue: parseInt(q(root, 'evict-value').value, 10) || 1,
        evictUnit: q(root, 'evict-unit').value,
    });
}

// ── Integración con el modal de Ajustes (panel #s-precarga) ──
const panel = () => document.getElementById('s-precarga');

/** Construye el panel de Ajustes una sola vez. */
export function initPreloadPanel() {
    renderPreloadForm(panel());
}

/** Rellena el panel al abrir Ajustes; pide la vista y las estadísticas a Rust
 *  (la UI solo muestra los números que Rust calcula). */
export async function loadPreloadPanel() {
    try {
        fillPreloadForm(panel(), await invoke('get_preload_config'));
        const s = await invoke('get_preload_stats');
        const el = panel().querySelector('[data-pl="stats"]');
        if (el) {
            el.textContent =
                `${t('preload.stats')}: ${s.used_mb.toFixed(1)} / ${s.budget_mb} MB · ${s.count} ${t('preload.files')}`;
        }
    } catch (e) {
        console.error('Error al cargar precarga:', e);
    }
}

/** Guarda el panel; lo llama el botón Guardar del modal de Ajustes. */
export function savePreload() {
    return savePreloadFrom(panel());
}

function _template() {
    return `
      <label class="checkbox-line"><input type="checkbox" data-pl="enabled"> <b>${t('preload.enable')}</b></label>
      <p class="hint">${t('preload.enable_hint')}</p>
      <hr class="modal-divider">
      <fieldset class="preload-options" data-pl="options">
        <div class="row" style="margin-top:0">
          <div class="col">
            <label>${t('preload.ram')}</label>
            <select data-pl="ram">
              <option value="32">32 MB</option>
              <option value="64">64 MB</option>
              <option value="128">128 MB</option>
              <option value="256">256 MB</option>
            </select>
          </div>
          <div class="col">
            <label>${t('preload.max_dur')}</label>
            <select data-pl="dur">
              <option value="5">${t('preload.dur_5')}</option>
              <option value="10">${t('preload.dur_10')}</option>
              <option value="15">${t('preload.dur_15')}</option>
              <option value="30">${t('preload.dur_30')}</option>
            </select>
          </div>
        </div>
        <div class="row">
          <div class="col">
            <label>${t('preload.strategy')}</label>
            <select data-pl="strategy">
              <option value="full_profile">${t('preload.strat_full')}</option>
              <option value="visible_tabs">${t('preload.strat_tabs')}</option>
              <option value="on_play">${t('preload.strat_onplay')}</option>
            </select>
          </div>
        </div>
        <div class="row" data-pl="ttl-row">
          <div class="col">
            <label>${t('preload.evict')}</label>
            <div class="file-input-group">
              <input type="number" data-pl="evict-value" min="1" max="365" value="3">
              <select data-pl="evict-unit" style="max-width:120px">
                <option value="hours">${t('preload.unit_hours')}</option>
                <option value="days">${t('preload.unit_days')}</option>
              </select>
            </div>
          </div>
        </div>
      </fieldset>
      <p class="hint" data-pl="stats" style="margin-top:10px"></p>`;
}
