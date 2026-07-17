//! El colchon contra archivos de VERDAD. Van marcadas `#[ignore]`: necesitan un
//! audio real en ESTA maquina, y una prueba que falla por el entorno no dice
//! nada. Las rutas se dan por variable de entorno para no clavar las de un equipo
//! en el repo:
//!
//! - `LF_TEST_LONG_SONG`  → una cancion larga (varios minutos): el caso disco, el
//!   que el colchon protege de verdad.
//! - `LF_TEST_SHORT_FX`   → un efecto corto (< 10 s): el caso que TAMBIEN se
//!   precarga en RAM, para ver que pasa cuando se juntan colchon y precarga.
//!
//! ```text
//! cargo test --lib prefetch_real -- --ignored --nocapture
//! ```
use super::*;
use crate::engine::cache::preload::{build_play_source, decode_pcm, PreloadCache};
use crate::engine::persist::db;
use std::sync::Mutex;
use std::time::Instant;

/// La ruta de la variable, o None para saltar sin fallar.
fn ruta(var: &str) -> Option<String> {
    std::env::var(var).ok().filter(|s| !s.trim().is_empty())
}

fn decodifica(path: &str) -> BoxSource {
    let cache = Arc::new(Mutex::new(PreloadCache::new(0)));
    build_play_source(&cache, path, false, 0.0, None).expect("el archivo deberia decodificar")
}

/// Compara dos fuentes muestra a muestra SIN juntar las dos en memoria: una
/// cancion larga son decenas de millones de muestras. Devuelve cuantas hubo.
fn iguales(mut a: BoxSource, mut b: BoxSource) -> u64 {
    let mut n = 0u64;
    loop {
        match (a.next(), b.next()) {
            (Some(x), Some(y)) => {
                assert!((x - y).abs() < 1e-6, "muestra {n} distinta: {x} vs {y}");
                n += 1;
            }
            (None, None) => return n,
            _ => panic!("las dos fuentes tienen distinta longitud (muestra {n})"),
        }
    }
}

/// Lo esencial en un MP3 de verdad: por el colchon no se altera ni una muestra de
/// la cancion. Se decodifica el mismo archivo dos veces —el decodificador es
/// determinista— y se comparan, una envuelta y la otra no.
#[test]
#[ignore]
fn el_colchon_no_altera_una_cancion_de_verdad() {
    let Some(path) = ruta("LF_TEST_LONG_SONG") else {
        eprintln!("saltada: define LF_TEST_LONG_SONG");
        return;
    };
    let n = iguales(decodifica(&path), buffered(decodifica(&path)));
    println!("  ✓ {n} muestras intactas a traves del colchon");
    assert!(n > 0, "la cancion no dio ni una muestra");
}

/// La razon de que el colchon funcione: decodificar va MUCHISIMO mas rapido que
/// el tiempo real, asi que el hilo siempre va por delante y un atasco de disco
/// mas corto que el colchon ni se nota. Se mide y se reporta la holgura.
#[test]
#[ignore]
fn el_colchon_va_muy_por_delante_del_tiempo_real() {
    let Some(path) = ruta("LF_TEST_LONG_SONG") else {
        eprintln!("saltada: define LF_TEST_LONG_SONG");
        return;
    };
    let fuente = buffered(decodifica(&path));
    let rate = fuente.sample_rate() as f64 * fuente.channels().max(1) as f64;
    let inicio = Instant::now();
    let muestras = fuente.count() as f64;
    let real = inicio.elapsed().as_secs_f64();
    let audio = muestras / rate;
    println!(
        "  cancion de {audio:.0}s decodificada por el colchon en {real:.2}s → {:.0}× mas rapida que el tiempo real",
        audio / real
    );
    assert!(
        audio / real > 5.0,
        "el colchon apenas adelanta al tiempo real ({:.1}×): un atasco lo vaciaria",
        audio / real
    );
}

/// El caso que preguntaba el autor: un efecto corto SÍ se precarga en RAM. Aqui
/// se mete en una caché de verdad, se saca por acierto (cache HIT) y se pasa por
/// el colchon. Comparado con el mismo acierto SIN colchon, tiene que dar lo
/// mismo: envolver algo que ya esta en RAM no lo estropea, solo no le hace falta.
#[test]
#[ignore]
fn un_efecto_precargado_pasa_por_el_colchon_intacto() {
    let Some(path) = ruta("LF_TEST_SHORT_FX") else {
        eprintln!("saltada: define LF_TEST_SHORT_FX");
        return;
    };
    let pcm = decode_pcm(&path).expect("el efecto deberia decodificar a PCM");
    let cache = Arc::new(Mutex::new(PreloadCache::new(64)));
    cache
        .lock()
        .unwrap()
        .insert(db::normalize_key(&path), pcm);
    assert!(
        cache.lock().unwrap().contains(&db::normalize_key(&path)),
        "el efecto deberia estar precargado"
    );

    let sin = build_play_source(&cache, &path, false, 0.0, None).expect("acierto de caché");
    let con = buffered(build_play_source(&cache, &path, false, 0.0, None).expect("acierto de caché"));
    let n = iguales(sin, con);
    println!("  ✓ efecto precargado en RAM: {n} muestras iguales con y sin colchon");
}
