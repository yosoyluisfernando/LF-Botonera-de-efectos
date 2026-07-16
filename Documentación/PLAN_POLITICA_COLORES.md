# Plan — Política de colores de los botones nuevos

Diseño **acordado con el autor y no implementado**. Se rescató de la conversación de diseño del
panel lateral (julio 2026) antes de borrar aquella exportación temporal, para no perder las
decisiones ya tomadas.

- **Estado:** diseño aprobado; implementación **no iniciada**.
- **Verificado en el código (2026-07-15):** no existe `new_button_style` ni equivalente. Hoy el
  color de todo botón nuevo sale de `domain::colors::random_color()`.

---

## 1. Qué queremos

Hoy, cada botón nuevo recibe un **color aleatorio**. Queremos poder elegir en Ajustes **cómo se
colorean los botones nuevos**, para poder darle a la botonera un aspecto ordenado (por ejemplo,
una pestaña entera de un color, o franjas por filas).

### Los cuatro modos acordados

- **Aleatorio** — comportamiento actual y **predeterminado**.
- **Color único** — todos los botones nuevos reciben el color elegido.
- **Por columnas** — cada columna usa un color: se ven franjas verticales.
- **Por filas** — cada fila usa un color: se ven franjas horizontales.

```
Por columnas                 Por filas

Rojo  Azul  Verde            Rojo  Rojo  Rojo
Rojo  Azul  Verde            Azul  Azul  Azul
Rojo  Azul  Verde            Verde Verde Verde
```

## 2. Decisiones firmes (ya acordadas, no volver a discutirlas)

1. **Nunca recolorea lo existente.** La política se aplica **solo al crear o reemplazar** un
   botón. Cambiar la preferencia no repinta la botonera.
2. **La edición manual siempre manda.** Cualquier botón puede cambiar su color desde su editor
   (clic derecho → editar), sin importar la política vigente.
3. **Se reutiliza la paleta de 32 colores que ya existe** (`get_color_palette`). No se crea otro
   selector.
4. **El color del texto se calcula solo** para que se lea sobre el fondo, y se puede cambiar a
   mano después. Ya existe: `domain::colors::text_for_theme`.
5. **La política es global**, no por perfil: si eliges "azul", todo botón nuevo será azul en
   cualquier perfil y pestaña.
6. **En los modos por filas y por columnas** se elige una secuencia ordenada de colores; cuando
   se agotan, se repiten desde el principio.
7. **El patrón se calcula por la posición real** del botón en la rejilla. Mover un botón no
   cambia su color; pero el próximo botón que ocupe esa celda recibe el color que le toca a esa
   fila o columna. Es coherente con la decisión 1.

## 3. Cómo lo haríamos

**Todo en Rust, en un solo sitio** (reglas 4 y 5). El diseño original hablaba de tres puntos de
llamada, pero al rescatarlo (2026-07-15) `random_color()` se llama ya desde **siete**, porque
entonces no existían ni el panel fijo ni el reproductor:

```
domain/export/lfa_format/paleta.rs:105   ipc/cmd_grid.rs:67, :80, :103
ipc/cmd_fixed_buttons.rs:22              ipc/cmd_player_file.rs:54
ipc/cmd_player_queue.rs:97
```

Eso refuerza la decisión: la política **no** puede ser una condición suelta repetida en siete
módulos (regla 2). Se resuelve **centralizada** en `domain/colors`, y cada punto sigue pidiendo
"dame el color que toca" sin saber qué política hay vigente. Ojo: no todos esos puntos tienen
rejilla — el reproductor y las listas importadas no tienen filas ni columnas, así que para ellos
`by_row`/`by_column` no aplican y deberían caer a `single`/`random` (ver dudas abiertas).

Modelo propuesto, colgado de `AppConfig` con `#[serde(default)]` (regla 6):

```text
new_button_style:
  mode: random | single | by_column | by_row   (default: random)
  colors: ["#...", ...]        # secuencia; en `single`, un solo color
```

La firma tendría que recibir la posición (`index`, `cols`) para poder resolver fila y columna.

**En Ajustes:** la sección natural es la pestaña **Panel fijo** o **Principal** (decidir al
implementar). i18n en los cuatro idiomas (regla 7).

## 4. Resultado esperado

- En Ajustes se elige entre Aleatorio, Color único, Por columnas y Por filas.
- Los botones nuevos siguen la política; los existentes **no se tocan**.
- Cualquier botón puede editarse a mano después.
- Verificación: `cargo test --lib` y `npm run build`; archivos bajo 200 líneas; i18n completo.

## 5. Dudas abiertas (a resolver antes de implementar)

- ¿La política aplica también a los **botones del panel fijo**, o solo a la rejilla? En el panel,
  "columna" y "fila" existen en modo `buttons`, pero no en modo `player`.
- ¿Qué hace `suggest_button_style` (que hoy sugiere colores a partir del archivo) cuando hay una
  política fija activa: se respeta la política o sigue sugiriendo?
