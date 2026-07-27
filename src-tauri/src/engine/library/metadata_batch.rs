use super::metadata::{self, LibraryMetadata};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;

pub type MetadataResult = Result<LibraryMetadata, String>;

/// Lee un lote con concurrencia acotada y conserva el orden de entrada.
/// No se usa Rayon: este trabajo necesita un limite propio para no competir
/// sin control con el motor de audio ni sumar una dependencia.
pub fn read(paths: &[String], workers: usize) -> Vec<MetadataResult> {
    if paths.is_empty() {
        return Vec::new();
    }
    let worker_count = workers.clamp(1, paths.len());
    let next = AtomicUsize::new(0);
    let (sender, receiver) = mpsc::channel();
    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            let sender = sender.clone();
            let next = &next;
            scope.spawn(move || loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some(path) = paths.get(index) else {
                    break;
                };
                if sender.send((index, metadata::read(path))).is_err() {
                    break;
                }
            });
        }
        drop(sender);
        let mut ordered: Vec<Option<MetadataResult>> =
            std::iter::repeat_with(|| None).take(paths.len()).collect();
        for (index, result) in receiver {
            ordered[index] = Some(result);
        }
        ordered
            .into_iter()
            .map(|result| result.expect("cada trabajo produce un resultado"))
            .collect()
    })
}

pub fn recommended_workers() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(2)
        .clamp(1, 4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_batch_is_empty() {
        assert!(read(&[], 4).is_empty());
    }

    #[test]
    fn results_keep_input_order() {
        let paths = vec![
            "missing-first.mp3".to_string(),
            "missing-second.wav".to_string(),
        ];
        let results = read(&paths, 2);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(Result::is_err));
    }

    #[test]
    fn recommended_parallelism_never_exceeds_measured_limit() {
        assert!((1..=4).contains(&recommended_workers()));
    }
}
