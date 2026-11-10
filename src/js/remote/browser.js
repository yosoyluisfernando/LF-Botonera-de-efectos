import { loadLanguage, t } from '../util/i18n.js';
import { mountClient } from './client.js';

const language = navigator.language;
await loadLanguage(language.startsWith('en') ? 'en'
    : language === 'pt-PT' ? 'pt-PT' : language.startsWith('pt') ? 'pt-BR' : 'es');
document.title = t('remote.title');
mountClient(document.getElementById('remote-client'), {
    browser: true,
    send: async (_address, token, request) => {
        const response = await fetch('/api', {
            method: 'POST', headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' },
            body: JSON.stringify(request), signal: AbortSignal.timeout(4000),
        });
        const result = await response.json();
        if (!response.ok) throw new Error(result.error || 'remote_connection');
        return result;
    },
});
