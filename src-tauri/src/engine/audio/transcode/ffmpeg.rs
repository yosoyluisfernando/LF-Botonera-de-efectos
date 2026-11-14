//! Proceso externo; adapta la conversión aportada por Danilo Wenner.
//! No se ejecuta desde el hilo de audio ni desde el hilo de ventana.
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
const EXE: &str = if cfg!(windows) {
    "ffmpeg.exe"
} else {
    "ffmpeg"
};

fn binary() -> Option<&'static Path> {
    static FOUND: OnceLock<Option<PathBuf>> = OnceLock::new();
    FOUND
        .get_or_init(|| {
            let mut candidates = Vec::new();
            if let Some(path) = std::env::var_os("LF_FFMPEG_PATH") {
                candidates.push(PathBuf::from(path));
            }
            if let Some(dir) = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(Path::to_path_buf))
            {
                candidates.push(dir.join(EXE));
            }
            #[cfg(debug_assertions)]
            candidates.push(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("binaries")
                    .join(EXE),
            );
            if let Some(paths) = std::env::var_os("PATH") {
                candidates.extend(std::env::split_paths(&paths).map(|dir| dir.join(EXE)));
            }
            candidates
                .into_iter()
                .find(|path| path.is_absolute() && path.is_file())
        })
        .as_deref()
}

pub fn convert(input: &str, output: &Path) -> Result<(), String> {
    let binary = binary().ok_or("audio_converter_missing")?;
    let dir = output.parent().ok_or("audio_convert_failed")?;
    std::fs::create_dir_all(dir).map_err(|_| "audio_convert_failed")?;
    let partial = output.with_extension(format!("{}.part", std::process::id()));
    let mut command = Command::new(binary);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let result = (|| {
        let mut child = command
            .args([
                "-nostdin",
                "-nostats",
                "-progress",
                "pipe:1",
                "-stats_period",
                "0.2",
                "-hide_banner",
                "-loglevel",
                "error",
                "-y",
                "-protocol_whitelist",
                "file",
                "-i",
            ])
            .arg(input)
            .args([
                "-map",
                "0:a:0",
                "-vn",
                "-map_metadata",
                "-1",
                "-c:a",
                "flac",
                "-sample_fmt",
                "s32",
                "-f",
                "flac",
            ])
            .arg(&partial)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "audio_converter_missing")?;
        let path = input.to_string();
        let duration = crate::engine::audio::formats::probe_duration_secs(input);
        let stdout = child.stdout.take().ok_or("audio_convert_failed")?;
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                super::progress::advance(&path, &line, duration);
            }
        });
        let start = Instant::now();
        let success = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status.success(),
                Ok(None) if start.elapsed() < Duration::from_secs(300) => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break false;
                }
            }
        };
        let _ = reader.join();
        if !success {
            return Err("audio_convert_failed".to_string());
        }
        // No publicar una copia vacía o que el decodificador compartido no abra.
        if !crate::engine::audio::decode::can_decode_native(&partial.to_string_lossy()) {
            return Err("audio_convert_failed".to_string());
        }
        std::fs::rename(&partial, output).map_err(|_| "audio_convert_failed".to_string())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&partial);
    }
    result
}
