# Mic — estudio y propuesta de adaptación

**Fecha:** 2026-09-17. **Estado:** plan aprobado e implementado localmente;
pendiente de prueba funcional del autor. Nivel absoluto aprobado con el plan.
El autor mantiene la instrucción de no publicar hasta que lo solicite.

## 1. Alcance solicitado

Botón con icono de micrófono y etiqueta traducible `Mic`, con tres modos:

- Mantener pulsado: ratón o atajo; atenúa mientras se mantiene y restaura al soltar.
- Alternar: primera pulsación atenúa; la siguiente restaura.
- Automático: detecta señal del micrófono seleccionado y atenúa durante la voz.

Destinos combinables mediante casillas: botonera principal, botones del panel fijo
y reproductor auxiliar. Seleccionar todos equivale a marcar las tres casillas.
El volumen previo se conserva, incluso cuando no era 100%.

## 2. Referencia examinada

Repositorio: [LF Automatizador](https://github.com/yosoyluisfernando/lf-automatizador).
Commit estudiado: `46730b507ee98a76ab9a828a7e87484ed218be2c` (rama `main`).
La revisión utilizó archivos concretos del árbol principal, descargados fuera de
la Botonera. No se estudió ni se incorporó código de una carpeta de versión 2;
no aparece una carpeta con ese nombre en el árbol de esta revisión.

Fuente principal:
[frontend/render.js](https://github.com/yosoyluisfernando/lf-automatizador/blob/46730b507ee98a76ab9a828a7e87484ed218be2c/frontend/render.js#L16349).

- Líneas 16253–16300: atajo `app.talk`, manejadores de pulsación y liberación.
- Líneas 16349–16423: `activateTalk`, `deactivateTalk`, `toggleTalk` y transición.
- Líneas 16425–16576: modos hold/toggle, ratón/táctil y ajustes.
- Líneas 16579–16723: captura con `getUserMedia`, nivel RMS convertido a dB,
  umbral, tiempos de señal/silencio, medidor y selección de entrada.

Aunque el motor del Automatizador usa Rust, esta función concreta está en JS.
Mueve el deslizador master mediante `requestAnimationFrame`, guarda su valor
anterior y lo recupera. El objetivo es un porcentaje absoluto (20 por defecto),
con transición de 500 ms; detección automática desactivada por defecto, umbral
-30 dB, confirmación de 200 ms y espera de silencio de 1000 ms.

La detección es de nivel, no un clasificador de voz: otros sonidos fuertes en la
entrada también pueden dispararla. La entrada analizada no se conecta a una salida.
La modalidad automática es una opción adicional a hold/toggle en la referencia;
la propuesta de la Botonera usa los tres modos explícitos pedidos por el autor.

## 3. Adaptación a la arquitectura de la Botonera

La consola existente ya separa `Efectos`, `Panel` y `Reproductor`.
Se añadió una atenuación temporal propia a cada bus seleccionado, antes de su
medidor. No se modifican los valores persistidos de sus faders ni el master.
El master no recibe una segunda atenuación al seleccionar todos los destinos.
La preescucha `Cue` queda fuera de los tres destinos solicitados.

La ganancia temporal vuelve a la unidad al desactivar Mic. Así se conserva el
volumen normal, sin guardar un volumen rebajado en la configuración ni restaurar
una copia antigua. Si el operador mueve un volumen durante Mic, se propone
respetar ese nuevo ajuste al desactivarlo.

**Decisión aprobada:** porcentaje absoluto.
Con un objetivo 20% y un volumen previo 60%, absoluto significa 60 → 20 → 60;
relativo significa 60 → 12 → 60. La primera opción coincide con la referencia.
En ningún caso Mic debe subir un nivel que ya esté por debajo del objetivo.
Se calcula contra la ganancia de grupo y el master cuando
su ruta efectiva pasa por Programa; una salida independiente no recibe ese master.
Los ajustes particulares de las pistas/botones conservan sus proporciones.

## 4. Diseño aprobado

- `model`: configuración con `serde(default)`; modo, destinos, nivel y tiempos;
  entrada y sensibilidad de automático. Configuración de Mic global, igual que
  la consola/reproductor; atajo opcional por perfil en el sistema existente.
- `domain`: máquina de estados y reglas de detección/ganancia comprobables sin
  hardware; modos separados para evitar que el silencio anule una pulsación manual.
- `engine`: captura nativa de entrada mediante `rodio::cpal`, ya disponible;
  medición y control en Rust, independientes del refresco de la interfaz.
- `engine/console`: transición suave de ganancia por bus y por fotograma de audio,
  con continuidad al invertir una transición o reconstruir el grafo de salidas.
- `engine/input`: extender la resolución compartida de atajos para pulsar/soltar;
  reconocer repeticiones sin alternar varias veces y recordar la pulsación activa
  para liberar aunque cambien los modificadores o la configuración.
- `ipc`: consultar/guardar configuración, entradas, pulsación/liberación y estado;
  evento de Mic propio, también cuando no haya efectos reproduciéndose.
- `ui`: botón compacto entre Solo y Vol, sin desplazar Detener todo;
  clic derecho abre modo, casillas de destinos y parámetros. En teclado solo se
  asigna el atajo. JS solo captura la
  interacción y representa el estado recibido, con textos en los cuatro idiomas.

El botón indicaría reposo, atenuación activa y automático escuchando; un fallo de
entrada tendría estado visible. En automático, botón/atajo habilitarían o
suspenderían la escucha. Se propone iniciar sin atenuación y con el atajo vacío.
Los modos manuales no necesitan abrir una entrada de micrófono.
Mejora aprobada el 2026-10-11: abrir el apartado automático de ajustes solicita
medición temporal de la entrada del borrador, aunque Mic esté desactivado. Cerrar
el apartado libera esa medición sin suspender la escucha automática habilitada.

Ajustes propuestos: nivel al hablar, tiempo de bajada, tiempo de subida y, para
automático, entrada, sensibilidad y espera de silencio. El detector tendría
umbrales de entrada/salida separados para estabilizarlo cerca del límite.
Se interpreta la función como control de volumen: no añade mezcla, grabación ni
transmisión del micrófono. Esas funciones requerirían otro alcance.

No se prevé una dependencia nueva: CPAL ya está accesible mediante rodio y se usa
para enumerar las salidas. La captura se desarrolla y verifica para Windows/Linux.
Cada módulo de código deberá respetar el máximo de 200 líneas.

## 5. Casos de verificación al implementar

- Restauración con niveles 100%, 60%, 40%, cero y amplificación; cambios del
  operador durante Mic; objetivo superior al nivel normal nunca aumenta el audio.
- Cada destino y sus combinaciones, con rutas compartidas e independientes;
  sonidos iniciados mientras Mic ya está activo; Cue sin atenuación de Mic.
- Pulsación/liberación rápida, inversión de transición, teclado repetido,
  modificadores liberados en distinto orden y ratón/teclado simultáneos.
- Pérdida de captura o foco local, cambio de perfil/modo/atajo y cierre de ventana:
  cancelar la pulsación correspondiente sin dejar una atenuación retenida.
  Los atajos globales deben seguir operativos sin foco según la opción existente.
- Automático: umbral, pausas, nueva voz durante recuperación, desconexión/fallo
  de entrada y suspensión; restaurar el nivel al perder la entrada.
- Compatibilidad de configuraciones y perfiles antiguos con campos ausentes;
  validación Rust de datos y conflictos del atajo.
- `cargo test --lib`, `cargo build --lib`, `npm run build`; prueba funcional y
  acústica del autor, sin lanzar la app durante las verificaciones automatizadas.

La implementación actualiza manual, arquitectura, glosario, continuidad y
`CHANGELOG.md` en `[Sin publicar]`. No se publica ni cambia la versión.

El manifiesto MSIX declara `microphone` para identificar el uso de la entrada en
el paquete futuro. Microsoft recomienda declarar capacidades sensibles también
para aplicaciones `mediumIL`: [documentación de capacidades](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/app-capability-declarations#privacy-sensitive-capabilities).
`PRIVACY.md` describe la medición local; la página web pública no se ha modificado.
