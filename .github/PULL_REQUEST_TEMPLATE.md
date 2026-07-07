## ¿Qué hace este PR? / What does this PR do?

Una breve descripción de los cambios.
*A brief description of the changes.*

## Referencias / References

Si esto resuelve un issue abierto, enlázalo aquí (ej: `Fixes #123`).
*If this resolves an open issue, link it here.*

## Lista de verificación / Checklist

Por favor, confirma que has cumplido con las reglas del proyecto marcando con una `[x]`:
*Please confirm that you have followed the project rules by checking with an `[x]`*:

- [ ] **Regla 3:** Ninguno de los archivos creados o modificados en Rust o JS supera las 200 líneas (medido con `wc -l`). / *No file exceeds 200 lines.*
- [ ] **Regla 4:** La lógica de negocio está en Rust, no en JS. / *Business logic is in Rust, not JS.*
- [ ] **Regla 7:** No he añadido strings hardcodeados en el código. He utilizado claves de i18n (`t(key)`) y añadido las claves en `es.json`, `en.json`, `pt-BR.json` y `pt-PT.json`. / *No hardcoded UI strings, used i18n instead.*
- [ ] **Regla 10 (Backend):** Ejecuté y pasaron todos los tests unitarios (`cd src-tauri && cargo test --lib`). / *Ran and passed Rust unit tests.*
- [ ] **Regla 10 (Frontend):** Compilé el frontend sin errores (`npm run build`). / *Frontend builds without errors.*
- [ ] **Regla 12:** Si añadí dependencias, justifiqué necesidad, licencia, mantenimiento, impacto y seguridad. / *New dependencies are justified.*
- [ ] **Regla 13:** Actualicé la documentación relacionada si cambié arquitectura, IPC, modelo de datos, formatos o flujos importantes. / *Related docs are updated.*
- [ ] **Regla 14:** No dejé archivos temporales, planes completados ni artefactos generados innecesarios. / *Working tree contains no temporary leftovers.*
- [ ] El código está alineado con la arquitectura de "Núcleo + Motores" explicada en `ARCHITECTURE.md`. / *Code aligns with the Core+Engines architecture.*

## Notas adicionales / Additional notes

Cualquier nota extra para el revisor (ej: "Hay que probar X comportamiento específico").
*Any extra notes for the reviewer.*
