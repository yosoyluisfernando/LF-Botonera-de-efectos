//! Lista cerrada: el servidor no resuelve rutas del sistema de archivos.
pub fn get(path: &str) -> Option<(&'static str, &'static [u8])> {
    let item: (&str, &str) = match path {
        "/" | "/remote.html" => ("text/html", include_str!("../../../../src/remote.html")),
        "/css/theme.css" => ("text/css", include_str!("../../../../src/css/theme.css")),
        "/css/remote.css" => ("text/css", include_str!("../../../../src/css/remote.css")),
        "/js/remote/browser.js" => ("text/javascript", include_str!("../../../../src/js/remote/browser.js")),
        "/js/remote/client.js" => ("text/javascript", include_str!("../../../../src/js/remote/client.js")),
        "/js/remote/view.js" => ("text/javascript", include_str!("../../../../src/js/remote/view.js")),
        "/js/util/i18n.js" => ("text/javascript", include_str!("../../../../src/js/util/i18n.js")),
        "/i18n/es.json" => ("application/json", include_str!("../../../../src/public/i18n/es.json")),
        "/i18n/en.json" => ("application/json", include_str!("../../../../src/public/i18n/en.json")),
        "/i18n/pt-BR.json" => ("application/json", include_str!("../../../../src/public/i18n/pt-BR.json")),
        "/i18n/pt-PT.json" => ("application/json", include_str!("../../../../src/public/i18n/pt-PT.json")),
        "/js/util/colorAdapter.js" => ("text/javascript", include_str!("../../../../src/js/util/colorAdapter.js")),
        _ => return None,
    };
    Some((item.0, item.1.as_bytes()))
}
