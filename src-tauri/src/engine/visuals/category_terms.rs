/// Términos conceptuales de cada grupo Unicode. Complementan CLDR para que una
/// búsqueda como "animal" encuentre la categoría, no solo nombres individuales.
pub(super) fn for_group(language: &str, group: &str) -> &'static str {
    match (language, group) {
        ("es", "smileys-and-emotion") => "cara caras emoción emociones gesto gestos",
        ("es", "people-and-body") => "persona personas gente cuerpo humano mano manos",
        ("es", "component") => "componente componentes pelo cabello tono piel",
        ("es", "animals-and-nature") => "animal animales naturaleza planta plantas flor flores",
        ("es", "food-and-drink") => "comida comidas bebida bebidas alimento alimentos",
        ("es", "travel-and-places") => "viaje viajes lugar lugares transporte vehículo",
        ("es", "activities") => "actividad actividades deporte deportes juego juegos ocio",
        ("es", "objects") => "objeto objetos cosa cosas herramienta herramientas",
        ("es", "symbols") => "símbolo símbolos señal señales signo signos",
        ("es", "flags") => "bandera banderas país países región regiones",
        ("en", "smileys-and-emotion") => "face faces emotion emotions gesture gestures",
        ("en", "people-and-body") => "person people body human hand hands",
        ("en", "component") => "component components hair skin tone",
        ("en", "animals-and-nature") => "animal animals nature plant plants flower flowers",
        ("en", "food-and-drink") => "food foods drink drinks beverage beverages",
        ("en", "travel-and-places") => "travel travels place places transport vehicle",
        ("en", "activities") => "activity activities sport sports game games leisure",
        ("en", "objects") => "object objects thing things tool tools",
        ("en", "symbols") => "symbol symbols sign signs signal signals",
        ("en", "flags") => "flag flags country countries region regions",
        ("pt-BR", "smileys-and-emotion") | ("pt-PT", "smileys-and-emotion") => {
            "cara caras emoção emoções gesto gestos"
        }
        ("pt-BR", "people-and-body") | ("pt-PT", "people-and-body") => {
            "pessoa pessoas gente corpo humano mão mãos"
        }
        ("pt-BR", "component") | ("pt-PT", "component") => "componente componentes cabelo tom pele",
        ("pt-BR", "animals-and-nature") | ("pt-PT", "animals-and-nature") => {
            "animal animais natureza planta plantas flor flores"
        }
        ("pt-BR", "food-and-drink") | ("pt-PT", "food-and-drink") => {
            "comida comidas bebida bebidas alimento alimentos"
        }
        ("pt-BR", "travel-and-places") | ("pt-PT", "travel-and-places") => {
            "viagem viagens lugar lugares transporte veículo veículos"
        }
        ("pt-BR", "activities") | ("pt-PT", "activities") => {
            "atividade atividades desporto esporte jogo jogos lazer"
        }
        ("pt-BR", "objects") | ("pt-PT", "objects") => {
            "objeto objetos coisa coisas ferramenta ferramentas"
        }
        ("pt-BR", "symbols") | ("pt-PT", "symbols") => "símbolo símbolos sinal sinais signo signos",
        ("pt-BR", "flags") | ("pt-PT", "flags") => "bandeira bandeiras país países região regiões",
        _ => "",
    }
}
