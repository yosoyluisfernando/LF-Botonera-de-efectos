/// Módulo: locution.rs
/// Propósito: qué archivo dice qué. Las reglas de nombres de las locuciones de
/// hora y clima, puras y sin disco: aquí se decide y `engine/weather/resolver.rs`
/// solo lee la carpeta y obedece.
///
/// El formato base es el de ZaraRadio. Lo comparte Salamandra en la hora, y
/// RadioBOSS dice lo mismo con otra convención numérica. Se aceptan las tres
/// variantes para que un pack traído de cualquiera de ellos suene sin renombrar
/// nada:
///  - Hora:        HRS14 + MIN25. En punto HRS14_O, y también HRS14_0: confundir
///                 la letra O con el cero es el error más repetido, y aceptarlo
///                 sale más barato que explicarlo.
///  - Temperatura: TMP025 (ZaraRadio) o TMP25 (RadioBOSS). Bajo cero, TMPN003 o
///                 TMP-3.
///  - Humedad:     HUM082 o HUM82.
///
/// Dinesat y Audicom NO entran, y no es un olvido: no localizan estos audios por
/// nombre de archivo —categoría de su base de datos el primero, módulo propio con
/// voces pregrabadas el segundo—, así que no hay convención con la que ser
/// compatible.

/// Lo que se quiere decir. El motor lo traduce a un archivo de la carpeta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locution {
    /// "Son las dos en punto": un archivo que se basta solo.
    HourSharp(u32),
    /// "Son las dos y…", que necesita su minuto detrás.
    Hour(u32),
    Minute(u32),
    /// Grados ya redondeados; el signo decide el prefijo.
    Temperature(i64),
    /// Porcentaje 0–100.
    Humidity(i64),
}

impl Locution {
    /// Los nombres aceptados, del canónico al tolerado. Se prueban en ese orden.
    pub fn aliases(&self) -> Vec<String> {
        match *self {
            Locution::HourSharp(h) => vec![format!("HRS{h:02}_O"), format!("HRS{h:02}_0")],
            Locution::Hour(h) => vec![format!("HRS{h:02}")],
            Locution::Minute(m) => vec![format!("MIN{m:02}")],
            Locution::Temperature(t) if t < 0 => {
                let mut names = padded("TMPN", t.abs());
                names.extend(padded("TMP-", t.abs()));
                names
            }
            Locution::Temperature(t) => padded("TMP", t),
            Locution::Humidity(h) => padded("HUM", h.clamp(0, 100)),
        }
    }

    /// Nombres que, aun empezando por un alias, NO son esta locución: el "y…" de
    /// la hora no puede quedarse con el archivo del "en punto".
    pub fn forbidden(&self) -> Vec<String> {
        match *self {
            Locution::Hour(h) => Locution::HourSharp(h).aliases(),
            _ => Vec::new(),
        }
    }
}

/// Los archivos de la hora, en el orden en que suenan.
///
/// En punto se prefiere el archivo propio, pero si el pack no lo trae se dice la
/// hora y el minuto en vez de callar: Salamandra no documenta el `_O` y muchos
/// packs no lo incluyen, así que esa hora no sonaba nada.
pub fn time_sequence(names: &[String], hh: u32, mm: u32) -> Vec<usize> {
    if mm == 0 {
        if let Some(i) = pick(names, &Locution::HourSharp(hh)) {
            return vec![i];
        }
    }
    [
        pick(names, &Locution::Hour(hh)),
        pick(names, &Locution::Minute(mm)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// El archivo de un valor de clima. Se redondea aquí porque el nombre es entero:
/// 24,6 °C dice "veinticinco grados".
pub fn climate(names: &[String], kind: &str, value: f64) -> Option<usize> {
    let rounded = value.round() as i64;
    let want = if kind == "humidity" {
        Locution::Humidity(rounded)
    } else {
        Locution::Temperature(rounded)
    };
    pick(names, &want)
}

/// Elige el archivo que dice `want` y devuelve su posición en `names`.
///
/// De cada alias se prueba primero el nombre exacto y luego el rotulado
/// ("HRS14 - las dos.mp3"), que es como viene mucho pack. El desempate es
/// alfabético y no el del sistema de archivos: `read_dir` no promete ningún
/// orden, así que sin ordenar la misma carpeta podía elegir un archivo distinto
/// en cada equipo.
pub fn pick(names: &[String], want: &Locution) -> Option<usize> {
    let mut stems: Vec<(usize, String)> = names.iter().map(|n| stem(n)).enumerate().collect();
    stems.sort_by(|a, b| a.1.cmp(&b.1));

    let forbidden = want.forbidden();
    let free: Vec<&(usize, String)> = stems
        .iter()
        .filter(|(_, s)| !forbidden.iter().any(|f| starts(s, f)))
        .collect();

    for alias in want.aliases() {
        if let Some((i, _)) = free.iter().find(|(_, s)| *s == alias) {
            return Some(*i);
        }
        if let Some((i, _)) = free.iter().find(|(_, s)| starts(s, &alias)) {
            return Some(*i);
        }
    }
    None
}

/// Un número con y sin ceros a la izquierda: ZaraRadio los pone, RadioBOSS no.
/// A partir de 100 los dos nombres coinciden y sobra uno.
fn padded(prefix: &str, n: i64) -> Vec<String> {
    let zeros = format!("{prefix}{n:03}");
    let plain = format!("{prefix}{n}");
    if zeros == plain {
        vec![plain]
    } else {
        vec![zeros, plain]
    }
}

/// El nombre sin extensión y en mayúsculas: los packs vienen rotulados de
/// cualquier forma y Windows no distingue mayúsculas.
fn stem(name: &str) -> String {
    let cut = name.rfind('.').unwrap_or(name.len());
    name[..cut].to_uppercase()
}

/// `stem` empieza por `alias` y el número se acaba ahí: si lo que sigue es otra
/// cifra, es otra locución. Sin esta condición, a 0 grados el alias "TMP0" se
/// llevaría "TMP025.mp3" y la radio diría veinticinco.
fn starts(stem: &str, alias: &str) -> bool {
    stem.strip_prefix(alias)
        .is_some_and(|rest| !rest.starts_with(|c: char| c.is_ascii_digit()))
}

#[cfg(test)]
#[path = "locution_tests.rs"]
mod locution_tests;
