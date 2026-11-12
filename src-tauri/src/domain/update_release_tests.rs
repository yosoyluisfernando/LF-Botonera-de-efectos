use super::*;

fn release(tag: &str, assets: &[&str]) -> GithubRelease {
    GithubRelease {
        tag_name: tag.into(),
        html_url: "https://github.com/release".into(),
        body: None,
        draft: false,
        prerelease: false,
        assets: assets
            .iter()
            .map(|name| GithubAsset {
                name: (*name).into(),
            })
            .collect(),
    }
}

#[test]
fn android_never_offers_a_desktop_only_release() {
    let latest = latest_for(
        vec![
            release("v2.0.0", &["App.msi"]),
            release("android-v1.6.0", &["LF-Botonera-1.6.0-android-arm64.apk"]),
        ],
        "android",
    )
    .unwrap();
    assert_eq!(latest.tag_name, "android-v1.6.0");
    assert!(latest_for(vec![release("v2.0.0", &["App.msi"])], "android").is_none());
}

#[test]
fn desktop_never_offers_an_android_only_release() {
    let releases = vec![
        release("android-v1.6.0", &["App.apk"]),
        release("v1.5.0", &["App.msi", "App.deb"]),
    ];
    assert_eq!(
        latest_for(releases.clone(), "windows").unwrap().tag_name,
        "v1.5.0"
    );
    assert_eq!(latest_for(releases, "linux").unwrap().tag_name, "v1.5.0");
}

#[test]
fn draft_prerelease_and_test_packages_are_excluded() {
    let mut draft = release("v2.0.0", &["App.apk"]);
    draft.draft = true;
    let mut prerelease = release("v2.0.0", &["App.apk"]);
    prerelease.prerelease = true;
    for candidate in [
        draft,
        prerelease,
        release("v2.0.0-beta.1", &["App.apk"]),
        release("v2.0.0", &["App-debug.apk"]),
        release("v2.0.0", &["App-beta.apk"]),
    ] {
        assert!(latest_for(vec![candidate], "android").is_none());
    }
}

#[test]
fn latest_is_chosen_by_version_not_api_order() {
    let latest = latest_for(
        vec![
            release("v1.5.0", &["App.apk"]),
            release("android-v1.6.0", &["App.apk"]),
            release("v1.4.0", &["App.apk"]),
        ],
        "android",
    )
    .unwrap();
    assert_eq!(latest.tag_name, "android-v1.6.0");
}

#[test]
fn version_comparison_handles_stable_and_prerelease_current_versions() {
    assert!(is_newer("1.6.0", "1.6.0-beta.5"));
    assert!(is_newer("android-v1.7.0", "1.6.0"));
    assert!(!is_newer("1.6.0", "1.6.0"));
    assert!(!is_newer("1.5.0", "1.6.0"));
    assert!(!is_newer("invalid", "1.6.0"));
}
