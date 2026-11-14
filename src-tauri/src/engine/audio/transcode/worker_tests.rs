use super::*;
#[test]
fn concurrent_requests_are_enqueued_once_and_completion_is_not_overwritten() {
    let state = Arc::new(State::default());
    let target = PathBuf::from("same-stamp.flac");
    let threads: Vec<_> = (0..12)
        .map(|_| {
            let state = Arc::clone(&state);
            let target = target.clone();
            std::thread::spawn(move || state.insert_pending(&target) as usize)
        })
        .collect();
    assert_eq!(
        threads
            .into_iter()
            .map(|t| t.join().unwrap())
            .sum::<usize>(),
        1
    );
    state.finish(
        target.clone(),
        Status::Failed("audio_convert_failed".into()),
    );
    assert!(!state.insert_pending(&target));
    assert_eq!(
        state.entries.lock().unwrap().get(&target),
        Some(&Status::Failed("audio_convert_failed".into()))
    );
}
#[test]
fn missing_cached_file_is_prepared_again() {
    let state = State::default();
    let target = PathBuf::from("removed-stamp.flac");
    state.finish(
        target.clone(),
        Status::Ready("missing-audio-convert-test.flac".into()),
    );
    assert!(state.insert_pending(&target));
    assert_eq!(
        state.entries.lock().unwrap().get(&target),
        Some(&Status::Pending)
    );
}
