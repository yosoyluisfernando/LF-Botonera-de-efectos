/// Modulo: random_folder.rs
/// Proposito: resolver carpetas aleatorias con bolsa mezclada por boton.
use crate::audio_formats;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Indica si la ruta apunta a un archivo de audio compatible por extension.
pub fn is_audio_file(path: &Path) -> bool {
    audio_formats::is_audio_path(path)
}

/// Valida que una carpeta tenga al menos un audio compatible.
pub fn ensure_has_audio(folder: &str) -> Result<(), String> {
    if audio_files(folder)?.is_empty() {
        Err("random_folder_empty".to_string())
    } else {
        Ok(())
    }
}

#[derive(Default)]
pub struct RandomFolderState {
    bags: HashMap<String, FolderBag>,
}

struct FolderBag {
    folder: String,
    queue: VecDeque<PathBuf>,
    recent: VecDeque<PathBuf>,
    current: Option<PathBuf>,
}

impl RandomFolderState {
    /// Devuelve el audio activo si el boton sigue sonando; si no, avanza la bolsa.
    pub fn active_or_next_audio(
        &mut self,
        button_id: &str,
        folder: &str,
        is_active: bool,
    ) -> Result<String, String> {
        self.resolve_audio(button_id, folder, is_active)
    }

    fn resolve_audio(
        &mut self,
        button_id: &str,
        folder: &str,
        keep_active: bool,
    ) -> Result<String, String> {
        if keep_active {
            if let Some(path) = self.active_audio(button_id, folder) {
                return Ok(path.to_string_lossy().to_string());
            }
        }

        let needs_new = self
            .bags
            .get(button_id)
            .map(|b| b.folder != folder || b.queue.is_empty())
            .unwrap_or(true);

        if needs_new {
            let recent = self
                .bags
                .remove(button_id)
                .filter(|b| b.folder == folder)
                .map(|b| b.recent)
                .unwrap_or_default();
            let bag = FolderBag::new(folder, recent)?;
            self.bags.insert(button_id.to_string(), bag);
        }

        let bag = self.bags.get_mut(button_id).ok_or("random_folder_empty")?;
        let path = bag.queue.pop_front().ok_or("random_folder_empty")?;
        bag.remember(path.clone());
        bag.current = Some(path.clone());
        Ok(path.to_string_lossy().to_string())
    }

    fn active_audio(&self, button_id: &str, folder: &str) -> Option<PathBuf> {
        self.bags
            .get(button_id)
            .filter(|bag| bag.folder == folder)
            .and_then(|bag| bag.current.clone())
    }
}

impl FolderBag {
    fn new(folder: &str, recent: VecDeque<PathBuf>) -> Result<Self, String> {
        let mut files = audio_files(folder)?;
        if files.is_empty() {
            return Err("random_folder_empty".to_string());
        }
        shuffle(&mut files);
        avoid_recent_start(&mut files, &recent);
        Ok(Self {
            folder: folder.to_string(),
            queue: files.into(),
            recent,
            current: None,
        })
    }

    fn remember(&mut self, path: PathBuf) {
        self.recent.push_front(path);
        while self.recent.len() > 3 {
            self.recent.pop_back();
        }
    }
}

fn audio_files(folder: &str) -> Result<Vec<PathBuf>, String> {
    let dir = Path::new(folder);
    if !dir.is_dir() {
        return Err("random_folder_not_found".to_string());
    }

    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file() && is_audio_file(path))
        .collect();
    files.sort();
    Ok(files)
}

fn avoid_recent_start(files: &mut Vec<PathBuf>, recent: &VecDeque<PathBuf>) {
    if files.len() < 2 || recent.is_empty() {
        return;
    }
    let limit = recent.len().min(files.len() - 1);
    for _ in 0..files.len() {
        if !recent.iter().take(limit).any(|p| p == &files[0]) {
            return;
        }
        files.rotate_left(1);
    }
}

fn shuffle(files: &mut [PathBuf]) {
    let mut seed = seed_for(files);
    for i in (1..files.len()).rev() {
        seed = xorshift(seed);
        files.swap(i, seed as usize % (i + 1));
    }
}

fn seed_for(files: &[PathBuf]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
        .hash(&mut hasher);
    for path in files {
        path.hash(&mut hasher);
    }
    hasher.finish()
}

fn xorshift(mut x: u64) -> u64 {
    if x == 0 {
        x = 0x9E3779B97F4A7C15;
    }
    x ^= x << 13;
    x ^= x >> 7;
    x ^ (x << 17)
}
