# Reglas de la Reorganización

Estas reglas son **inquebrantables** durante todo el proceso. Aplican a cada
fase, cada commit, cada movimiento de archivo.

---

## 1. Mover, no reescribir

El contenido interno de un archivo **no se modifica** al moverlo de ubicación.
Lo único que cambia son:
- Las líneas `use crate::...` (rutas de importación)
- Las declaraciones `mod` en `mod.rs` y `lib.rs`
- Los `pub` necesarios para la nueva visibilidad entre módulos

Si un archivo funciona antes del movimiento, debe funcionar idénticamente después.

## 2. Una fase, un commit

Cada fase produce **un commit limpio** en la rama `refactor/architecture`.
No se mezclan fases en un solo commit. Esto permite:
- Revertir una fase específica si algo sale mal
- Bisectar problemas con `git bisect`
- Entender el historial: "este commit movió el modelo de datos"

## 3. Tests después de cada fase

Antes de hacer commit de una fase, **obligatorio**:
```bash
cd src-tauri && cargo test --lib
cd .. && npm run build
```

Si algo falla, se corrige **antes** de pasar a la siguiente fase.
No se arrastra deuda entre fases.

## 4. Sin cambios de lógica oportunistas

Si durante el movimiento se descubre un bug o una mejora posible,
**se documenta** en `PROGRESO.md` pero **no se corrige ahora**.
La reorganización y las mejoras lógicas son operaciones distintas.

**Excepción:** Las 6 mejoras explícitamente aprobadas (helpers, deduplicación,
errores, split de cmd_profiles, clock, probe_duration) sí se hacen, pero
en sus fases correspondientes (5 y 6), nunca durante las fases de movimiento
(1-4).

## 5. git mv, no copiar+borrar

Siempre usar `git mv` para que git preserve el historial del archivo.
Un `git log --follow engine/audio/thread.rs` debe mostrar todo el historial
previo como `audio_thread.rs`.

## 6. Compilar = verde

En ningún momento de ninguna fase el proyecto debe quedar en estado
"no compila". Si una fase requiere mover 13 archivos y actualizar 40 imports,
**todos los 13 movimientos y 40 actualizaciones** se hacen en la misma
operación atómica. No hay estados intermedios rotos.

## 7. El frontend espera

La reorganización del frontend (`src/js/` → `bridge/`, `ui/`, `util/`)
es **fase separada y opcional**. No se hace hasta que el backend esté
completamente reorganizado y verificado.

## 8. Documentar cada movimiento

La tabla de `MAPA_MOVIMIENTOS.md` es la fuente de verdad de qué archivo
va a dónde. Si hay un cambio de plan, primero se actualiza el mapa,
luego se ejecuta.

## 9. Respetar las reglas del proyecto

Las reglas de `AGENTS.md` y `REGLAS_PROYECTO.md` siguen vigentes:
- Máximo 200 líneas por archivo
- `#[serde(default)]` en campos nuevos
- i18n en 4 idiomas para texto nuevo
- `cargo test --lib` + `npm run build` para verificar
- Sin atribuciones de IA en commits

## 10. La IA hace, el humano aprueba

La IA ejecuta cada fase completa (mover archivos, actualizar imports,
crear mod.rs, verificar compilación). El humano revisa el resultado
y da el visto bueno antes de pasar a la siguiente fase. La IA no
avanza sin aprobación explícita entre fases.
