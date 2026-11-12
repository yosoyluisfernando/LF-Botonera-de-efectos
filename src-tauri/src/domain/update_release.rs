//! Selección de versiones publicadas que realmente distribuyen la plataforma.
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct GithubAsset {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub html_url: String,
    pub body: Option<String>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub assets: Vec<GithubAsset>,
}

pub fn latest_for(releases: Vec<GithubRelease>, platform: &str) -> Option<GithubRelease> {
    releases
        .into_iter()
        .filter(|release| {
            !release.draft
                && !release.prerelease
                && stable_version(&release.tag_name).is_some()
                && release
                    .assets
                    .iter()
                    .any(|asset| matches_platform(&asset.name, platform))
        })
        .max_by_key(|release| stable_version(&release.tag_name))
}

fn matches_platform(name: &str, platform: &str) -> bool {
    let name = name.to_ascii_lowercase();
    if name.contains("beta") || name.contains("alpha") || name.contains("debug") {
        return false;
    }
    match platform {
        "android" => name.ends_with(".apk"),
        "windows" => name.ends_with(".msi") || name.ends_with(".exe"),
        "linux" => [".deb", ".rpm", ".appimage"]
            .iter()
            .any(|ext| name.ends_with(ext)),
        "macos" => name.ends_with(".dmg") || name.ends_with(".app.tar.gz"),
        _ => false,
    }
}

pub fn normalize_version(value: &str) -> &str {
    value
        .trim()
        .strip_prefix("android-")
        .unwrap_or(value.trim())
        .trim_start_matches('v')
}

fn stable_version(value: &str) -> Option<(u32, u32, u32)> {
    let mut parts = normalize_version(value).split('.');
    let version = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    parts.next().is_none().then_some(version)
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    let Some(latest) = stable_version(latest) else {
        return false;
    };
    let current_base = current.split('-').next().unwrap_or(current);
    let Some(base) = stable_version(current_base) else {
        return false;
    };
    latest > base || (latest == base && current.contains('-'))
}

#[cfg(test)]
#[path = "update_release_tests.rs"]
mod tests;
