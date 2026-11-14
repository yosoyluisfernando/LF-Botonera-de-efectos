# Integración de aportaciones de Danilo Wenner

## Estado y alcance — 2026-11-13

Primera fase vigente: rueda, barra inferior y reproducción nativa de audio
compatible, incluido MP4. El autor retiró la conversión externa el 2026-11-13.
Buscador y editor unificado de colores integrados por autorización individual;
las salidas por pestaña siguen pendientes. No se compila ni publica en GitHub.
El autor mantiene seis aportaciones seleccionadas de LF Botonera Radio. La integración parte
de la rama de desarrollo oficial 1.6.0, no de sustituirla por el fork 1.5.1.
El autor aprobó explícitamente la primera fase; las fases restantes requieren
revisión individual antes de modificar módulos, modelo o IPC.

## Entorno aislado

- Rama local y GitHub: `variante-danilo`.
- Base: `3502072a5ff9ba3993bd4ef7ef63044f85e47114` de `codex/desarrollo`.
- Worktree: `C:/OVERLAY/BOTONERA/.worktrees/variante-danilo`.
- Referencia descargada: `C:/OVERLAY/BOTONERA/.references/danilo`.
- Las dos carpetas están excluidas localmente del repositorio principal.
- La carpeta principal conserva `codex/desarrollo` y sus cambios sin commit.
  Esos cambios NO están incluidos en esta base; conciliarlos antes de integrar
  el resultado o preparar una entrega, sin sobrescribirlos ni descartarlos.
- No modificar main, publicar releases ni cambiar versiones durante la preparación.
- D: queda para experimentos reversibles; el trabajo de integración reside en C:.

## Procedencia y crédito permanente

Autor de las aportaciones: **Danilo Wenner** (GitHub: `danilo-wenner-1`).
Repositorio: https://github.com/danilo-wenner-1/LF-Botonera-de-efectos
Variante: LF Botonera Radio 1.5.1, bajo GPL-3.0-or-later.
Commit funcional: `7c1f1ae484cdb4bd0110b4761b8a221023bdea6f`.
Referencia descargada: `97d935c9eb145eb9ac9c25eef61b244ec2504fac`.

El commit funcional agrupa más funciones que las seleccionadas: NO fusionarlo
completo. Extraer por función conservando avisos y una referencia al commit
original. Preservar su autoría en los cambios importados; registrar por separado
las adaptaciones del mantenedor, con identidad humana real. No atribuir a Danilo
código nuevo que no escribió. No reutilizar su nombre/ID de aplicación ni su
actualizador: conservar la identidad oficial y los canales PC/Android vigentes.

Antes de publicar, revisar los créditos de CHANGELOG, README y avisos legales;
las notas deben nombrar las funciones realmente incorporadas y enlazar el fork.
No presentar este plan como funciones ya disponibles.

## Funciones seleccionadas y propuesta de adaptación

### 1. Rueda sobre la barra de pestañas

Arriba: pestaña anterior; abajo: siguiente, mediante el comando Rust existente
`cycle_paleta`. JS solo interpreta el gesto y refresca la vista. Mantener el
scroll horizontal y evitar saltos excesivos de trackpad. Fuera de la barra,
la rueda mantiene su comportamiento actual. Sin nuevos campos ni IPC.

### 2. Colores HEX y degradados — implementados para prueba del autor

Selector único: Predefinidos primero, sólido y cuatro degradados. Vista previa
inmediata compartida con la rejilla y el panel fijo. Ver EDITOR_COLORES.md.
La revisión funcional y los ajustes posteriores corresponden a las pruebas del autor. Preservar colores anteriores, temas y selección
múltiple. Proponer campos opcionales con `serde(default)` y validación en Rust;
CSS/JS solo representan y editan visualmente. Comprobar exportación/importación
con LF Automatizador y los cuatro idiomas. La antigua política automática de
colores descartada no forma parte de este trabajo. Tamaño de fuente por botón
no está seleccionado en esta primera fase.

### 3. Salida de audio por pestaña integrada en consola

Aprovechar la intención del ruteo, no copiar el bypass del fork. La consola
oficial debe seguir siendo dueña de las tarjetas y de sus buses; una tarjeta
física no debe abrirse varias veces. La propuesta detallada debe explicar cómo
se aplican máster, Mic, faders y medidores a los destinos independientes y cómo
se representan en la consola, sin doble ganancia ni mezcla física ficticia.
Mantener CUE separado; definir explícitamente reconexión y destino alternativo
si falta una tarjeta antes de programar. La salida propia no puede quedar fuera
del máster/Mic/medición como ocurre en el fork. Revisar Windows y Linux; no
habilitar este ruteo en Android sin comprobar su backend.

### 4. Buscador integrado — implementado por autorización del autor

Clic izquierdo: buscador de botones de Danilo, conservando su funcionamiento.
Clic derecho: atajo al Buscador existente del panel fijo. Abre el panel si está
oculto, cambia desde Reproductor/Botones fijos y permanece abierto si ya muestra
Buscador. No abre otra ventana ni duplica el catálogo. Conserva las preferencias
del panel y no toca audio. Ver BUSCADOR_BOTONES.md para procedencia y pruebas.

### 5. Conversión automática — retirada por decisión del autor

No implementar ni restaurar FFmpeg como convertidor. Retirados sus módulos,
procesos, caché, IPC, eventos, modal, indicadores, scripts y binario local.
La decisión sustituye la autorización anterior de integrar la conversión.
Se conserva la apertura nativa segura del audio compatible y el detector AAC,
con crédito de Danilo. No se alteran archivos de audio ni datos del usuario.
Los perfiles no admitidos devuelven formato no compatible; no hay fallback externo.

### 6. Barra inferior ocultable

Añadir una preferencia persistente, visible por defecto para configuraciones
anteriores. Mantener un acceso claro en Ajustes para volver a mostrarla y no
alterar atajos ni audio. JS/CSS solo aplican visibilidad; Rust valida/persiste
la preferencia. Revisar escritorio y Android antes de definir el alcance final.

### 7. Acceso contextual al atajo de pestaña — añadido por petición del autor

Conservado el acceso original de Danilo: menú de pestaña → Asignar atajo de
teclado… abre captureTab sobre la pestaña pulsada, sin activar otra pestaña ni
reproducir botones. La captura/limpieza/conflictos de teclado siguen igual.
Adaptación: MIDI en details plegado, visible solo con MIDI habilitado y entrada
conectada, usando get_shortcut_editor_state. Desconexión/cierre cancelan captura
sin borrar asignaciones. No hay cambios de modelo ni comandos nuevos.
Ver ATAJOS_PESTANA_Y_CONSOLA.md. Corrección de foco de consola independiente.

## Orden propuesto y comprobaciones

1. Primera fase vigente: rueda, barra inferior y reproducción nativa compatible.
2. Buscador y editor de colores integrados; pendientes pruebas del autor.
3. Ruteo integrado en consola, con diseño de buses y reconexión aprobado.
4. No distribuir ni reintroducir el convertidor retirado.

Trabajar en cambios pequeños por función, con procedencia y créditos trazables.
Mantener archivos de máximo 200 líneas y traducciones es/en/pt-BR/pt-PT.
Al modificar código: `cargo test --lib`, `cargo build --lib` y `npm run build`,
más pruebas dirigidas cuando verifiquen comportamiento real. No lanzar la app;
la validación funcional y de tarjetas la realiza el autor con su equipo.
Verificaciones locales de biblioteca y frontend, sin abrir la app ni crear
instaladores. El autor prueba con el DEV.bat del worktree aislado.

## Encontrar las opciones en el instalador de Danilo

- Barra: Ajustes → Principal → «Mostrar la barra inferior (reloj, modos, Mic,
  volumen, Detener todo)»; desmarcar para ocultarla.
- La conversión de la variante original de Danilo no se conserva en la botonera
  oficial. Las opciones anteriores se mantienen solo como contexto del fork.
- La prueba funcional con DEV.bat queda a cargo del autor.
