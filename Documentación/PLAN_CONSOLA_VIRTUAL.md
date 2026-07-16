# Plan propuesto — Consola de audio virtual

Documento de lectura y discusión. **No se ha tocado código.** Sesión del 2026-07-16.

Guía externa consultada: `C:\LF Automatizador v1.0\entendiendo la consola virtual.md`.
Reglas aplicadas: `Documentación/REGLAS_PROYECTO.md` (en especial la 4, la 5 y la 9).

---

## 1. Cómo viaja el audio hoy, de verdad

Antes de proponer nada, esto es lo que hay en el disco, leído módulo a módulo. Es importante
porque tres de los cuatro problemas de abajo no se ven desde la interfaz.

### Hay tres motores, no uno

La aplicación abre hasta **tres `OutputStream` independientes**:

1. **Efectos** — `engine/audio/thread.rs`, campo `device`. Es un `MasterBus`
   (`engine/audio/bus.rs`): un `DynamicMixer` de rodio a 48 kHz estéreo, con un `LevelSource`
   detrás que mide el pico, y un `Sink` al final. Por aquí suenan **los botones de la botonera
   principal y los del panel fijo**, mezclados en el mismo sitio.
2. **Pre-escucha** — mismo hilo, campo `device_pre`. Es **otro `MasterBus` completo**, con su
   propio mixer, su propio medidor (`pre_l` / `pre_r`) y su propio volumen (`pre_volume`).
3. **Reproductor auxiliar** — `engine/player/thread.rs`. Motor aparte, hilo aparte,
   `OutputStream` propio y **dos decks** que son `Sink` de rodio conectados directamente al
   dispositivo. No hay mixer, no hay medidor, no hay fader de bus.

### El recorrido de un botón

Pulsar un botón entra por `play_button` → `cmd_button_playback` lee la configuración y consulta
`tracks.db` para el cue y la ganancia → `AudioEngine::play_file` manda un `AudioCommand::Play`
por un canal al hilo de audio → el hilo elige bus con **un booleano**, `to_pre`
(`thread.rs:111-115`) → `MasterBus::add_source` envuelve la fuente en un `ButtonSource` y la
mete en el mixer.

El `ButtonSource` (`engine/audio/button.rs:110`) es donde se hace toda la ganancia:

```
muestra × file_gain × volumen_del_botón × master_volume × fade
```

### Los cuatro hallazgos

**Primero: el máster no existe como punto de la señal.** No hay ningún fader de máster. Hay un
atómico `master_volume` que **cada fuente lee por su cuenta y se multiplica a sí misma**. Es una
convención compartida entre fuentes, no una etapa. La consecuencia práctica es que no existe
ningún lugar en el código donde puedas decir "aquí está el programa"; el programa solo existe
como resultado accidental de que varias fuentes decidieron obedecer el mismo número. Esto es
exactamente lo que el documento del LFA marca como objetivo pendiente: *"Fader único sobre el bus
de programa, no por player"*.

**Segundo: la pre-escucha se cuela en el máster, y de forma inconsistente.** Esto es un hallazgo
concreto, verificable en `thread.rs:111-115`:

```rust
let bus = if to_pre {
    device_pre.bus().or_else(|| device.bus())   // ← el fallback
} else {
    device.bus()
};
```

`out_pre` viene vacío por defecto, y `cmd_audio.rs:126-130` convierte "vacío o igual a la
principal" en "no hay dispositivo pre". Entonces `device_pre` no tiene bus, y **la pre-escucha
cae al bus principal**. Al entrar ahí se le aplica `master_volume` (porque `add_source` usa el
`self.master_volume` del bus donde entra) y **suma al vúmetro de programa**.

O sea: con tarjeta de pre-escucha dedicada, el máster no toca la pre-escucha. Sin tarjeta
dedicada — que es el caso por defecto de la mayoría de usuarios — el máster sí la ataca y además
la escucha el vúmetro. **El mismo botón se comporta distinto según cómo esté configurado el
equipo.** Es justo lo que dijiste que no puede pasar, y hoy pasa.

**Tercero: dos motores pueden abrir la misma tarjeta a la vez.** `player_set_device("")` está
documentado como "el mismo dispositivo de los efectos", y `cmd_player_config.rs:67-70` lo
resuelve al `out_main` del perfil. Pero el reproductor **no se une al bus de los efectos**: abre
su propio `OutputStream` sobre esa misma tarjeta. Son dos flujos que el sistema operativo suma
por su cuenta, fuera de nuestro control. Es literalmente el defecto que el LFA describe: *"el
Encoder está intentando reconstruir la mezcla en lugar de capturar la suma de los buses"*.

**Cuarto: ya hay dos promesas de ruteo que no rutean.** `PaletaData.audio_out` existe, se guarda
y se exporta, y **no enruta nada** (el `GLOSARIO.md` lo llama "trampa" con todas las letras).
Y `PlaybackGroup { Main, Fixed }` agrupa para el Solo y el Stop, pero **no enruta**. El vocabulario
de la consola ya está en el modelo de datos; lo que falta es el motor debajo.

---

## 2. Qué es una consola de verdad (y qué implica aquí)

Una consola física separa tres cosas que aquí están fundidas en una sola:

- **Canal** — una fuente con su ganancia de entrada y su fader.
- **Bus** — un punto de suma con nombre, con su propio fader y su propio medidor. Un bus es una
  *señal*, no una tarjeta.
- **Salida física** — el conector. Varios buses pueden salir por el mismo conector sin ser el
  mismo bus.

**Esa última frase es la clave de todo lo que pediste.** Que la pre-escucha y el programa salgan
por la misma tarjeta cuando solo hay una tarjeta **no los convierte en el mismo bus**. Siguen
siendo dos señales independientes: el fader de máster no toca el CUE, el vúmetro de programa no
lo mide, el Stop general no lo detiene. Simplemente van a parar al mismo conector porque no hay
otro. Hoy el código no puede expresar esa diferencia, porque bus y salida son el mismo objeto
(`MasterBus` contiene su propio `Sink`).

El otro principio del LFA que conviene respetar es el **modelo de oyente**: quien quiere escuchar
un bus se *engancha* a él, no reconstruye la mezcla por su cuenta. Es lo que hoy incumple el
reproductor.

---

## 3. La arquitectura propuesta

### Topología

Cuatro buses. Tres suman en programa; uno nunca.

- **Bus EFECTOS** — botones de la botonera principal (`PlaybackGroup::Main`).
- **Bus PANEL** — botones del panel fijo (`PlaybackGroup::Fixed`).
- **Bus REPRODUCTOR** — el reproductor auxiliar.
- **Bus CUE** — pre-escucha (`__prelisten__`) y previa del editor (`__track_preview__`).

Los tres primeros suman en un **bus PROGRAMA (PGM)**, que pasa por **un único fader de máster** y
por **un único medidor de programa**, y de ahí a su salida física.

El **CUE nunca entra en PGM**. Ni con una tarjeta ni con cinco. Va directo a su salida física.
Si su salida física resulta ser la misma que la de PGM, se suman **en el conector**, no en el bus:
el máster no lo toca y el vúmetro de PGM no lo cuenta. Esto arregla el hallazgo segundo de raíz,
en lugar de parchear el fallback.

Cada uno de los tres buses de programa mantiene su fader propio y su medidor propio. Ahí es donde
la regla se "rompe" como dijiste: si un bus se asigna a una salida física distinta de la de PGM,
**deja de sumar en PGM** y se va por su cuenta, con su fader, sin pasar por el máster. Un bus
sabe a qué salida va; PGM solo suma a los que están asignados a él.

### La consola es un motor propio (decidido 2026-07-16)

La consola nace como **`engine/console/`**, motor propio dentro de la arquitectura "Núcleo +
Motores". No es una pieza suelta dentro de `engine/audio/`, y la razón no es de gusto: es que
**`engine/player/` también la necesita**, y el reproductor no debe depender del motor de efectos.

Con un matiz que conviene fijar, porque cambia el orden de dependencias del proyecto y hay que
reflejarlo en `ARCHITECTURE.md`: los demás motores de audio **producen** señal; la consola **no
produce nada**, la recibe y la encamina. No es un motor *al lado* de `audio/` y `player/`, es un
motor *debajo*: ambos pasan a ser sus clientes. Hay precedente exacto — `engine/cache/` tampoco
produce audio y ya lo comparten los dos motores.

**Efecto colateral bueno:** hoy `engine/player/` importa de `engine/audio/` (`device::find_device`
y `decode::BoxSource`). El reproductor depende del motor de efectos, que no es su par. Al mudar la
propiedad de las tarjetas a `engine/console/`, ese acoplamiento **se rompe**: ambos dependerán de
la consola, no uno del otro.

### Por qué esto encaja con rodio (verificado en la fuente de rodio 0.19)

Tres comprobaciones hechas sobre `~/.cargo/registry/.../rodio-0.19.0`, porque el diseño entero
depende de ellas:

**`Arc<DynamicMixerController<f32>>` es `Send + Sync`, y `add()` toma `&self`.** El controller solo
contiene un `AtomicBool`, un `Mutex` y dos primitivos. Esto es lo que hace viable todo: los motores
`audio/` y `player/` pueden **entregar fuentes a un bus que no es suyo, desde sus propios hilos**,
sin canales nuevos ni cirugía. La consola reparte controllers y cada motor añade cuando quiera.

**`OutputStream` ya es un endpoint.** Por dentro es literalmente `{ mixer:
Arc<DynamicMixerController<f32>>, _stream: cpal::Stream }`, y `play_raw()` añade al mixer. **No hay
que construir el mixer de salida: rodio ya lo trae.** Un bus se enchufa a una tarjeta con
`handle.play_raw(bus)` y varios buses en la misma tarjeta se suman solos, en el conector, sin
tocarse entre ellos. Esto es exactamente la separación señal/conector, y sale casi gratis.

**`OutputStream` no es `Send`** (lleva un `cpal::Stream` dentro). Alguien tiene que ser su dueño y
quedarse quieto en un hilo. **Ese es el argumento de fondo para el motor:** hoy hay *dos* hilos
dueños de tarjetas — `audio/thread.rs` (con `device` y `device_pre`) y `player/thread.rs` (con el
suyo) — y por eso no puede existir un punto de suma común. La consola centraliza esa propiedad en
un hilo guardián y reparte `OutputStreamHandle`, que sí es `Clone + Send + Sync`.

### Los módulos

```
engine/console/
  mod.rs       — ConsoleEngine: el handle Send+Sync que ven los motores
  thread.rs    — hilo guardián: único dueño de los OutputStream
  endpoint.rs  — OutputEndpoint + registro por nombre (una tarjeta se abre UNA vez)
  bus.rs       — Bus: mixer + fader + medidor, y a qué endpoint entrega
  fader.rs     — FaderSource: multiplica por un atómico. Una etapa real.
  device.rs    — find_device / available_devices (se mudan de engine/audio/)

domain/console/
  topology.rs  — qué buses existen y cuáles suman en PGM
  routing.rs   — qué bus le toca a cada fuente. Reglas puras, sin tarjeta de sonido.
```

**`engine/audio/bus.rs` (`MasterBus`) y `engine/audio/device.rs` (`AudioDeviceRuntime`)
desaparecen**, absorbidos por la consola. El `Bus` nuevo es casi el `MasterBus` de hoy **menos el
`Sink`**: esa amputación es el corazón del cambio — separar "soy una señal" de "soy un conector".
Y el `Sink` no se sustituye por otro, se cambia por `play_raw`: un bus nunca se pausa, así que la
capa de control de `Sink` sobra.

Nota que se cae sola: el *"habría hecho falta un tercer `AudioDeviceRuntime`"* que aparece en las
decisiones anteriores **deja de existir como concepto**. No hay N runtimes; hay una consola con N
buses.

### Cómo queda la ganancia

Hoy: `muestra × file_gain × vol_botón × master × fade`, todo dentro de `ButtonSource`.

Propuesto — cada factor en su etapa, que es lo que hace una consola:

```
ButtonSource:  muestra × file_gain × vol_botón × fade     (el canal)
Bus:           × fader_del_bus                            (el bus)
PGM:           × fader_de_máster                          (el máster)
```

El resultado audible es idéntico cuando todos los faders de bus están a 1.0, que es el arranque
por defecto. **Nadie nota nada el día que se cambia.** Pero por primera vez el máster es una
etapa y no un acuerdo entre fuentes, y el reproductor puede obedecerlo si tú decides que debe.

### Lo que el reproductor gana y lo que arriesga

El reproductor deja de abrir tarjeta propia cuando comparte salida: entrega su señal al bus
REPRODUCTOR como cualquier otro. Gana **vúmetro** (hoy no tiene) y gana **fader de bus**.

**Decidido (2026-07-16): el reproductor suma en PGM y obedece al máster.**

Con una precisión que conviene fijar por escrito, porque son dos cosas distintas y es fácil
confundirlas:

- **Bajar la música para hablar encima** → eso es el **fader del bus REPRODUCTOR**, no el máster.
  Baja solo la música y deja los efectos intactos. Esto **ya se puede hacer hoy** con
  `player_set_volume`; lo que aporta la consola es hacerlo visible como una tira de canal con su
  vúmetro al lado, en vez de un botón de volumen escondido.
- **Bajar todo a la vez** → eso es el **máster**. Baja música, efectos y panel juntos.

Son complementarios y se tienen los dos. El caso de uso del locutor lo resuelve el primero; que
el máster gobierne de verdad toda la aplicación lo resuelve el segundo.

**"Obedecer al máster" es volumen, no transporte.** El Stop general y el Solo de los efectos
siguen **sin** tocar el reproductor: eso está decidido desde el diseño del modo reproductor y este
plan no lo cambia. El máster es un fader; parar es otra cosa.

El riesgo técnico real: los decks de rodio hoy usan `sink.get_pos()` para la posición y
`sink.empty()` para saber que la pista acabó. Si el deck deja de ser un `Sink` y pasa a ser una
fuente dentro de un mixer, **hay que sustituir esas dos señales**, y de ellas dependen la barra de
progreso, el contador y el relevo ping-pong entre decks. Es la parte más delicada del plan y por
eso va en su propia fase, sola, sin mezclar con nada.

---

## 4. Fases

Cada fase compila, pasa las 118 pruebas y **se puede parar ahí sin dejar el motor a medias**.

**Fase 1 — Nace `engine/console/` y separa la salida del bus.** El hilo guardián toma la propiedad
de los `OutputStream`; nace `OutputEndpoint` con su registro por nombre. El `MasterBus` pierde su
`Sink`, se convierte en `Bus` y se enchufa al endpoint con `play_raw`. `engine/audio/thread.rs`
deja de ser dueño de tarjetas y pasa a pedir buses a la consola. Los tres motores siguen sonando
igual. Beneficio inmediato y aislado: **la misma tarjeta deja de abrirse dos veces**.

**Fase 2 — El fader existe.** Nace `FaderSource`. `master_volume` sale de `ButtonSource` y pasa a
ser el fader del bus. Cero cambio audible. `ButtonSource` adelgaza. La aritmética de ganancia
queda documentada donde toca.

**Fase 3 — Buses con nombre y la matriz.** Muere el booleano `to_pre`; nace `Console`. El CUE deja
de caer al PGM: cuando comparte tarjeta, se suma en el endpoint y no en el bus. **Aquí se arregla
el hallazgo segundo.** Las reglas de ruteo nacen en `domain/console/` con pruebas propias.

**Fase 4 — El reproductor se une a la consola. Obligatoria.** Al decidirse que el reproductor
obedece al máster, esta fase deja de ser opcional: para sumar en PGM tiene que dejar de ser un
`Sink` suelto. Es la fase delicada — reemplazar `sink.get_pos()` y `sink.empty()` por señales
propias del bus, de las que dependen la barra de progreso, el contador y el relevo entre decks.
Se hace **sola**, después de que lo demás esté firme, y se prueba antes de seguir.

**Fase 5 — Vúmetros por bus.** Cada bus expone su nivel. El evento `audio-tick` crece con los
niveles por bus (campos nuevos, opcionales). El reproductor por fin tiene vúmetro.

**Fase 6 — La interfaz de la consola.** Cuatro tiras de canal con su fader y su vúmetro, más el
máster. Recibe niveles y estado del ruteo, dibuja y manda comandos. **No calcula ni un dB.** La
regla 4 se cumple sola porque para entonces no queda nada que calcular en JS: el motor ya lo sabe
todo.

### Orden de trabajo acordado

Tres tandas, con prueba tuya entre una y otra:

1. **Fases 1 a 3.** No se ven: la aplicación suena igual, pero deja de comportarse distinto según
   el equipo y se arreglan los cuatro hallazgos. Si algo sonara raro, la causa queda acotada a
   estas tres.
2. **Fase 4 sola.** La arriesgada, y la única que toca un reproductor que hoy funciona bien.
   Aislada a propósito: si algo se tuerce, se sabe exactamente dónde.
3. **Fases 5 y 6.** La consola se hace visible. La interfaz llega la última, cuando ya no queda
   ninguna tentación de que decida nada.

---

## 5. Impacto en compatibilidad y rendimiento

**`.bdelf` / `.bdeplf` (regla 6).** El ruteo es de la aplicación y del equipo, no de la paleta:
un `.bdelf` que viaje a otro ordenador con otra tarjeta no debe arrastrar ruteo. Por eso propongo
que **la asignación de bus se derive del grupo, no se guarde por botón**: cero campos nuevos en
`ButtonData`, cero superficie de incompatibilidad. La configuración de la consola vive en
`AppConfig`, y si se añade allí va con `#[serde(default)]` como manda la regla.

**Rendimiento.** Se añaden dos etapas por muestra (fader de bus y mixer de salida). A 48 kHz
estéreo son unos 96.000 pasos más por segundo, cada uno una multiplicación en `f32`. En Rust
nativo es ruido estadístico frente a la decodificación que ya hacemos. A cambio se quita una
lectura atómica por muestra y por fuente dentro de `ButtonSource` (el `master_volume`), que hoy
se paga tantas veces como fuentes suenen a la vez.

**Trampa de rodio ya conocida.** Cada mixer necesita su `Zero::new` de arranque o devuelve `None`
en vacío y el `Sink` se para para siempre (está comentado en `bus.rs:86-88`). Cada bus y cada
endpoint nuevo necesita el mismo cuidado. Está identificado; no es un descubrimiento pendiente.

---

## 6. Fuera de alcance (a propósito)

**El DSP.** El LFA tiene EQ, compresor y limitador en una `DynamicDspSource`. Aquí **no lo
propongo**. Esto es una botonera de efectos, no una cadena de aire, y meter DSP en el mismo
movimiento que el ruteo convertiría un cambio verificable en un proyecto entero. La arquitectura
deja el punto de inserción listo (entre el fader del bus y el medidor); si algún día lo quieres,
entra sin rediseñar nada. Pero es otra conversación.

**El encoder / streaming.** No existe en la Botonera y no lo propongo. Si algún día se quiere, el
modelo de oyente del LFA encaja con esta arquitectura sin tocarla.

---

## 7. Dos decisiones ya cerradas con las que esto roza

Las señalo porque la regla 9 pide alineación antes de reestructurar, y porque **no quiero
re-proponer algo que ya descartaste**.

**Salida propia del panel fijo — descartada por ti hoy mismo (2026-07-16).** Está en
`PLAN_MODO_REPRODUCTOR.md:551-557`, y la razón registrada fue el coste: *"Habría hecho falta un
tercer `AudioDeviceRuntime` y cambiar el booleano `to_pre` por una decisión de tres — cirugía en
el corazón del motor de efectos"*.

Conviene que lo sepas: **este plan es exactamente esa cirugía**, hecha por su propio motivo. Si se
hace, la salida propia del panel fijo pasa de "cirugía en el corazón del motor" a una línea en la
matriz, porque el bus PANEL ya existiría. **No te la estoy proponiendo** — la descartaste y el
documento pide no volver a plantearla. Solo dejo constancia de que el argumento que la tumbó era
de coste, y este plan cambia ese coste. Si quieres que siga descartada por otros motivos, sigue
descartada y no vuelvo a mencionarla.

**`PaletaData.audio_out`.** Mismo caso: hoy es una trampa documentada. Con la consola *podría*
cumplirse por fin. Tampoco lo propongo; lo dejo como consecuencia disponible.

---

## 8. Decisiones tomadas (2026-07-16)

1. **El reproductor obedece al máster: sí.** Suma en PGM. Motivo del autor: que el operador pueda
   gobernar la música al hablar. Matiz registrado en la sección 3: ese caso concreto lo resuelve
   el *fader del bus*, no el máster; se tienen los dos y son complementarios. El Stop general y el
   Solo siguen sin tocar el reproductor — el máster es volumen, no transporte.

2. **Faders por bus: sí, los cuatro.** Efectos, Panel, Reproductor y CUE, cada uno con su fader y
   su vúmetro, más el máster. Es una consola de verdad, no un volumen con adornos.

3. **Alcance: las seis fases**, en las tres tandas de la sección 4, con prueba del autor entre
   tanda y tanda.

4. **La Fase 4 entra, y es obligatoria.** Es consecuencia directa de la decisión 1: no hay forma
   de que el reproductor obedezca al máster sin unirlo a la consola.

---

## 9. Resumen en tres frases

Hoy hay tres motores que se ignoran, un máster que no es una etapa sino un acuerdo, y una
pre-escucha que se cuela en el programa cuando solo tienes una tarjeta. La consola virtual separa
*señal* de *conector*, que es la distinción que pediste y la que hoy el código no puede expresar.
Se puede hacer en fases pequeñas y verificables, y las tres primeras ya pagan solas aunque nunca
se dibuje una sola tira de canal.
