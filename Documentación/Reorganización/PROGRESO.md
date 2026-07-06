# Progreso de la Reorganización

> Última actualización: 2026-07-06

---

## Estado general

```
Fase 1 — Modelo de datos .... [x] Completada
Fase 2 — Motores ............ [x] Completada
Fase 3 — Dominio ............ [x] Completada
Fase 4 — Puerta IPC ........ [x] Completada
Fase 5 — Núcleo ............. [x] Completada
Fase 6 — Deduplicación ..... [x] Completada
Fase 7 — Frontend ........... [x] Completada
Fase 8 — Verificación final . [ ] Pendiente
```

---

## Decisiones tomadas

| # | Pregunta | Decisión | Fecha |
|---|----------|----------|-------|
| 1 | ¿Mantener `gainDb.js` en JS? | ✅ **A) Mantener en JS** con comentario de justificación. El slider necesita feedback instantáneo sin round-trip IPC. | 2026-07-05 |
| 2 | ¿Tipo de error unificado con `thiserror`? | ✅ **A) Sí.** Crear `AppError` con `thiserror`. Estándar profesional en Rust. | 2026-07-05 |
| 3 | ¿Renombrar archivos al moverlos? | ✅ **A) Sí, eliminar redundancia.** `audio_thread.rs` → `thread.rs` dentro de `audio/`. | 2026-07-05 |
| 4 | ¿Reorganizar frontend ahora o después? | ✅ **B) Fase separada.** Primero backend completo, luego frontend. | 2026-07-05 |
| 5 | ¿Algún motor adicional? | ✅ **Motor de Entrada (`engine/input/`).** Atajos de teclado ahora + soporte futuro para hardware (Stream Deck, botoneras físicas, teclados macro). | 2026-07-05 |

---

## Registro por fase

### Fase 1 — Modelo de datos
- **Estado:** ✅ Completada
- **Archivos movidos:** 10/10
- **Tests:** ✅ `cargo test --lib` — 55 passed, 0 failed, 1 ignored
- **Build:** ✅ `npm run build`
- **Commit:** `refactor: move data types to model directory`
- **Notas:** `tauri dev` no ejecutado: la fase solo reubica tipos de datos y actualiza imports; no cambia flujos runtime ni UI. Se verificó además que todos los archivos de `model/` quedan bajo 200 líneas.

### Fase 2 — Motores
- **Estado:** ✅ Completada
- **Archivos movidos:** 35/35 listados en `MAPA_MOVIMIENTOS.md`
- **Tests:** ✅ `cargo test --lib` — 55 passed, 0 failed, 1 ignored
- **Build:** ✅ `npm run build`
- **Commit:** `refactor: move subsystems to engine directory`
- **Notas:** `tauri dev` no ejecutado: la fase solo reubica motores y actualiza imports; no cambia contratos IPC, UI ni flujos funcionales. Se verificó que no quedan imports `crate::...` hacia módulos de motor antiguos y que los archivos de `engine/` quedan bajo 200 líneas.

### Fase 3 — Dominio
- **Estado:** ✅ Completada
- **Archivos movidos:** 18/18 listados en `MAPA_MOVIMIENTOS.md`
- **Tests:** ✅ `cargo test --lib` — 55 passed, 0 failed, 1 ignored
- **Build:** ✅ `npm run build`
- **Commit:** `refactor: move business logic to domain directory`
- **Notas:** `tauri dev` no ejecutado: la fase solo reubica lógica de dominio y actualiza imports; no cambia contratos IPC, UI ni flujos funcionales. Se verificó que no quedan archivos antiguos de dominio en `src-tauri/src/` y que ningún `.rs` supera 200 líneas.

### Fase 4 — Puerta IPC
- **Estado:** ✅ Completada
- **Archivos movidos:** 22/22 comandos IPC + `register_handlers.rs`
- **Splits:** 2/2 (`cmd_config.rs`, `cmd_norm.rs`)
- **Tests:** ✅ `cargo test --lib` — 55 passed, 0 failed, 1 ignored
- **Build:** ✅ `npm run build`
- **Commit:** `refactor: move IPC commands to ipc directory`
- **Notas:** `tauri dev` no ejecutado: la fase mantiene nombres y firmas IPC, y solo reubica comandos/splits documentados. Se verificó que no quedan `cmd_*.rs` ni `register_handlers.rs` en `src-tauri/src/`, y que ningún `.rs` supera 200 líneas.

### Fase 5 — Núcleo
- **Estado:** ✅ Completada
- **Archivos movidos/creados:** 4/4
- **Tests:** ✅ `cargo test --lib` — 55 passed, 0 failed, 1 ignored
- **Build:** ✅ `npm run build`
- **Commit:** `refactor: create core with AppState and setup`
- **Notas:** `tauri dev` no ejecutado: la fase reubica el estado y setup sin cambiar contratos IPC, UI ni flujos funcionales. Se verificó que `AppState` solo se define en `core/state.rs`, que `app_setup.rs` ya no queda en raíz y que ningún `.rs` supera 200 líneas.

### Fase 6 — Helpers y deduplicación
- **Estado:** ✅ Completada
- **Archivos nuevos:** 3/3 (`config_helpers.rs`, `actions.rs`, `clock.rs`)
- **Archivos actualizados:** 18/~18
- **Tests:** ✅ `cargo test --lib` — 55 passed, 0 failed, 1 ignored
- **Build:** ✅ `npm run build`
- **Commit:** `refactor: add helpers, centralize actions, deduplicate`
- **Notas:** `tauri dev` no ejecutado: la fase solo deduplica lógica Rust y conserva contratos IPC/UI. Se verificó que `probe_duration_secs` ya vive en `engine/audio/formats.rs`, que el reloj delega en `domain/clock.rs`, que los atajos usan acciones centralizadas y que los archivos revisados no superan 200 líneas. `wc`/`bash` no estaban disponibles en este entorno; el conteo se hizo con PowerShell como respaldo y `tracks.rs` queda exactamente en 200 líneas.

### Fase 7 — Frontend (fase separada)
- **Estado:** ✅ Completada
- **Archivos movidos:** 54/54
- **Tests:** ✅ `cargo test --lib` — 55 passed, 0 failed, 1 ignored
- **Build:** ✅ `npm run build`
- **Commit:** `refactor: reorganize frontend into bridge ui util directories`
- **Notas:** `tauri dev` no ejecutado: la fase solo reorganiza módulos JS y actualiza imports/rutas de entrada; no cambia contratos IPC ni lógica funcional. La nueva estructura queda en `src/js/bridge`, `src/js/ui` y `src/js/util`. Se verificó que no quedan archivos `.js` sueltos en `src/js`, que `src/index.html` apunta a `/js/ui/theme.js` y `/js/ui/main.js`, y que ningún módulo JS supera 200 líneas.

### Fase 8 — Verificación final
- **Estado:** ⬜ Pendiente
- **Checklist completado:** 0/13
- **Notas:** —

---

## Hallazgos durante el proceso

> Aquí se documentan bugs, mejoras, o problemas descubiertos durante la
> reorganización que NO se corrigen ahora (Regla 4) pero se anotan para
> atender después.

| # | Hallazgo | Archivo | Severidad | Ticket |
|---|----------|---------|-----------|--------|
| — | — | — | — | — |
