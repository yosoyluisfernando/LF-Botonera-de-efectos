//! Diseño de escritorio, independiente del modo de color.
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopStyle {
    #[default]
    #[serde(alias = "studio", alias = "essential")]
    Default,
    Modern,
}

impl DesktopStyle {
    pub fn is_default(&self) -> bool {
        *self == Self::Default
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "default" => Ok(Self::Default),
            "modern" => Ok(Self::Modern),
            _ => Err("invalid_desktop_style".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AppConfig;

    #[test]
    fn old_config_keeps_default_style() {
        let config: AppConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(config.desktop_style, DesktopStyle::Default);
        assert!(serde_json::to_value(config)
            .unwrap()
            .get("desktop_style")
            .is_none());
    }

    #[test]
    fn retired_experiments_do_not_invalidate_existing_preferences() {
        for value in ["studio", "essential"] {
            let json =
                serde_json::json!({"desktop_style": value, "theme": "light", "language": "pt-BR"});
            let config: AppConfig = serde_json::from_value(json).unwrap();
            assert_eq!(config.desktop_style, DesktopStyle::Default);
            assert_eq!(config.theme, "light");
            assert_eq!(config.language, "pt-BR");
            assert!(DesktopStyle::parse(value).is_err());
        }
    }

    #[test]
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    fn modern_style_round_trips_with_color_mode() {
        let config = AppConfig {
            desktop_style: DesktopStyle::Modern,
            theme: "system".into(),
            ..Default::default()
        };
        let loaded: AppConfig =
            serde_json::from_value(serde_json::to_value(config).unwrap()).unwrap();
        assert_eq!(loaded.desktop_style, DesktopStyle::Modern);
        assert_eq!(loaded.theme, "system");
        assert!(DesktopStyle::parse("unknown").is_err());
    }

    #[test]
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    fn unsupported_platforms_ignore_the_style_field() {
        let mut config: AppConfig =
            serde_json::from_value(serde_json::json!({"desktop_style":"modern"})).unwrap();
        assert_eq!(config.desktop_style, DesktopStyle::Default);
        config.desktop_style = DesktopStyle::Modern;
        assert!(serde_json::to_value(config)
            .unwrap()
            .get("desktop_style")
            .is_none());
    }
}
