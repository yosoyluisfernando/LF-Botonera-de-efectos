# Reorganización Arquitectónica — Núcleo + Motores

> **Estado:** 🟡 En planificación  
> **Inicio:** 2026-07-05  
> **Rama:** `refactor/architecture` (por crear)  
> **Versión base:** 1.1.3

## Qué es esto

Este directorio contiene toda la documentación del proceso de reorganización
arquitectónica de LF Botonera de Efectos. El objetivo es pasar de una estructura
plana (~88 archivos `.rs` sueltos en un directorio y ~54 `.js` sueltos en otro)
a una arquitectura profesional de **Núcleo + Motores** donde cada subsistema
tiene su propia carpeta y sus responsabilidades están claramente delimitadas.

## Documentos

| Archivo | Contenido |
|---------|-----------|
| [VISION.md](VISION.md) | La visión arquitectónica: qué queremos lograr y por qué |
| [REGLAS.md](REGLAS.md) | Reglas inquebrantables durante la reorganización |
| [FASES.md](FASES.md) | Las fases de trabajo con checklist detallado |
| [MAPA_MOVIMIENTOS.md](MAPA_MOVIMIENTOS.md) | Tabla exacta: archivo origen → destino |
| [PROGRESO.md](PROGRESO.md) | Estado actual de cada fase (se actualiza en vivo) |

## Principio fundamental

> **Se mueve, no se reescribe.**
>
> El 85% del trabajo es mover archivos a carpetas y actualizar rutas de import.
> La lógica interna de cada archivo NO se toca salvo para las ~6 mejoras
> específicas documentadas (helpers, deduplicación, errores).
> Si algo funciona hoy, debe seguir funcionando igual después del movimiento.

## Cómo verificamos

Después de **cada fase**:
```bash
cd src-tauri && cargo test --lib    # 39+ tests pasan
cd .. && npm run build              # Frontend compila sin errores
```

La prueba funcional completa (app corriendo, reproduciendo audio, editor de
pistas, atajos, etc.) la hace el usuario al final de todo el proceso.

Además de compilar, la IA debe hacer pruebas razonables de regresión con las
herramientas disponibles. Si una fase toca comportamiento que solo puede
validarse con la aplicación en ejecución, se intentará `tauri dev`; si no es
posible completar esa prueba en el entorno de trabajo, se documentará el motivo
en `PROGRESO.md` y quedará pendiente para la prueba funcional del usuario.
