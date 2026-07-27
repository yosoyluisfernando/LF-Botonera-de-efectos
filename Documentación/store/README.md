# Actualización de la ficha de Microsoft Store

Fuente de verdad para la actualización de metadatos posterior a la publicación de
LF Botonera de Efectos 1.2.1. Esta actualización no sustituye el paquete aprobado.

## Enlaces oficiales

- Sitio web: `https://lfbotonera.blogspot.com/`
- Soporte: `https://lfbotonera.blogspot.com/p/soporte.html`
- Privacidad: `https://lfbotonera.blogspot.com/p/privacidad.html`
- Código fuente: `https://github.com/yosoyluisfernando/LF-Botonera-de-efectos`

## Orden definitivo de la galería

1. `principal/01-rejilla-modo-claro.png`: vista general y primera impresión.
2. `principal/05-rejilla-modo-oscuro-panel-fijo.png`: tema oscuro con panel fijo.
3. `editor/01-forma-de-onda-y-cues.png`: edición no destructiva de pistas.
4. `panel/03-escucha-previa.png`: preescucha y salida independiente.
5. `consola/01-consola-audio-modo-oscuro.png`: consola, indicando que está en prueba.
6. `principal/03-tipos-de-boton.png`: audio, carpetas y locuciones dinámicas.
7. `configuracion/05-hora-y-clima-aragua-de-barcelona.png`: clima, siempre al final.

Los archivos parten de `Capturas/windows/es/`. La segunda imagen queda pendiente de
captura limpia. No usar `panel/02-reproductor-auxiliar-cargando.png`: muestra un
proceso de carga y no representa el estado normal del reproductor.

## Idiomas

- [`es-ES.md`](es-ES.md)
- [`en-US.md`](en-US.md)
- [`pt-BR.md`](pt-BR.md)
- [`pt-PT.md`](pt-PT.md)

Las siete imágenes se cargaron en ese mismo orden en las cuatro fichas. La interfaz
de las capturas permanece en español. La ficha española incluye los siete pies de
imagen; las otras tres conservan las capturas sin pie. Los textos localizados quedan
preparados en esta carpeta por si se incorporan en una actualización posterior.

## Alcance del envío

- Mantener el paquete MSIX 1.2.1 aprobado.
- Sustituir el sitio web y el soporte antiguos de GitHub por las páginas oficiales de
  Blogger. La política aprobada continúa como texto completo dentro de Store.
- Sustituir la descripción y la descripción breve en los cuatro idiomas.
- Añadir las siete capturas en las cuatro fichas y los pies en español.
- Mantener 1.2.1 como versión publicada; no prometer funciones futuras.

La Submission 2 (`1152921505701463690`) se envió a certificación el 2026-07-21 sin
cambiar el paquete MSIX 1.2.1.

## Próxima actualización funcional

La versión propuesta es **1.3.0** (`1.3.0.0` dentro del MSIX), porque incorpora la
Biblioteca y el buscador interno sin romper compatibilidad. Sus textos localizados
están en [`NOTAS_VERSION_SIGUIENTE.md`](NOTAS_VERSION_SIGUIENTE.md).

El 2026-07-27 se creó y envió la Submission 3
(`1152921505701501616`) del producto `9NJ8ST39QP7V`:

- paquete `LF-Botonera-1.3.0.0-x64-unsigned.msix` validado por Partner Center;
- notas de versión actualizadas en español, inglés, portugués de Brasil y portugués
  de Portugal;
- publicación configurada para comenzar automáticamente cuando apruebe la
  certificación;
- estado confirmado al terminar el envío: `Update in certification`, etapa
  `Pre-processing` (paso 2 de 4).

No cancelar la certificación ni crear otra Submission. El siguiente control consiste
en esperar la decisión de Microsoft; si aprueba, la publicación comienza sin fecha ni
hora manual.

### Paquete preparado

El 2026-07-27 se generó y auditó:

`src-tauri/target/msix/LF-Botonera-1.3.0.0-x64-unsigned.msix`

- identidad: `LuisFernandoVelasquez.LFBotoneradeEfectos`;
- editor: `CN=AD90DE58-447F-47AE-AC1A-3D369955282B`;
- arquitectura: `x64`;
- versión MSIX: `1.3.0.0`;
- versión del ejecutable: `1.3.0`;
- canal encontrado en el binario: `microsoft_store`;
- tamaño: `9.694.349` bytes;
- SHA-256:
  `56E2D70AC832B36AB1A005B734DB948FB0CB0F19B2886EBEA5705E5FFCD2AD8E`.

El paquete se desempaquetó correctamente: 60 archivos, manifiesto 1.3.0.0 y ejecutable
idéntico al original por SHA-256. `cargo test --lib` aprobó 265 pruebas y dejó 16
ignoradas deliberadamente; `cargo build --lib`, `cargo check`, `npm run build` y el
formato de los módulos nuevos aprobaron. `cargo fmt --check` global continúa fallando
por formato histórico en archivos no relacionados; no se aplicó una reescritura
masiva dentro de este release.
