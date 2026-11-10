import { invoke, waitForTauri } from './api.js';

let capabilities;
export async function loadCapabilities() {
    await waitForTauri();
    capabilities = await invoke('get_platform_capabilities');
    document.body.dataset.platform = capabilities.platform;
    return capabilities;
}
export function platformCapabilities() { return capabilities; }
export function isAndroid() { return capabilities?.platform === 'android'; }
