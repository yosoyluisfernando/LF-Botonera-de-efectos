use super::*;
#[test]
fn cache_identity_changes_with_original_and_preserves_source() {
    let dir = std::env::temp_dir().join(format!("lf-convert-cache-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("source.m4a");
    std::fs::write(&source, b"original").unwrap();
    let name = source.to_str().unwrap();
    let first = target_in(&dir, name).unwrap();
    assert_eq!(first, target_in(&dir, name).unwrap());
    assert_eq!(first.extension().unwrap(), "flac");
    std::fs::write(&source, b"changed original").unwrap();
    assert_ne!(first, target_in(&dir, name).unwrap());
    assert_eq!(std::fs::read(&source).unwrap(), b"changed original");
    std::fs::remove_file(&source).unwrap();
    assert!(target_in(&dir, name).is_none());
    let _ = std::fs::remove_dir(&dir);
}
