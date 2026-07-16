# Continuidad de sesión — dónde estamos y cómo trabajamos

Punto de entrada para retomar el trabajo cuando se pierda el hilo de la conversación
(compactación, sesión nueva, otro agente). **No repite** lo que ya está documentado: apunta a
dónde está y guarda lo que solo vive en la conversación.

**Léelo en este orden:**

1. [`REGLAS_PROYECTO.md`](REGLAS_PROYECTO.md) — las 14 reglas. **Son ley.**
2. Este archivo — estado, acuerdos y trampas.
3. [`PLAN_MODO_REPRODUCTOR.md`](PLAN_MODO_REPRODUCTOR.md) — el registro de avance fase a fase.
4. [`../CLAUDE.md`](../CLAUDE.md) y [`../AGENTS.md`](../AGENTS.md) — mapa técnico y comandos IPC.
5. [`GLOSARIO.md`](GLOSARIO.md) — cada término, con sus trampas. Búscalo aquí **antes** de
   deducirlo del código.

---

## 1. Estado (2026-07-16)

- **Rama:** `codex/panel-lateral-fijo`. Último commit: `d7947df` "Añade el modo reproductor al
  panel lateral" (105 archivos).
- **Sin commitear:** el arreglo del salto de posición (`seek_source.rs`) y la retirada del modo
  `manual`. Ambos verificados.
- **Verificación:** 121 pruebas, `cargo build --lib` sin avisos, `npm run build` correcto,
  ningún archivo sobre 200 líneas.
- **El modo reproductor está completo.** Lo que queda son mejoras, no deudas: ver §5.

---

## 2. Cómo trabaja el autor (esto no está en ninguna regla)

- **Conversar antes de codificar.** En cualquier duda de arquitectura, preguntar. Él responde y
  decide; varias veces ha corregido el rumbo antes de escribir una línea.
- **Preguntar cuando el enunciado tenga dos lecturas.** Prefiere aclarar a que se adivine.
- **Trocear.** Fases verificables, no tandas grandes. Él prueba cada una en su equipo.
- **Accesibilidad:** visión baja y lector de pantalla. Responder en **prosa limpia, sin tablas**,
  con lo importante primero.
- **Al final de cada respuesta: qué se hizo y qué sigue.**
- **Su consola es una fuente de verdad.** Cuando pega la salida de `tauri dev`, ahí salen avisos
  que las comprobaciones del agente no ven. Así se descubrió que `cargo test --lib` no basta.
- **Los errores propios se dicen.** Ha agradecido cada vez que se le ha señalado un fallo del
  agente en vez de taparlo.

---

## 3. Decisiones firmes (NO volver a discutirlas)

Del reproductor (el detalle y el porqué, en `PLAN_MODO_REPRODUCTOR.md` §2):

- Motor propio, cola propia, **un solo** reproductor, alcance global, arranque detenido.
- **Marcar siguiente es LEY** y sigue a *su canción* (por `id`), no a la posición.
- **Editar la lista nunca corta la música** (pista huérfana).
- **Tres modos**: normal, repetir, aleatorio. `manual` se quitó: duplicaba "detener al
  finalizar" *y* limitaba (forzaba el orden normal, así que no convivía con aleatorio).
- **Loop** (🔂) repite *una canción*; **modo repetir** (∞) repite *la lista*. Iconos distintos a
  propósito.
- **No se persisten**: el Loop ni "detener al finalizar". Sí: modo, volumen, salida, contador.
- **Salida propia del panel fijo: DESCARTADA** (2026-07-16). No volver a proponerla.
- Sin fundidos ni segundo reproductor por ahora. Sin emisoras por URL.
- Iconos: transporte en símbolos monocromos (▶ ⏸ ■ ⏭ ▶■ → ∞ ⇄); Limpiar 🧹, Abrir 📂,
  Guardar 💾 y volumen 🔊 en emoji, para que destaquen. Orden: Limpiar, Abrir, Guardar, Modo.
- **Columnas y Filas** no son del reproductor: son la capacidad de la rejilla de botones fijos.
  No se borran; se ocultan en modo reproductor.

---

## 4. Trampas que ya costaron caro (verificadas, no suposiciones)

- **`cargo test --lib` NO basta.** Compila con las pruebas, y un `use super::*` puede mantener
  vivo un import muerto. El usuario compila **sin** pruebas. Correr **siempre** también
  `cargo build --lib`.
- **`audio-tick` no se emite en reposo.** Por eso el reproductor tiene su propio `player-tick`.
- **El salto de posición nunca funcionó** hasta `seek_source.rs`: rodio no informa a symphonia
  del tamaño del archivo. No se notaba porque los efectos están en la caché de RAM.
- **El `audio_out` de las paletas no enruta nada.** Solo existe por compatibilidad con el LFA.
- **Las locuciones del LFA no traen carpeta**, traen un *marcador* (`time_locution`): cada app
  resuelve con las suyas. Eso ES la compatibilidad.
- **La duración en `.LFPlay`** viene como número o como cadena según la versión del LFA.
- **Reaplicar el dispositivo del reproductor corta la música**: solo enviarlo si cambió.
- **No editar los i18n con scripts que reserialicen**: reformatean los 4 archivos enteros. Editar
  el texto directamente. Los cuatro deben cuadrar en número de claves.
- **`player_set_volume` persiste**: al arrastrar un deslizador, `persist: false`.

---

## 5. Qué queda (nada es obligatorio)

1. **En curso** (lo pedido el 2026-07-16, ver §6).
2. ~~Política de colores~~ — **DESCARTADA** (2026-07-16): el autor la vio complicada de explicar
   y de usar. En su lugar hay **selección múltiple** (Ctrl+clic + clic derecho → pintar). No
   volver a proponerla; `PLAN_POLITICA_COLORES.md` se conserva solo como registro.
3. Mejoras futuras del reproductor: fundidos/crossfade entre pistas; un segundo reproductor.
4. Deuda menor: `master_volume` es `f32` y su JSON crece solo (`0.45` → `0.4499999…`).
5. Prueba física en Linux (`.deb`, `.AppImage`).
6. `LF_Botonera_conversacion_botones_fijos_TEMPORAL.md`: ya rescatado a documentación; se puede
   borrar cuando el autor lo confirme. **`Prueba.LFPlay` se conserva** (decisión del autor).

---

## 6. Hecho el 2026-07-16 (última tanda)

**A — Gris de "detener al finalizar".** Con el interruptor activo Y algo sonando, lo marcado se
pinta **gris** (`--player-held-bg`, `#3b3f46`) en vez de naranja: se respeta, pero avisa de que
no sonará solo. **Parado sigue naranja**, porque no hay una "actual" tras la que sonar y ahí el
naranja es la guía de por dónde se retomaría (duda resuelta en el código, no suposición). El LFA
ya tenía este concepto y este color: `.row-manual-next`.

**B — Carpetas y multiselección, solo en el reproductor.** `cmd_player_drop.rs`:
`player_scan_drop` (cuenta y **Rust decide** si preguntar) y `player_add_drop` (añade en
`spawn_blocking` por lotes de 20, emitiendo `player-drop-progress`; la lista crece a la vista y
una sola escritura a disco al final). `audio_files_recursive` en `formats.rs` recorre subcarpetas
con pila explícita (nada de recursión: un árbol hondo desbordaría). La botonera **no cambió**.

**Aviso a partir de 250** (`LARGE_FOLDER_THRESHOLD`), con check "recordar siempre"
(`appConfirmRemember`, nuevo en el diálogo compartido) y tres estados en
`PlayerConfig.large_folder_action`: `ask` (por defecto) / `always` / `never`. **Se puede cambiar
en Ajustes → Panel fijo** (`player_set_large_folder_action`), que era el requisito: poder
rectificar si se respondió mal.

Verificación: 124 pruebas, `cargo build --lib` sin avisos, 400 claves i18n cuadradas.
