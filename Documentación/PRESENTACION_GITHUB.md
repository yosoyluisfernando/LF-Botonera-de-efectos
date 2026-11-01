# Presentación del proyecto en GitHub

## README en cuatro idiomas

La portada predeterminada es `README.md`, en español. Los enlaces del selector
llevan a `README.en.md`, `README.pt-BR.md` y `README.pt-PT.md`, en la raíz.
Cada versión ofrece descargas, primeros pasos, un resumen de funciones y un
desplegable con los detalles. La documentación técnica se enlaza desde la portada.
La galería reutiliza tres PNG maestros de `Capturas/windows/es/`: rejilla clara,
panel fijo en tema oscuro y editor de pistas. La primera imagen queda visible;
las otras dos se abren en un desplegable. Cada README localiza el texto alternativo
y aclara que son capturas de Windows en español tomadas en julio de 2026.

En español e inglés, el título es **LF Botonera de Efectos (Soundboard)**.
En ambas versiones portuguesas es **LF Botonera de Efectos (Painel de efeitos sonoros)**.
El término entre paréntesis describe el tipo de aplicación en el idioma de la
presentación; no cambia el nombre del repositorio, la identidad de la aplicación
ni los nombres de los paquetes.

Las traducciones se adaptan a cada variante: por ejemplo, `arquivo`, `usuário`,
`mouse`, `abas` y `baixar` en Brasil; `ficheiro`, `utilizador`, `rato`,
`separadores` y `transferir` en Portugal. El nombre del producto se conserva.

## Mantenimiento

- El español es la referencia editorial. Al cambiar funciones, descargas o
  requisitos, revisar los cuatro archivos en el mismo cambio.
- Traducir el sentido y el uso real, manteniendo equivalentes los datos técnicos,
  límites, formatos y enlaces. No traducir comandos, extensiones ni rutas.
- Presentar solo funciones publicadas. La integración OBS local no se anuncia
  como parte de la versión estable.
- Los manuales y documentos enlazados siguen principalmente en español;
  las portadas lo indican para evitar confusiones.
- Comprobar los enlaces relativos, las anclas del índice y los desplegables.
  Revisar las cuatro vistas renderizadas en GitHub al publicar.
- No hay traducción ni sincronización automática entre archivos. Las herramientas
  de traducción pueden ayudar a preparar un borrador que después se revisa.

## Metadatos de «Acerca de»

Descripción y sitio web aplicados el 2026-11-01. Por decisión posterior del autor,
los 20 topics iniciales se redujeron a dos para simplificar la presentación visual.

**Descripción:**

> Botonera de efectos de sonido (Soundboard) para radio y streaming en directo. Free and open-source soundboard for live radio and streaming. Windows y Linux · Español, English, Português (Brasil/Portugal).

**Sitio web:** https://lfbotonera.blogspot.com/

**Topics publicados:**

```text
soundboard
painel-de-efeitos-sonoros
```

Los topics identifican el tipo de aplicación en inglés y portugués. La expresión
portuguesa es compartida por Brasil y Portugal. Los demás términos descriptivos y
las traducciones permanecen en los README. Esta selección breve sustituye a la
lista inicial extensa; no volver a ampliarla sin petición del autor.

Se editan desde el engranaje junto a **About / Acerca de** en la portada de GitHub.
GitHub admite hasta 20 topics, de hasta 50 caracteres cada uno, escritos con
minúsculas, números y guiones. Son distintos de las etiquetas de incidencias.
Referencia: [documentación oficial de GitHub sobre topics](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/classifying-your-repository-with-topics).
