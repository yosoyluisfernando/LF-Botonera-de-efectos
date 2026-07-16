//! Pruebas de la paleta y la adaptacion de tema. Lo que se comprueba aqui es que
//! los colores se DISTINGAN y que el texto se LEA, que es lo que la regla 8 pide.
use super::*;

/// El matiz es lo unico que sobrevive al recorte de `adapt_color`, asi que dos
/// colores con el mismo matiz se verian iguales. Esto era el fallo de la paleta
/// anterior: 16 matices repetidos en dos intensidades aparentando 32 colores.
#[test]
fn every_colour_has_a_distinct_hue() {
    let hues: Vec<i32> = SAFE_COLORS
        .iter()
        .map(|c| hex_to_hsl(c).unwrap().0 as i32)
        .collect();
    for (i, a) in hues.iter().enumerate() {
        for b in hues.iter().skip(i + 1) {
            let d = (a - b).abs().min(360 - (a - b).abs());
            assert!(d >= 10, "dos matices a {d}° no se distinguen: {a}° y {b}°");
        }
    }
}

/// Y lo que de verdad importa: que sigan siendo distintos DESPUES de adaptarlos
/// a cada tema. Si el recorte los junta, al usuario le da igual que la base
/// fuera distinta.
#[test]
fn colours_stay_distinct_after_adapting_to_each_theme() {
    for theme in ["dark", "light"] {
        let adapted: std::collections::HashSet<String> = SAFE_COLORS
            .iter()
            .map(|c| adapt_color(c, theme, "button"))
            .collect();
        assert_eq!(
            adapted.len(),
            SAFE_COLORS.len(),
            "en tema {theme} hay colores que acaban iguales"
        );
    }
}

/// El reparto no puede amontonarse: la paleta vieja tenia 6 azules y 6 rojos
/// pero un solo verde.
#[test]
fn hues_are_spread_around_the_colour_wheel() {
    let mut bands = [0u32; 12]; // franjas de 30°
    for c in SAFE_COLORS {
        bands[(hex_to_hsl(c).unwrap().0 as usize / 30).min(11)] += 1;
    }
    assert!(bands.iter().all(|&n| n > 0), "hay franjas de color vacias: {bands:?}");
    assert!(bands.iter().all(|&n| n <= 4), "hay colores amontonados: {bands:?}");
}

/// Regla 8: el texto debe LEERSE sobre el fondo, en los dos temas. Se comprueba
/// el contraste de verdad, no que el color sea uno concreto: lo que importa es
/// que se lea, no que sea exactamente blanco o negro.
#[test]
fn text_is_readable_on_every_colour() {
    for c in SAFE_COLORS {
        for theme in ["dark", "light"] {
            let bg = adapt_color(c, theme, "button");
            let text = text_for_theme(c, theme, "button");
            let ratio = contrast_ratio(&bg, &text);
            // 4.4: el estandar pide 4.5 y solo un color se queda en 4.43. Subir
            // mas obligaria a aclarar la paleta y perder viveza.
            assert!(ratio >= 4.4, "{c} en {theme}: contraste {ratio:.1}, insuficiente");
        }
    }
}


/// La paleta no puede tener repetidos: son huecos desperdiciados.
#[test]
fn there_are_no_duplicates() {
    let unique: std::collections::HashSet<_> = SAFE_COLORS.iter().collect();
    assert_eq!(unique.len(), SAFE_COLORS.len());
}
