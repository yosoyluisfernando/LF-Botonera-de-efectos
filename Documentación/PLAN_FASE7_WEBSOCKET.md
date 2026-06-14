# PLAN FASE 7 — Enlace WebSocket Botonera ⇄ LF Automatizador v1.0

> Plan de trabajo aprobable ANTES de escribir código. Objetivo: la versión 1.0
> (Electron) que ya usan los testers. La v2 es un proyecto aislado y NO es
> referencia obligatoria; el protocolo se diseña para lo que la v1 puede hacer hoy.

---

## 1. Hallazgos del estudio del LFA v1.0

| Hecho | Implicación |
|---|---|
| El LFA v1.0 **no tiene ningún servidor WebSocket** | Hay que añadirle un módulo servidor (npm `ws` en el proceso main de Electron) |
| Toda la botonera del LFA vive en `backend/ipc/cartwall.js` + ventana cartwall | El servidor ws será un **puente fino**: traduce mensajes de red a los mismos canales IPC que ya existen (no se duplica lógica) |
| El cartwall del LFA ya normaliza perfiles (`normalizeProfile`) | La sincronización de perfiles por red reutiliza `lfa_format.rs` (ya 100% compatible) |
| Wizard de la botonera ya pregunta por el enlace (`lf_automatizador_link`) | El cliente solo se activa si el usuario lo pidió |

## 2. Arquitectura (decisión)

```
┌─────────────────────────────┐      WebSocket (LAN)      ┌──────────────────────┐
│   LF AUTOMATIZADOR v1.0     │   ws://host:8390          │     LF BOTONERA      │
│  main: cartwall_ws.js (ws)  │ ◄── commands (JSON) ────  │  Rust: ws_client.rs  │
│   └─► IPC existente         │ ─── events (push) ──────► │  (hilo + reconexión) │
│       (cartwall.js)         │                           │                      │
└─────────────────────────────┘                           └──────────────────────┘
```

- **El LFA es el servidor** (es quien está al aire); la botonera es un control
  remoto que puede aparecer/desaparecer sin afectar la emisión.
- **Puerto por defecto 8390** (configurable en ambos lados). Host configurable;
  default `127.0.0.1` (misma máquina, el caso típico de cabina).
- La botonera opera en dos modos, decididos por su motor Rust (Regla 4):
  - **LOCAL** (hoy): reproduce con rodio.
  - **ENLACE**: cada pulsación envía `play` al LFA; el sonido sale por las
    tarjetas/buses del LFA y la UI se pinta con los eventos que el LFA emite.
  - **Degradación automática**: si el enlace cae, vuelve a LOCAL sin
    intervención; la pulsación en curso se ejecuta local (nunca se pierde).

## 3. Protocolo (propio, mínimo y versionado)

Diseñado para la v1; nombres simples. Sobre único en el cable:
```json
{ "v": 1, "kind": "command|event|reply", "name": "play",
  "request_id": "opcional", "payload": { } }
```

Comandos que ENVÍA la botonera (el puente los traduce al cartwall del LFA):
- `hello`        `{ token, app: "lf-botonera", version }` → respuesta con versión del LFA (apretón de manos)
- `play`         `{ profile_id, palette_index, button_index, options { loop, overlap, stop_other, restart } }`
- `stop`         `{ button_index, palette_index }`
- `stop_all`     `{}`
- `get_profiles` `{}` → el LFA devuelve sus perfiles (formato .bdeplf, el de `lfa_format.rs`)
- `save_profiles``{ profiles }` → empuja perfiles de la botonera al LFA

Eventos que RECIBE la botonera (push, sin polling):
- `play_state` `{ button_index, palette_index, state: "playing|stopped", position_ms, duration_ms }` → mismo pintor que `audio-tick`
- `ui_state`   `{ active_profile_id, active_tab_index }` → sincronizar pantallas
- `error`      `{ code }` → traducido en la UI (Regla 6, códigos no frases)

## 4. Seguridad y resiliencia (lo profesional, no lo fácil)

1. **Emparejamiento por token**: el LFA muestra un código de 6 dígitos en sus
   ajustes; la botonera lo pide una vez y lo persiste. `hello` sin token válido
   → desconexión. Sin esto, cualquier equipo de la LAN controlaría la radio al aire.
2. **Apretón de manos versionado** (`v: 1`): si no coincide, la botonera
   muestra "versiones incompatibles" — nunca degrada en silencio.
3. **Reconexión con backoff exponencial**: 1s → 2s → 5s → 15s → 30s (tope),
   ping/pong cada 5s para detectar enlaces medio-muertos.
4. **Máquina de estados explícita** en Rust: `Desactivado / Conectando /
   Enlazado / Degradado` → evento `link-status` a la UI (indicador 🔗/🔌/⚠ en
   la barra de título).
5. **Idempotencia**: repetir `play`/`stop` no corrompe estado (igual que en
   nuestro motor local).
6. **Cero bloqueo del audio local**: el cliente ws vive en su propio hilo
   (como `audio_monitor`); si la red se congela, la botonera local ni se entera.
7. **Aislamiento del protocolo**: el sobre y los nombres viven en un solo
   módulo por lado (`ws_protocol.rs` / `cartwall_ws.js`). Si algún día la v2
   ofrece su propio servidor, adaptar la botonera es tocar UN módulo.

## 5. Plan de trabajo por etapas (cada una compilable y probable)

| Etapa | Entregable | Dónde |
|---|---|---|
| **7.a Contrato congelado** | Este documento (§3) revisado y aprobado por ti | Documentación |
| **7.b Servidor puente en LFA v1** | `backend/ipc/cartwall_ws.js` (npm `ws`): traduce ws ⇄ IPC existente, token en ajustes del LFA | Repo del LFA |
| **7.c Cliente Rust** | `ws_client.rs` + `ws_protocol.rs` (crate `tungstenite`, hilo dedicado) + máquina de estados + indicador en barra de título | Botonera |
| **7.d Delegación de reproducción** | El motor Rust decide LOCAL/ENLACE por pulsación; pintado por `play_state` | Botonera |
| **7.e Sincronización de perfiles** | `get_profiles`/`save_profiles` ⇄ `lfa_format.rs` (botones "Traer del LFA / Enviar al LFA" en Ajustes) | Ambos |
| **7.f Resiliencia y pruebas LAN** | Matriz: caída del LFA en vivo, red lenta, token inválido, versión vieja, 2 botoneras a la vez | Ambos |

Dependencia Rust nueva: `tungstenite` (cliente ws síncrono, encaja con nuestra
arquitectura de hilos; sin runtime async adicional).

## 6. Lo que NO haremos (anti-patrones descartados)

- ❌ Polling de estado: todo es push del LFA.
- ❌ Servidor en la botonera: la fuente de verdad al aire es el LFA.
- ❌ Duplicar lógica del cartwall en el puente: solo traduce a IPC existente.
- ❌ Conexión sin autenticación "porque es LAN".
- ❌ Atarse al contrato de la v2 (proyecto aislado); solo se aísla el protocolo
  en un módulo para que un futuro cambio sea barato.

---
**Estado:** PENDIENTE DE APROBACIÓN. No se escribe código de Fase 7 hasta que
confirmes: puerto (8390), token de 6 dígitos, nombres del protocolo (§3) y el
orden de etapas (7.a–7.f).
