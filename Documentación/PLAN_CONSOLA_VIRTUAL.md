# Plan propuesto — Consola de audio virtual

Documento de lectura y discusión. **No se ha tocado código.** Sesión del 2026-07-16.

Guía externa consultada: `C:\LF Automatizador v1.0\entendiendo la consola virtual.md`.
Reglas aplicadas: `Documentación/REGLAS_PROYECTO.md` (en especial la 4, la 5 y la 9).

---

## 1. Cómo viajaba el audio antes de la Fase 1

> **Nota (Fase 1 completada):** esta sección es la **auditoría de partida**, y describe el motor
> tal como estaba **antes** de que naciera `engine/console/`. Se conserva porque explica *por qué*
> se hizo lo que se hizo. Para el estado actual, ver `ARCHITECTURE.md` y el registro de la
> sección 10. Los cuatro hallazgos siguen siendo el mapa de lo que falta: la Fase 1 arregló la
> propiedad de las tarjetas, no el fallback de la pre-escucha ni el master.

Esto es lo que había en el disco, leído módulo a módulo. Es importante porque tres de los cuatro
problemas de abajo no se ven desde la interfaz.

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

**Fase 4.5 — Cambio de tarjeta en caliente.** Pedida por el autor el 2026-07-16 tras probar la
Fase 3: cambiar de salida **no debe callar nada** — ni la botonera, ni el panel, ni el
reproductor.

**Sin cortar del todo es imposible, y conviene saber por qué:** al soltar un `OutputStream`, su
mixer vive *dentro* del callback de cpal, así que las fuentes que contiene mueren con él. No hay
forma de sacarlas y llevarlas a la tarjeta nueva. Las alternativas que lo permitirían —un
`Arc<Mutex>` leído muestra a muestra, o un ring buffer con hilo de render— meten un candado o
latencia en el camino del audio a cambio de una operación que se hace una vez al configurar el
equipo. No compensa.

**Lo que sí se hace:** reconstruir cada fuente en la tarjeta nueva **por la posición en la que
iba**. La música no se pierde, sigue donde estaba; hay un salto de milisegundos, que además es
inevitable porque el altavoz físico también cambia. La pieza ya existe: `seek_source.rs` (Fase
E.4 del reproductor) reconstruye en ~10 ms sea cual sea la distancia, y `playback_seek` ya lo hace
para un botón — aquí es para todos.

**Va después de la Fase 4 a propósito.** Hacerlo antes obligaría a escribirlo dos veces: una para
los efectos y otra para el reproductor cuando entre al bus. Con el reproductor ya dentro, se
resuelve una vez para los tres.

Lo que hay que resolver, anotado antes de empezar:
- `replays` solo guarda los botones de la botonera principal (`main_button`); harían falta también
  los del panel.
- Las locuciones (`SequenceSource`) son varios archivos encadenados y **no se pueden
  reposicionar**: al cambiar de tarjeta habrá que decidir si se reinician o se dejan caer.
- Un botón con `overlap` tiene varias instancias del mismo id, cada una en su posición.

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

## 9. Registro de fases

### Fase 1 — completada (2026-07-16)

**Nace `engine/console/`.** El hilo guardián toma la propiedad de las tarjetas y los motores
pasan a pedirle buses. Sin cambio audible: la aplicación suena exactamente igual.

**Lo que se movió:**

- Nacen `engine/console/`: `mod.rs` (`ConsoleEngine`, `BusId`, `BusSlot`), `thread.rs` (guardián),
  `endpoint.rs` (`OutputEndpoint` + registro por nombre), `bus.rs` (`Bus`), `level.rs`
  (`LevelSource`), `device.rs` (`find_device` / `available_devices`).
- **Desaparecen** `engine/audio/bus.rs` (`MasterBus`), `engine/audio/device.rs`
  (`AudioDeviceRuntime`), `engine/audio/device_list.rs` y `engine/audio/vu.rs`, absorbidos.
- `MasterBus::add_source` se convierte en `engine/audio/attach.rs::attach_button`. **Se queda en
  el motor de efectos a propósito:** un bus solo sabe sumar fuentes; que un botón tenga fades,
  trim, estado y grupo es asunto de quien sabe de botones. La consola no sabe qué es un botón.
- `SequenceSource` sale a `engine/audio/sequence.rs`; `LastPressedInfo` a
  `engine/audio/last_pressed.rs` (estaba mezclado con `LevelSource`, que es del bus).
- `AudioEngine` deja de crear sus atómicos: se los pide a la consola, porque los niveles y el
  volumen **son del bus** y sobreviven a que la tarjeta se caiga o se cambie.
- `get_available_devices` se cae de `AudioEngine`: listar tarjetas no es asunto del motor de
  efectos. `cmd_audio` se lo pregunta a la consola.

**Se rompió un acoplamiento:** `engine/player/` importaba `device::find_device` del motor de
efectos. Ahora ambos dependen de la consola, no uno del otro.

**Un bug cazado durante la fase.** La primera versión limpiaba los estados de botón en *todo*
`SetDevice`, pero el `AudioDeviceRuntime` original salía temprano si el dispositivo era el mismo
y su bus vivía. Reaplicar la misma salida (al arrancar, al reconectar) habría callado la botonera.
El hilo de efectos recuerda ahora la última tarjeta pedida y solo limpia si de verdad cambia — o
si el bus no existe, que también obliga a reabrir.

**Lo que la Fase 1 NO hizo, y conviene no dar por hecho:**

- **La doble apertura de tarjeta sigue.** El reproductor conserva su `OutputStream` propio. Se
  decidió sacarlo de esta fase: unirlo ahora obligaba a modelar un "cliente de endpoint" que la
  Fase 4 tira, porque allí el reproductor pasa a ser un bus. Era construir un andamio para
  demolerlo. Se arregla en la Fase 4.
- **El fallback de la pre-escucha sigue intacto**, a propósito: es la Fase 3.
- **El master sigue aplicándose por fuente**, a propósito: es la Fase 2.

**Verificación:** `cargo build --lib` sin avisos (que es como compila el usuario), `cargo test
--lib` con 140 pruebas (3 nuevas sobre qué tarjetas siguen en uso), `npm run build` correcto,
ningún archivo sobre 200 líneas. `engine/audio/engine.rs` llegó a 201 y bajó a 197 quitando el
passthrough de dispositivos, no recortando comentarios.

**Nota sobre las pruebas nuevas.** Solo cubren `devices_in_use`, la regla pura de qué tarjeta
sigue haciendo falta. Abrir tarjetas de verdad necesita hardware, así que `EndpointRegistry` y
`Bus` **no tienen prueba unitaria**: son envoltorios finos sobre rodio y cpal. Que la misma
tarjeta se abra una sola vez se comprueba en el equipo del autor, no aquí.

### Fase 2 — completada (2026-07-16)

**El master pasa a ser una etapa.** Nace `engine/console/fader.rs` con `FaderSource`, y
`master_volume` sale de `ButtonSource`. Sin cambio audible.

**Por qué el resultado es idéntico:** por distributividad. Antes cada fuente se multiplicaba por
el master antes de entrar al mixer, `Σ(sᵢ × m)`; ahora se suman y el fader multiplica una vez,
`Σ(sᵢ) × m`. El mismo número. De hecho ahora sale **una** multiplicación y **una** lectura
atómica por muestra, en lugar de una por cada fuente sonando.

**La decisión que hace la fase neutra: dónde va el medidor.** La cadena del bus queda
`mixer → FaderSource → LevelSource → play_raw`, con el medidor **después** del fader. Así el
vúmetro sigue enseñando lo que de verdad sale — que es como se comportaba cuando cada fuente
aplicaba el master antes de entrar al mixer, y es lo que hace el medidor de programa de una
consola. Puesto al revés habría medido antes del fader y la aguja no se habría enterado de nada:
eso sí habría sido un cambio de comportamiento, y de los que no se ven venir.

**El fallback de la pre-escucha sigue comportándose igual**, que era el riesgo de esta fase:
cuando cae al bus `Main` pasa por el fader de `Main` y le sigue pegando el master, exactamente
como antes. Con tarjeta dedicada va al bus `Pre`, cuyo fader está a 1.0 y no hace nada. La Fase 3
es la que lo cambia.

**Sin rampa, a propósito.** El master ya saltaba de golpe (cada fuente leía el atómico y
multiplicaba), así que suavizarlo aquí habría cambiado el sonido en la misma fase que mueve la
aritmética de sitio, y no se sabría qué causó qué si algo suena raro. Cuando el fader sea visible
y se arrastre de verdad (Fase 6) se decide si hace falta.

**Lo que NO se movió, y toca en la Fase 3:** `AudioEngine::master_volume` / `set_master_volume`
siguen donde estaban, como paso al atómico del slot `Main`. En la Fase 3 nace el bus PGM y el
master pasa a ser *su* fader, así que esa API se muda entonces — moverla ahora sería tocarla dos
veces.

**Verificación:** `cargo build --lib` sin avisos, `cargo test --lib` con 143 pruebas (3 nuevas
sobre el fader: que escala, que se mueve mientras suena y que a cero calla sin detener la
fuente), `npm run build` correcto, ningún archivo sobre 200 líneas.

### Fase 3 — completada (2026-07-16)

**Nacen los buses con nombre y muere el fallback.** Es la primera fase que **cambia
comportamiento a propósito**, y por eso es la primera que va al `CHANGELOG`.

**El arreglo, que era el hallazgo segundo de la auditoría:** el bus `Cue` existe **siempre**. Si
no tiene tarjeta propia usa `Routing::ProgramDevice` — sale por la tarjeta del programa, pero con
su propio `play_raw`, su fader y su medidor. Se suma con el programa **en el conector, no en el
bus**. Resultado: la pre-escucha ya no recibe el máster ni mueve el vúmetro aunque solo haya una
tarjeta. Deja de comportarse distinto según el equipo.

`Routing::ProgramDevice` es la variante que carga con toda la idea de la consola: que dos cosas
salgan por el mismo altavoz no las convierte en la misma señal.

**La topología:**
```
  Efectos ─┐
           ├─► Programa ─► fader (master) ─► medidor (vúmetro) ─► tarjeta
  Panel ───┘
  Cue ──────────────────► fader ──────────► medidor ──────────► tarjeta
```

**El máster se mudó a la consola.** Es el fader del bus `Programa` y el vúmetro es su medidor: los
dos miran el mismo bus, que es la única forma de que la aguja no mienta sobre lo que el fader
controla. `cmd_master_volume` y `cmd_profiles` se lo piden ahora a `console.fader(BusId::Programa)`
en vez de a `AudioEngine`, que solo aporta uno de los buses que suman en el programa. Esto estaba
anunciado en el registro de la Fase 2.

**El vúmetro principal mide Efectos + Panel**, igual que antes (el bus `Main` recibía los dos
grupos). Los botones del panel fijo siempre se han reflejado en él; lo que no aparece todavía es
el reproductor, que sigue siendo un motor aparte — Fase 4.

**`domain/console/` nace con las reglas puras:** `BusId`, `Routing`, `sanitize`, `device_of`,
`devices_in_use`. Trece pruebas sin tocar una tarjeta de sonido. `sanitize` **no es programación
defensiva**: pedir que la pre-escucha suene "en el programa" no es un error de un llamante que
haya que silenciar, es una petición que la consola traduce a lo único que puede significar.

**El grafo se reconstruye entero ante cualquier cambio de ruteo.** rodio no sabe sacar una fuente
de un mixer, así que remendar dejaría el mixer de un bus viejo colgado dentro del programa para
siempre, sonando a silencio y sin que nadie pueda quitarlo. Reconstruir corta lo que suene —igual
que ya hacía cambiar de tarjeta— pero no acumula basura.

**Muere el booleano `to_pre`** como decisión: `routing::bus_for(to_pre, group)` devuelve un
`BusId`, y `AudioCommand::Play` lleva el bus, no un booleano. El hilo ya no elige entre dos
salidas fijas con un `if`.

**Se corrigió una incoherencia de paso:** las locuciones (`PlaySequence`) iban siempre al bus
principal aunque las disparara un botón del panel fijo. Ahora van al bus de su grupo, como
cualquier otro botón.

**Dos archivos pasaron de 200 líneas y se partieron sin recortar nada:** las pruebas de ruteo
salieron a `routing_tests.rs` (patrón que el proyecto ya usa), y `engine.rs` adelgazó al mudar el
máster a la consola, que era su sitio.

**Verificación:** `cargo build --lib` sin avisos, `cargo test --lib` con 152 pruebas (12 nuevas),
`npm run build` correcto, ningún archivo sobre 200 líneas.

**Lo que sigue pendiente:** el reproductor conserva su `OutputStream` propio, así que la doble
apertura de tarjeta y su ausencia del vúmetro siguen. Los faders de `Efectos`, `Panel` y `Cue`
existen y están a 1.0, pero no se ven ni se pueden mover: eso es la Fase 6.

### Fase 4 — completada (2026-07-16)

**El reproductor entra en la consola.** Era la fase delicada: la única que tocaba un motor que
funcionaba bien. Con ella se cierran los cuatro hallazgos de la auditoría.

**Los decks dejan de ser `Sink`.** Nace `engine/player/source.rs` con `DeckSource` y `DeckHandle`,
que sostienen a mano las tres cosas que el `Sink` daba gratis y que una fuente metida en un mixer
ya no puede responder:

- **Posición** (`sink.get_pos`) → contar las muestras consumidas.
- **Terminó** (`sink.empty`) → un flag que la fuente marca al agotarse.
- **Pausa** (`sink.pause`) → un flag; en pausa devuelve silencio **sin pedir muestra** a la fuente,
  así que al reanudar sigue por donde iba. Devolver `None` la retiraría del mixer y pausar sería
  en realidad parar. Seis pruebas cubren justo esto, porque de ello dependen la barra de progreso,
  el contador y el relevo entre canciones.

**El volumen del reproductor resultó ser el fader de su bus**, y eso no estaba planeado: sale
gratis y resuelve los dos gestos que se confundían en la conversación inicial. Bajar la música
para hablar encima es mover el fader del bus `Reproductor` (no toca los efectos); bajarlo todo es
el máster, que es el fader de `Programa`. Los dos se multiplican, así que se tienen ambos sin
elegir. El `DeckSource` solo aplica la ganancia de **su** pista, que es del archivo.

**Se cierra el hallazgo tercero:** el reproductor ya no abre una segunda vez la tarjeta que
comparte con los efectos. Ahora entrega al bus como cualquier otro.

**Se cierra la pregunta del autor sobre el vúmetro:** el bus `Programa` suma `Efectos`, `Panel` y
`Reproductor`, así que el vúmetro los muestra a los tres. Hubo que arreglar el monitor, que
usaba "no hay botones" como sinónimo de "no hay nada sonando" y habría dejado la aguja plana con
música de fondo. Ahora el reposo es "ni efectos ni reproductor", que es lo que de verdad significa
que el bus esté en silencio.

**Un bug que el compilador no podía ver.** `cmd_player_config` y `core/setup.rs` traducían el
dispositivo vacío al nombre de la salida principal. Con la consola eso ya no es lo mismo: `""`
significa `Routing::Program` (suma en el programa, obedece al máster) y un nombre significa
`Routing::Device` (salida directa, ajena al máster). Traducirlo habría dejado al reproductor
sonando por su cuenta en la misma tarjeta — exactamente lo que hacía antes — y la decisión del
autor no se habría cumplido, sin que nada fallara.

**Verificación:** `cargo build --lib` sin avisos, `cargo test --lib` con 158 pruebas (6 nuevas),
`npm run build` correcto, ningún archivo sobre 200 líneas.

**Lo que la Fase 4 NO hace:** cambiar de tarjeta sigue cortando lo que suena. Es la Fase 4.5, que
ahora ya se puede hacer una sola vez para los tres motores.

### Fase 4.5 — completada (2026-07-16)

**Cambiar de salida ya no calla nada.** Ni la botonera, ni el panel, ni la música.

**Cómo, ya que mover las fuentes es imposible:** rodio no sabe sacar una fuente de un mixer, así
que al rehacer el grafo mueren con sus buses. No se mueven — **se vuelven a crear en el segundo
por el que iban**. Hay un salto de milisegundos, que además es inevitable porque el altavoz físico
también cambia, pero nada se pierde. La pieza ya existía: `seek_source.rs` reconstruye en ~10 ms
sea cual sea la distancia.

**Cómo se enteran los motores:** la consola lleva una **generación** que sube en cada `rebuild`.
Un motor que la ve cambiar sabe que lo que estaba tocando ya no existe.

- **Efectos y panel:** el hilo de audio dispara el cambio, así que lo hace en el acto —
  `set_bus_routing_sync` (espera a que el grafo esté listo; entregar antes sería dárselo al bus
  muerto) y `reattach::reattach_all`.
- **Reproductor:** su bus también muere aunque el cambio no sea suyo (el grafo se rehace entero).
  Lo detecta en su tick de 100 ms comparando la generación. Se rehace **antes** de mirar si el
  deck terminó: si no, el motor lo tomaría por fin de pista y saltaría a la siguiente canción.

**La ficha de reconstrucción se mudó al estado.** `ReplayInfo` vivía en un `HashMap` por id dentro
del hilo, mantenido a mano en cada Stop, StopAll y `stop_other`. Ahora va en el `ButtonState`, y
no es un capricho: **un botón con `overlap` tiene varias instancias sonando a la vez, cada una por
su sitio**, y una ficha compartida por id las habría rehecho todas en la misma posición.

De paso desaparece esa contabilidad entera: el mapa `replays` era un segundo censo de lo mismo, y
`seek_active` lo lee ahora del estado que suena. Dos censos de lo mismo solo sirven para
contradecirse. El hilo de audio adelgazó de 175 a 132 líneas.

**Lo que no se puede rehacer se dice cuál y se deja caer:** las locuciones son varios archivos
encadenados en una sola fuente y no admiten reposicionarse (`replay: None`). Se marcan terminadas
para que el motor releve y la lista siga, en vez de callarse esperándolas.

**Detalles que importan al oído:**
- **Sin fade de entrada al rehacer:** no es un disparo nuevo, es la misma fuente que sigue. Un
  fundido ahí se oiría como un bache.
- **Se usa el volumen en vivo, no el de la ficha:** el operador pudo haberlo movido después de
  disparar (la barra de pre-escucha lo hace).
- **En bucle se retoma la vuelta, no el total** (`rem_euclid`): la fuente nueva arranca dentro del
  archivo, y el archivo dura una vuelta.

**Verificación:** `cargo build --lib` sin avisos, `cargo test --lib` con 164 pruebas, `npm run
build` correcto, ningún archivo sobre 200 líneas (`deck.rs` llegó a 214 y sus datos salieron a
`deck_track.rs`).

#### Corrección posterior: los buses viejos no morían (2026-07-16)

El autor reportó dos síntomas tras probar la 4.5: la música dejó de moverse en el vúmetro, y el
vúmetro de los botones daba huecos con una canción larga sonando. Eran el mismo bug.

**La causa:** `rebuild` hacía `live.clear()`, que suelta el *handle* del `Bus` — pero **no retira
su cadena del `OutputStream`**. A un mixer de rodio no se le quita una fuente: la única forma de
sacar algo de ahí es que se agote, y la cadena del bus no se agota nunca (tiene su `Zero`).

Cuando el grafo se rehace **sin cambiar de tarjeta** —cambiar la salida del reproductor o de la
pre-escucha, por ejemplo— el endpoint del programa no se cierra, así que el bus viejo seguía
dentro, vivo. Y como **los atómicos del medidor son del `BusSlot` y sobreviven a la
reconstrucción**, el viejo y el nuevo escribían el mismo nivel a la vez. El viejo, ya sin fuentes,
escribía cero: de ahí el parpadeo. Cada cambio de ruteo acumulaba un fantasma más por bus.

**El arreglo:** `Bus::close()` y `BusOutlet`, el grifo al final de la cadena. Al cerrarse devuelve
`None` y el padre lo deja caer — el mismo patrón que ya usaba `DeckSource` para retirarse del bus.
`rebuild` cierra los viejos **antes** de soltarlos.

**La prueba se comprobó reintroduciendo la regresión**, y el número que dio es el diagnóstico
entero: `salió 1.5` — 1.0 del bus viejo más 0.5 del nuevo, sumando los dos a la vez.

**Por qué no se vio antes:** las fases 1 y 2 tenían **un solo bus por tarjeta**, y cambiar de
tarjeta cierra el endpoint, que se lleva la cadena por delante. El fantasma solo aparece al rehacer
el grafo **conservando la tarjeta**, que es algo que no existía hasta que hubo varios buses (Fase
3) y ruteos que se cambian por separado (Fase 4).

#### Corrección posterior: pedir la tarjeta del programa es pedir el programa (2026-07-16)

El autor reportó que el reproductor **no se veía en el vúmetro** al elegirle "Altavoces" por su
nombre, pero **sí** al elegir "la misma que los efectos" — siendo la misma tarjeta.

El código hacía exactamente lo diseñado: por nombre es `Routing::Device` (salida directa, ajena al
programa) y vacío es `Routing::Program`. Coherente por dentro, **mal diseño por fuera**: dos formas
de decir lo mismo que suenan distinto. Y el caso de uso que justificaría la diferencia —dos señales
en el mismo altavoz, una con máster y otra sin— no lo quiere nadie.

Nace `routing::effective(bus, routing, program_device)`: si un bus que puede sumar en programa pide
**la tarjeta del programa**, se resuelve a `Program`. El **CUE queda excluido** por
`can_sum_into_program`, y esa exclusión es la razón de ser de la consola: comparte el altavoz *a
propósito* sin sumar. Si el programa se muda después, el bus se queda en su tarjeta y pasa a ser
salida directa solo, sin que nadie lo toque.

**Pendiente conocido:** `"default"` y el nombre real de esa misma tarjeta se tratan como distintos,
porque la comparación es por nombre. Programa en `"Altavoces"` y reproductor en `"default"` —siendo
Altavoces el predeterminado— abriría la tarjeta dos veces. Resolverlo pide normalizar `"default"` a
su nombre real al abrir el endpoint. No se ha hecho: es un caso raro y la Fase 6 puede evitarlo
desde el selector.

#### Pruebas contra tarjetas reales (2026-07-16)

A petición del autor, antes de conectar la interfaz. `src-tauri/tests/consola_tarjetas_reales.rs`,
marcadas `#[ignore]` porque necesitan hardware — una prueba que falla por el entorno no dice nada:

```bash
cargo test --test consola_tarjetas_reales -- --ignored --nocapture --test-threads=1
```

Montan la `ConsoleEngine` de verdad sobre las tarjetas del equipo y comprueban lo que los mixers de
mentira no pueden. Ejecutadas sobre las dos tarjetas del autor (*Altavoces (High Definition Audio
Device)* y *AUDIO PCI (C-Media PCI Audio Device)*):

- **Reproductor en la tarjeta del programa, por su nombre** → programa `0.8`. El caso reportado.
- **Reproductor en otra tarjeta** → programa `0`, reproductor `0.8`. Sale del máster, como debe.
- **Cinco reconstrucciones del grafo sin cambiar de tarjeta** → `[0.8 ×10]`, ni un cero.
- **Altavoces → AUDIO PCI → Altavoces** → `0.8` en los tres pasos.

**La tercera se comprobó reintroduciendo la regresión**, y el resultado es el síntoma del autor
palabra por palabra: `[0.8, 0.0, 0.0, 0.0, 0.8, 0.0, 0.0, 0.8, 0.8, 0.0]`. Esos ceros intercalados
son "los espacios vacíos cuando la música está sonando normalmente".

### Fase 5 — completada (2026-07-16)

**Cada bus expone su nivel.** El `audio-tick` lleva ahora `buses {efectos, panel, reproductor,
cue}`, que es lo que necesita una tira de canal por bus.

`master_level_l/r` **se conserva** y sigue siendo el bus `Programa`: el vúmetro de la barra
inferior lo lee desde siempre y no había motivo para moverlo.

**Nace `engine/audio/tick.rs`** con lo que viaja en el tick y `LevelTaps`, que pide los atómicos de
los cinco buses **una vez** al arrancar el monitor. No se vuelven a pedir: son del `BusSlot` y
sobreviven a que el grafo se rehaga, así que valen para toda la vida del monitor — pedirlos en cada
tick sería tomar el candado de la consola diez veces por segundo para nada. `monitor.rs` adelgazó
de 163 a 130 líneas y el monitor dejó de recibir atómicos sueltos: pide lo que necesita a la
consola.

**En reposo van todos a cero**, por lo mismo que el programa: los atómicos aún pueden retener el
último pico. También el CUE, y no es un descuido — la pre-escucha suena por el motor de efectos y
cuenta como botón, así que si estuviera sonando no habría reposo.

**Verificación contra las tarjetas reales**, a petición del autor y antes de tocar la interfaz. Las
pruebas se repartieron por tema, con el rig común en `tests/common/mod.rs`:

```bash
cargo test --test consola_ruteo_real --test consola_vumetros_real -- --ignored --nocapture --test-threads=1
```

Lo que dieron sobre las dos tarjetas del autor:

- **Cada bus mide lo suyo** → efectos `0.5` · panel `0.25` · reproductor `0` · programa `0.75`.
  La suma exacta, y el reproductor callado con su aguja quieta.
- **Bajar la música a la mitad** → efectos `0.5` (intacto) · reproductor `0.125` · programa
  `0.625`. Es el gesto del locutor: baja la música sin tocar los efectos.
- **Máster a la mitad** → efectos `0.8` (intacto) · programa `0.4`. Cada tira enseña lo que
  *aporta*, no lo que sale al aire.

Esa aritmética es la de una consola funcionando, y es justo lo que no se podía ver con mixers de
mentira.

**Lo que la Fase 5 NO hace:** nadie lee `buses` todavía. Los datos existen y están probados; la
tira de canal que los pinta es la Fase 6.

#### Corrección posterior: rehacer el grafo lo dispara cualquiera (2026-07-16)

El autor reportó que cambiar la salida **del reproductor** dejaba muda **la botonera**, y que el
sonido volvía al abrir los ajustes y guardar sin tocar nada. Esa segunda pista es la que resuelve
el caso.

**La causa, y es un fallo de diseño de la Fase 4.5:** el hilo de audio rehacía sus fuentes dentro
de `AudioCommand::SetBusRouting`, es decir, **solo cuando el cambio lo pedía él**. Pero mover un
bus rehace el grafo **entero**, y el de la salida del reproductor lo dispara su propio hilo
llamando a la consola directamente. Las fuentes de la botonera morían con su bus y nadie las
rehacía, porque por el canal del motor de efectos no pasaba nada que lo delatara.

Y por eso "revivía" al guardar los ajustes: `_saveSettings` llama **siempre** a `set_pre_device`,
que sí entra por el hilo de audio y disparaba el rehacer.

**El arreglo:** el motor de efectos mira la generación **en su tick**, como ya hacía el
reproductor, en vez de fiarse de haber pedido el cambio. Su bucle pasa de `for cmd in rx` a
`recv_timeout(100ms)`. No es un pulso de trabajo — sigue despertando al instante con cada comando;
solo deja de estar ciego cuando no le llega ninguno.

De paso el `match` gigante sale a `handle()`, y `SetBusRouting` se queda en una línea: pedir el
cambio y ya. Rehacer es del bucle, que es quien sabe si el grafo cambió.

**La lección, anotada como trampa en `CLAUDE.md`:** con la consola, el ruteo es de todos. Un motor
no puede enterarse de lo que le pasa a su bus escuchando solo su propio canal.

---

## 10. Resumen en tres frases

Había tres motores que se ignoraban, un máster que no es una etapa sino un acuerdo entre fuentes,
y una pre-escucha que se cuela en el programa cuando solo tienes una tarjeta. La consola separa
*señal* de *conector*, que es la distinción que el código no podía expresar: la Fase 1 ya le dio
un dueño único a las salidas, y quedan el fader (Fase 2) y el fin del fallback (Fase 3). Se hace
en fases pequeñas y verificables, y las tres primeras pagan solas aunque nunca se dibuje una sola
tira de canal.
