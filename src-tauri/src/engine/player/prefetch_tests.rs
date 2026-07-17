//! Pruebas del colchon. Van sin tarjeta: lo que se prueba es que las muestras
//! salen intactas, que la pista termina cuando se vacia el colchon y —lo que mas
//! importa— que el hilo se muere solo cuando el deck suelta su fuente.
use super::*;
use rodio::buffer::SamplesBuffer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;

fn fuente(muestras: Vec<f32>) -> BoxSource {
    Box::new(SamplesBuffer::new(2, 48_000, muestras))
}

/// Lo esencial: por el colchon no se pierde ni se reordena una muestra. Se pasan
/// mas de un lote (4096) para que el hilo tenga que mandar varios.
#[test]
fn las_muestras_salen_intactas_y_en_orden() {
    let original: Vec<f32> = (0..10_000).map(|i| i as f32).collect();
    let salida: Vec<f32> = buffered(fuente(original.clone())).collect();
    assert_eq!(salida, original);
}

/// El fin de pista llega al vaciarse el colchon, no al acabarse el disco. Si
/// llegara antes, el reproductor cortaria los ultimos diez segundos de cada
/// cancion y saltaria a la siguiente antes de tiempo.
#[test]
fn la_pista_termina_cuando_el_colchon_se_vacia() {
    let mut fuente = buffered(fuente(vec![1.0, 2.0, 3.0]));
    assert_eq!(fuente.next(), Some(1.0));
    assert_eq!(fuente.next(), Some(2.0));
    assert_eq!(fuente.next(), Some(3.0));
    assert_eq!(fuente.next(), None, "se acabo de verdad");
    assert_eq!(fuente.next(), None, "y sigue acabada");
}

#[test]
fn una_fuente_vacia_termina_sin_colgarse() {
    assert_eq!(buffered(fuente(vec![])).next(), None);
}

/// El ritmo se pregunta antes de mandar la fuente al hilo, y el bus lo necesita
/// desde el primer momento para saber a que velocidad va.
#[test]
fn el_colchon_conserva_el_ritmo_de_la_fuente() {
    let fuente = buffered(fuente(vec![0.0; 10]));
    assert_eq!(fuente.channels(), 2);
    assert_eq!(fuente.sample_rate(), 48_000);
}

/// La prueba que de verdad importa: el deck suelta su fuente en cada cancion, en
/// cada salto de posicion y en cada cambio de tarjeta. Si el hilo no se muriera
/// con ella, cada una dejaria un hilo bloqueado y varios megas colgados para
/// siempre.
///
/// La fuente es INFINITA a proposito: asi el hilo llena la cola y se queda
/// bloqueado en `send`, que es el caso que podria colgarse. Al soltar el
/// consumidor, `send` falla, el hilo vuelve y suelta la fuente — y eso enciende
/// el testigo.
#[test]
fn soltar_la_fuente_mata_el_hilo_aunque_este_bloqueado() {
    static VIVA: AtomicBool = AtomicBool::new(true);
    VIVA.store(true, Ordering::SeqCst);

    struct Infinita;
    impl Iterator for Infinita {
        type Item = f32;
        fn next(&mut self) -> Option<f32> {
            Some(0.0)
        }
    }
    impl Source for Infinita {
        fn current_frame_len(&self) -> Option<usize> {
            None
        }
        fn channels(&self) -> u16 {
            2
        }
        fn sample_rate(&self) -> u32 {
            48_000
        }
        fn total_duration(&self) -> Option<Duration> {
            None
        }
    }
    impl Drop for Infinita {
        fn drop(&mut self) {
            VIVA.store(false, Ordering::SeqCst);
        }
    }

    let colchon = buffered(Box::new(Infinita));
    // Sin sacar una sola muestra, el hilo llena la cola acotada y se queda
    // bloqueado en `send`. Es un estado que no se puede observar desde fuera, asi
    // que se le da un momento para llegar a el — es justo el caso que podria
    // colgarse, y el que hay que provocar para que la prueba valga.
    sleep(Duration::from_millis(100));
    assert!(VIVA.load(Ordering::SeqCst), "el hilo aun deberia tenerla");

    drop(colchon);

    assert!(
        esperar(|| !VIVA.load(Ordering::SeqCst)),
        "el hilo quedo colgado con la fuente: cada cancion filtraria un hilo"
    );
}

/// El hilo va a su ritmo y no al nuestro: se le da un margen en vez de un sleep
/// fijo, que en un equipo cargado seria una prueba que falla sin que nada este
/// roto.
fn esperar(condicion: impl Fn() -> bool) -> bool {
    for _ in 0..100 {
        if condicion() {
            return true;
        }
        sleep(Duration::from_millis(20));
    }
    false
}
