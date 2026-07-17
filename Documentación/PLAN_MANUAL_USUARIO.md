# Plan — Manual de usuario de LF Botonera

> **Estado: propuesto, sin empezar.** Este documento no es el manual: es cómo hacerlo.
> Está escrito para una sesión futura que no recuerde nada de esta conversación.
> Fecha: 2026-07-17.

---

## 1. Para quién es, y por qué no vale lo que ya hay

El manual es para **quien dispara los efectos en directo**, no para quien compila el
proyecto. Y no es un solo público, son cuatro: **operadores de radio, locutores,
streamers y DJs**. Comparten la mecánica —una rejilla de sonidos y las manos ocupadas
mientras algo está saliendo al aire— pero no el vocabulario, ni el equipo, ni las mismas
necesidades.

**Consecuencia práctica, y hay que tenerla presente desde el primer capítulo:** ni todos
los capítulos sirven a los cuatro, ni se puede dar por supuesto un estudio de radio. Las
locuciones de hora y clima son de radio; un streamer no las va a tocar en su vida. Al
revés, mandar la música a una tarjeta distinta de la de los efectos le importa muchísimo a
quien emite por internet y menos a quien tiene una mesa física delante. **Cada capítulo
debería decir a quién sirve en su primera línea**, para que nadie lea seis páginas antes
de descubrir que no era para él.

Eso separa el manual de todo lo que ya existe:

- **`LIBRO_PROYECTO.md`** explica cómo funciona por dentro. Es para quien toca el código.
  Quien pincha los efectos no necesita saber qué es un `DynamicMixerController`.
- **`README.md`** tiene una lista de características. Es un escaparate, no una guía: dice
  *qué hay*, no *cómo se usa*.
- **`GLOSARIO.md`** define términos internos.

No hay nada, dentro ni fuera de la aplicación, que le explique a ninguno de los cuatro
cómo trabajar.
Esa es la laguna.

---

## 2. La decisión de fondo: dónde vive el manual

Hay que elegir esto **antes** de escribir una línea, porque cambia el formato de todo.

- **(A) Markdown en el repositorio** — `Documentación/manual/{es,en,pt-BR,pt-PT}.md`.
  Se versiona con git, se lee en GitHub, se difunde por lector de pantalla sin
  esfuerzo, y cada cambio se revisa como se revisa el código.
- **(B) Dentro de la aplicación** — una ventana de ayuda. Se lee sin salir del programa,
  que es donde está el lector justo cuando le surge la duda.
- **(C) PDF** — se imprime y se lleva al estudio, pero es el peor de mantener y su
  accesibilidad depende de cómo se genere.
- **(D) Web / GitHub Pages** — bonito y buscable, pero es otro sitio que mantener.

**Recomendación: empezar por (A), y escribirlo de forma que (B) no obligue a rehacer
nada.** Si el manual son archivos Markdown, meterlo luego en una ventana de ayuda es
renderizarlos —o precompilarlos a HTML en el build de Vite, que evita meter un intérprete
de Markdown en el frontend y respeta la regla 5—. Al revés no: si se escribe pegado a la
interfaz, sacarlo fuera obliga a reescribirlo.

**Lo que hay que decidir con el autor antes de empezar:** si el manual debe poder leerse
**sin internet**. Y aquí los cuatro públicos no responden igual: un streamer está
conectado por definición, pero un DJ en una sala y muchos estudios no. Si tiene que
leerse sin conexión, (D) queda descartado como sitio principal y (B) sube de prioridad.

---

## 3. El manual NO puede vivir en los `.json` de i18n

Hoy la traducción son 427 claves en 35 secciones por idioma, cuadradas en los cuatro. Ese
mecanismo es perfecto para etiquetas de botón y pésimo para prosa:

- El JSON no tiene saltos de línea: un capítulo entero sería una sola cadena ilegible.
- No hay títulos, listas ni énfasis.
- Un manual multiplicaría las claves por diez y haría inmanejable el archivo que de
  verdad importa, el de la interfaz.

**El manual necesita su propio mecanismo: un archivo por idioma.** Los `.json` siguen
siendo de la interfaz y nada más.

---

## 4. La regla de oro: el vocabulario de la traducción no es libre

Esto es lo más importante del documento y lo más fácil de estropear.

**El manual en inglés tiene que llamar a cada cosa exactamente como la llama `en.json`.**
Si la interfaz inglesa dice *Fixed Panel*, el manual no puede decir *side panel* aunque
suene mejor: el lector buscaría en la pantalla algo que no está. Lo mismo con los cuatro
idiomas.

**Por eso el primer paso de cada traducción no es traducir: es extraer el glosario de la
interfaz de ese idioma desde su `.json`.** Ese glosario manda sobre el buen gusto del
traductor.

**Y funciona en los dos sentidos:** si al traducir el manual un término de la interfaz
resulta que no se entiende, el fallo está en el `.json`, no en el manual. Escribir el
manual es la mejor auditoría que se le puede hacer a la redacción de la propia interfaz.
Cuando eso pase, se corrige el `.json` **y** el manual, en el mismo commit.

---

## 5. Qué significa "adaptativa y no literal"

El español es la fuente. Los otros tres **no se calcan: se reescriben para que se
entiendan**. En concreto:

**El término del oficio le gana al diccionario, pero tiene que servir a los cuatro
oficios.** "Botonera" traducida literalmente no es nada en inglés, así que hay que buscar
el término real; el cuidado está en cuál. *Soundboard* lo entienden los cuatro públicos.
*Cart wall* —de las viejas máquinas de cartuchos— es más exacto para la radio y no le dice
absolutamente nada a un streamer: sirve como aclaración dentro de un capítulo de radio,
no como nombre del programa. Un manual que diga *buttonboard* está escrito para nadie, y
uno que diga *cart wall* en la portada está escrito para la cuarta parte.

**Y cuando un término solo vive en un oficio, se explica en vez de disimularlo.**
"Locución" es palabra de radio: un DJ no la ha usado nunca. La interfaz ya la usa y el
manual está obligado a llamarla igual (sección 4), pero el capítulo puede abrir diciendo
qué es en lugar de darla por sabida.

**Los ejemplos se cambian, no se traducen.** El caso de las ciudades homónimas se explica
en español con Barcelona (Anzoátegui, Venezuela) frente a Barcelona (España), que a un
hispanohablante le dice todo. En inglés ese ejemplo no significa nada: hay que buscar un
homónimo que le duela al lector inglés (Birmingham de Inglaterra frente a Birmingham de
Alabama). **El punto que se explica es el mismo; el ejemplo, no.**

**Las unidades y los formatos siguen al lector.** Los grados Fahrenheit y el reloj de 12
horas son lo normal en inglés y lo raro en español. Los ejemplos del capítulo del clima
deberían reflejarlo.

---

## 6. `pt-BR` y `pt-PT` no son el mismo idioma con acento

Es el error clásico: traducir a portugués y duplicar el archivo cambiando cuatro
palabras. Los `.json` ya distinguen bien y ahí está el vocabulario establecido:

- *arquivo* (BR) frente a *ficheiro* (PT)
- *tela* (BR) frente a *ecrã* (PT)
- *usuário* (BR) frente a *utilizador* (PT)
- *Buscando…* (BR) frente a *A procurar…* (PT) — el gerundio portugués europeo

Y la radio brasileña y la portuguesa no hablan igual. Si hay dudas, se pregunta al autor
antes de inventar.

---

## 7. El problema de verdad: que los cuatro no se separen

Un manual traducido se pudre en cuanto el español avanza solo. Propuesta:

**Antes de la primera traducción**, el español se mueve libre. Traducir un texto que
todavía se está reescribiendo es tirar trabajo.

**Después**, la regla es la misma que ya rige la documentación: **el cambio en español y
sus tres adaptaciones van en el mismo commit.** Sin excepciones, igual que hoy no se
añade una clave a `es.json` sin añadirla a los otros tres.

**Y un seguro barato:** que cada archivo traducido lleve en su cabecera de qué revisión
del español salió, por ejemplo `<!-- adaptado-de: es.md @ a363422 -->`. Un script que
compare esa marca con `git log -1 --format=%h` del español canta el desfase en un
segundo, y puede vivir junto a la comprobación de claves de i18n.

---

## 8. Accesibilidad: no es un extra, es un requisito

El autor trabaja con **visión reducida y lector de pantalla**. Eso decide cómo se escribe:

- **Nada de tablas para contenido.** Un lector de pantalla las recorre celda a celda y se
  pierde el hilo. Lo que se explica, se explica en prosa.
- **Títulos de verdad y jerárquicos** (`##`, `###`): es como se navega un documento largo
  sin verlo.
- **Nada de arte ASCII ni diagramas de caracteres.** En `LIBRO_PROYECTO.md` se toleran
  porque su público lee código; en el manual de quien pincha, no.
- **Nada de "pulsa el botón de arriba a la derecha".** Se dice el **nombre** del botón,
  que es lo que el lector de pantalla anuncia.

---

## 9. La trampa de las capturas de pantalla

Parece obvio ilustrar un manual, y es una trampa cara: la interfaz está traducida a
cuatro idiomas y tiene tema claro y oscuro. **Eso son ocho versiones de cada captura**, y
cualquier retoque de la interfaz las invalida todas a la vez.

**Recomendación: escribir el manual sin depender de capturas.** Si alguna es
imprescindible, que sea en español, que se diga, y que el texto se entienda igual sin
verla — que es justo lo que necesita quien usa lector de pantalla. Lo que no se puede
explicar con palabras suele ser una señal de que **la interfaz** necesita el arreglo, no
el manual.

---

## 10. Índice propuesto

Por orden de utilidad para quien acaba de instalar:

1. Qué es y qué resuelve
2. La primera vez: el asistente de arranque
3. Perfiles, pestañas y botones
4. Asignar audios y dispararlos
5. Modos de reproducción: normal, loop, solapado, reinicio, y el Solo
6. Atajos de teclado, globales y por botón
7. El panel fijo
8. El reproductor auxiliar
9. La consola de audio: buses, máster y vúmetro
10. La pre-escucha
11. El editor de pistas: puntos de entrada y salida, ganancia y normalización
12. Locuciones de hora y clima
13. Salidas de audio y tarjetas
14. Precarga en RAM
15. Temas, idioma y tamaño de texto
16. Exportar, importar y convivir con LF Automatizador
17. Actualizaciones
18. Problemas frecuentes

**Ninguno de los cuatro públicos los necesita todos.** El 2, 3, 4, 5 y 6 los quiere
cualquiera. El 12 es radio pura. El 13 le importa sobre todo a quien emite por internet y
tiene que mandar el audio a un sitio distinto del que escucha. Que cada capítulo diga a
quién sirve (sección 1) vale más que reordenar la lista, porque el orden útil es distinto
para cada uno.

**El capítulo 12 sale casi gratis:** el capítulo 12 de `LIBRO_PROYECTO.md` ya tiene el
formato de nombres investigado contra las fuentes primarias (ZaraRadio, Salamandra,
RadioBOSS, y por qué Dinesat y Audicom no entran). Hay que reescribirlo para el operador
—sin `read_dir` ni prefijos—, pero el contenido está y está verificado.

---

## 11. Fases

1. **Decidir el sitio** (sección 2) y crear el esqueleto en español con los 18 títulos.
   Corto. Necesita al autor.
2. **Escribir el español entero**, capítulo a capítulo, empezando por el 1–4, que es lo
   que necesita quien abre la aplicación por primera vez. Es el grueso del trabajo, y da
   para varias sesiones.
3. **Congelar el español** y extraer el glosario de interfaz de cada idioma (sección 4).
4. **Adaptar a `en`, `pt-BR`, `pt-PT`**, una pasada completa por idioma —no capítulo a
   capítulo salteando idiomas, o el vocabulario se decide cuatro veces y sale distinto.
5. **Enganchar el seguro de desfase** (sección 7) y, si se eligió, la ventana de ayuda.

---

## 12. Qué no hacer

- **No traducir con el español a medias.** Fase 4 después de la 2, no en paralelo.
- **No copiar `LIBRO_PROYECTO.md` quitando código.** Es otro público y otro idioma; sale
  un manual que no ayuda a nadie.
- **No meter prosa en los `.json`** (sección 3).
- **No inventar términos de interfaz** que no estén en el `.json` de ese idioma
  (sección 4).
- **No documentar lo que no existe.** Si al escribir un capítulo aparece una función a
  medias, se anota y se pregunta; un manual que promete de más es peor que no tenerlo.
