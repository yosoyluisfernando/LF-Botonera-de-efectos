use super::*;

#[test]
fn cache_key_changes_when_file_signature_changes() {
    let a = key("C:/A/song.mp3", 1, 10);
    let b = key("C:/A/song.mp3", 2, 10);
    let c = key("C:/A/song.mp3", 1, 11);
    assert_ne!(a, b);
    assert_ne!(a, c);
}

#[test]
fn size_cleanup_removes_oldest_first() {
    let mut index = Index::default();
    index.items.insert(
        "old".into(),
        IndexEntry {
            file: "old.wfc".into(),
            bytes: 60,
            last_used: 1,
            mtime: 1,
            size: 1,
        },
    );
    index.items.insert(
        "new".into(),
        IndexEntry {
            file: "new.wfc".into(),
            bytes: 60,
            last_used: 2,
            mtime: 1,
            size: 1,
        },
    );
    evict_over_budget(&mut index, 80);
    assert!(!index.items.contains_key("old"));
    assert!(index.items.contains_key("new"));
}

#[test]
fn stats_reflects_config_limits() {
    let mut index = Index::default();
    index.items.insert(
        "a".into(),
        IndexEntry {
            file: "a.wfc".into(),
            bytes: MB,
            last_used: 1,
            mtime: 1,
            size: 1,
        },
    );
    let cfg = WaveformCacheConfig {
        max_mb: 7,
        max_age_days: 31,
    };
    let stats = stats_from(&index, &cfg);
    assert_eq!(stats.count, 1);
    assert_eq!(stats.max_mb, 7);
    assert_eq!(stats.max_age_days, 31);
}
