/// Canal de actualización; Android también consulta el instalador registrado.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistributionChannel {
    Direct,
    Store,
    PlayStore,
    FDroid,
}

impl DistributionChannel {
    pub fn code(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Store => "store",
            Self::PlayStore => "play_store",
            Self::FDroid => "fdroid",
        }
    }

    pub fn update_manager(self) -> &'static str {
        match self {
            Self::Direct => "github_releases",
            Self::Store => "microsoft_store",
            Self::PlayStore => "google_play",
            Self::FDroid => "fdroid",
        }
    }

    pub fn managed_externally(self) -> bool {
        self != Self::Direct
    }
}

pub fn current_channel() -> DistributionChannel {
    parse_channel(option_env!("LF_DISTRIBUTION_CHANNEL"))
}

pub fn current_platform() -> &'static str {
    platform_for(std::env::consts::OS)
}

fn platform_for(os: &str) -> &'static str {
    match os {
        "windows" => "windows",
        "linux" => "linux",
        "macos" => "macos",
        "android" => "android",
        _ => "other",
    }
}

pub fn android_channel(installer: &str, configured: Option<&str>) -> DistributionChannel {
    match installer {
        "com.android.vending" => DistributionChannel::PlayStore,
        "org.fdroid.fdroid" | "org.fdroid.basic" | "org.fdroid.fdroid.privileged" => {
            DistributionChannel::FDroid
        }
        _ => match parse_channel(configured) {
            DistributionChannel::Store => DistributionChannel::Direct,
            channel => channel,
        },
    }
}

fn parse_channel(value: Option<&str>) -> DistributionChannel {
    match value.map(str::to_ascii_lowercase).as_deref() {
        Some("store") => DistributionChannel::Store,
        Some("play_store") => DistributionChannel::PlayStore,
        Some("fdroid") => DistributionChannel::FDroid,
        _ => DistributionChannel::Direct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_distribution_and_desktop_store_remain_explicit() {
        assert_eq!(parse_channel(Some("STORE")), DistributionChannel::Store);
        assert_eq!(parse_channel(Some("direct")), DistributionChannel::Direct);
        assert_eq!(parse_channel(None), DistributionChannel::Direct);
    }

    #[test]
    fn android_has_its_own_platform_name() {
        assert_eq!(platform_for("android"), "android");
        assert_eq!(platform_for("windows"), "windows");
        assert_eq!(platform_for("unknown"), "other");
    }

    #[test]
    fn actual_android_installer_takes_precedence_over_build_fallback() {
        assert_eq!(
            android_channel("com.android.vending", Some("fdroid")),
            DistributionChannel::PlayStore
        );
        for installer in [
            "org.fdroid.fdroid",
            "org.fdroid.basic",
            "org.fdroid.fdroid.privileged",
        ] {
            assert_eq!(
                android_channel(installer, None),
                DistributionChannel::FDroid
            );
        }
        assert_eq!(
            android_channel("com.android.packageinstaller", None),
            DistributionChannel::Direct
        );
        assert_eq!(
            android_channel("", Some("store")),
            DistributionChannel::Direct
        );
        assert_eq!(
            android_channel("", Some("fdroid")),
            DistributionChannel::FDroid
        );
    }

    #[test]
    fn store_channels_delegate_updates_to_their_own_manager() {
        for channel in [
            DistributionChannel::Store,
            DistributionChannel::PlayStore,
            DistributionChannel::FDroid,
        ] {
            assert!(channel.managed_externally());
        }
        assert!(!DistributionChannel::Direct.managed_externally());
        assert_eq!(
            DistributionChannel::PlayStore.update_manager(),
            "google_play"
        );
        assert_eq!(DistributionChannel::FDroid.code(), "fdroid");
    }
}
