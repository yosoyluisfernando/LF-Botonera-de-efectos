/// Canal grabado en el ejecutable durante la compilación.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistributionChannel {
    Direct,
    Store,
}

impl DistributionChannel {
    pub fn code(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Store => "store",
        }
    }

    pub fn update_manager(self) -> &'static str {
        match self {
            Self::Direct => "github_releases",
            Self::Store => "microsoft_store",
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
    match std::env::consts::OS {
        "windows" => "windows",
        "linux" => "linux",
        "macos" => "macos",
        _ => "other",
    }
}

fn parse_channel(value: Option<&str>) -> DistributionChannel {
    match value {
        Some(channel) if channel.eq_ignore_ascii_case("store") => DistributionChannel::Store,
        _ => DistributionChannel::Direct,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_channel, DistributionChannel};

    #[test]
    fn store_is_explicit_and_every_current_direct_build_stays_direct() {
        assert_eq!(parse_channel(Some("store")), DistributionChannel::Store);
        assert_eq!(parse_channel(Some("STORE")), DistributionChannel::Store);
        assert_eq!(parse_channel(Some("direct")), DistributionChannel::Direct);
        assert_eq!(parse_channel(None), DistributionChannel::Direct);
    }
}
