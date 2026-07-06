# Progreso de la Reorganización

> Última actualización: 2026-07-06

---

## Estado general

```
Fase 1 — Modelo de datos .... [x] Completada
Fase 2 — Motores ............ [ ] Pendiente
Fase 3 — Dominio ............ [ ] Pendiente
Fase 4 — Puerta IPC ........ [ ] Pendiente
Fase 5 — Núcleo ............. [ ] Pendiente
Fase 6 — Deduplicación ..... [ ] Pendiente
Fase 7 — Frontend ........... [ ] Pendiente (fase separada)
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
- **Estado:** ⬜ Pendiente
- **Archivos movidos:** 0/37 (33 originales + 4 del motor de entrada)
- **Tests:** —
- **Build:** —
- **Commit:** —
- **Notas:** —

### Fase 3 — Dominio
- **Estado:** ⬜ Pendiente
- **Archivos movidos:** 0/14 (reducido: atajos se movieron a engine/input/)
- **Tests:** —
- **Build:** —
- **Commit:** —
- **Notas:** —

### Fase 4 — Puerta IPC
- **Estado:** ⬜ Pendiente
- **Archivos movidos:** 0/22
- **Splits:** 0/2
- **Tests:** —
- **Build:** —
- **Commit:** —
- **Notas:** —

### Fase 5 — Núcleo
- **Estado:** ⬜ Pendiente
- **Archivos movidos/creados:** 0/4
- **Tests:** —
- **Build:** —
- **Commit:** —
- **Notas:** —

### Fase 6 — Helpers y deduplicación
- **Estado:** ⬜ Pendiente
- **Archivos nuevos:** 0/3
- **Archivos actualizados:** 0/~18
- **Tests:** —
- **Build:** —
- **Commit:** —
- **Notas:** —

### Fase 7 — Frontend (fase separada)
- **Estado:** ⬜ Pendiente
- **Archivos movidos:** 0/54
- **Build:** —
- **Commit:** —
- **Notas:** —

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
