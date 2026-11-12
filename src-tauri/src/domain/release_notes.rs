//! Notas de versión independientes para Android y escritorio.
const DESKTOP: &str = include_str!("../../../CHANGELOG.md");
const ANDROID: &str = include_str!("../../../CHANGELOG.android.md");

pub fn for_platform(version: &str, android: bool) -> String {
    extract(if android { ANDROID } else { DESKTOP }, version)
}

fn extract(changelog: &str, version: &str) -> String {
    if version.contains('-') {
        let pending = section(changelog, "[Sin publicar]");
        if pending
            .lines()
            .any(|line| !line.trim().is_empty() && line.trim() != "---")
        {
            return pending;
        }
    }
    section(changelog, &format!("[{version}]"))
}

fn section(changelog: &str, marker: &str) -> String {
    let header = format!("## {marker}");
    let mut lines = changelog
        .lines()
        .skip_while(|line| *line != header && !line.starts_with(&format!("{header} ")));
    if lines.next().is_none() {
        return String::new();
    }
    lines
        .take_while(|line| !line.starts_with("## "))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_notes_never_display_unreleased_changes() {
        let log =
            "## [Sin publicar]\n### Nuevo\nFuturo\n## [1.6.0] — fecha\n### Público\nDisponible";
        assert_eq!(extract(log, "1.6.0"), "### Público\nDisponible");
        assert_eq!(extract(log, "1.6.0-beta.5"), "### Nuevo\nFuturo");
    }

    #[test]
    fn platform_notes_do_not_mix_android_and_desktop() {
        let android = for_platform("1.6.0", true);
        assert!(android.contains("teléfonos y tablets"));
        assert!(!android.contains("### Corregido"));
        let desktop = for_platform(env!("CARGO_PKG_VERSION"), false);
        assert!(desktop.contains("Apariencia en Windows y Linux"));
        assert!(!desktop.contains("### Botonera de sonidos para teléfonos"));
    }

    #[test]
    fn unknown_versions_and_empty_unreleased_notes_are_safe() {
        assert!(extract("## [Sin publicar]\n\n## [1.0.0]\nNotas", "2.0.0").is_empty());
        assert_eq!(
            extract(
                "## [Sin publicar]\n\n## [1.0.0-beta.1]\nNotas",
                "1.0.0-beta.1"
            ),
            "Notas"
        );
    }
}
