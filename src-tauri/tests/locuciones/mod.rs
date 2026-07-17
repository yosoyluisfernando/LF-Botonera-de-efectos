//! El andamio de las pruebas de locuciones contra carpetas de verdad: los packs
//! tal como los reparte cada programa, y una carpeta temporal que se borra sola.
//!
//! Va aparte de `common/`, que es de las pruebas contra tarjetas reales: aquí no
//! se toca el audio, solo el disco.
use chrono::{Local, Timelike};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri_app_lib::engine::weather::resolver::{resolve_climate_file, resolve_time_files};

// ─── Los packs, tal como los reparte cada programa ────────────────────────────

/// ZaraRadio: la hora con sus dos dígitos, el "en punto" y los minutos.
pub fn pack_zara_hora() -> Vec<String> {
    let mut pack = Vec::new();
    for h in 0..24 {
        pack.push(format!("HRS{h:02}.mp3"));
        pack.push(format!("HRS{h:02}_O.mp3"));
    }
    for m in 0..60 {
        pack.push(format!("MIN{m:02}.mp3"));
    }
    pack
}

/// Salamandra: la hora de ZaraRadio, pero SIN "en punto" —su documentación no lo
/// menciona— y con un jingle que aquí no pinta nada.
pub fn pack_salamandra_hora() -> Vec<String> {
    let mut pack: Vec<String> = (0..24).map(|h| format!("HRS{h:02}.mp3")).collect();
    pack.extend((0..60).map(|m| format!("MIN{m:02}.mp3")));
    pack.push("TIME_JINGLE.mp3".to_string());
    pack
}

/// Como viene rotulado mucho pack: el código y detrás el texto de la locución.
pub fn pack_rotulado() -> Vec<String> {
    let mut pack: Vec<String> = (0..24)
        .map(|h| format!("HRS{h:02} - son las {h}.mp3"))
        .collect();
    pack.extend((0..60).map(|m| format!("MIN{m:02} - y {m}.mp3")));
    pack
}

/// ZaraRadio: tres dígitos, y la N delante para el bajo cero.
pub fn pack_zara_clima() -> Vec<String> {
    let mut pack: Vec<String> = (0..=50).map(|t| format!("TMP{t:03}.mp3")).collect();
    pack.extend((1..=10).map(|t| format!("TMPN{t:03}.mp3")));
    pack.extend((0..=100).map(|h| format!("HUM{h:03}.mp3")));
    pack
}

/// RadioBOSS: sin ceros a la izquierda y con el signo. Su manual da literalmente
/// TMP29.mp3, TMP-10.mp3 y HUM3.mp3.
pub fn pack_radioboss_clima() -> Vec<String> {
    let mut pack: Vec<String> = (0..=50).map(|t| format!("TMP{t}.mp3")).collect();
    pack.extend((1..=10).map(|t| format!("TMP-{t}.mp3")));
    pack.extend((0..=100).map(|h| format!("HUM{h}.mp3")));
    pack
}

// ─── La carpeta ───────────────────────────────────────────────────────────────

/// Una carpeta temporal con los archivos dentro, que se borra sola al acabar.
pub struct Carpeta {
    ruta: PathBuf,
}

impl Carpeta {
    /// Los archivos van VACÍOS a propósito: el resolver elige rutas, no
    /// decodifica. Un MP3 de verdad no probaría nada más y ataría la prueba a un
    /// códec.
    pub fn con(archivos: &[String]) -> Carpeta {
        static N: AtomicUsize = AtomicUsize::new(0);
        let unica = format!(
            "lf_botonera_loc_{}_{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        );
        let ruta = std::env::temp_dir().join(unica);
        let _ = fs::remove_dir_all(&ruta);
        fs::create_dir_all(&ruta).expect("crear la carpeta temporal");
        for archivo in archivos {
            fs::write(ruta.join(archivo), b"").expect("crear el archivo");
        }
        Carpeta { ruta }
    }

    pub fn path(&self) -> &str {
        self.ruta.to_str().expect("ruta temporal válida")
    }

    /// El nombre del archivo que suena para ese valor de clima.
    pub fn clima(&self, kind: &str, value: f64) -> String {
        let ruta = resolve_climate_file(self.path(), kind, value)
            .unwrap_or_else(|e| panic!("{kind} {value} debería sonar, y dio {e}"));
        assert!(Path::new(&ruta).exists(), "la ruta devuelta no existe: {ruta}");
        nombre(&ruta)
    }
}

impl Drop for Carpeta {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.ruta);
    }
}

// ─── El reloj ─────────────────────────────────────────────────────────────────

/// Resuelve la hora asegurando que el reloj no cambió de minuto por el camino:
/// entre mirarlo y resolver cabe un cambio de minuto, y eso sería una prueba que
/// falla una vez cada mil sin que nada esté roto.
pub fn resuelve_con_el_reloj_quieto(carpeta: &Carpeta) -> ((u32, u32), Vec<String>) {
    for _ in 0..3 {
        let antes = ahora();
        let suena = resolve_time_files(carpeta.path()).expect("el pack cubre todas las horas");
        for ruta in &suena {
            assert!(Path::new(ruta).exists(), "la ruta devuelta no existe: {ruta}");
        }
        if ahora() == antes {
            return (antes, suena.iter().map(|r| nombre(r)).collect());
        }
    }
    panic!("el reloj no se estuvo quieto en tres intentos");
}

fn ahora() -> (u32, u32) {
    let now = Local::now();
    (now.hour(), now.minute())
}

fn nombre(ruta: &str) -> String {
    Path::new(ruta)
        .file_name()
        .expect("la ruta trae nombre")
        .to_string_lossy()
        .to_string()
}
