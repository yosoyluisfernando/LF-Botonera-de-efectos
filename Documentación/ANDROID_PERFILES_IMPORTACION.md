# Perfiles y control remoto en Android

La barra Android muestra el icono de perfiles con el nombre del perfil activo
en su etiqueta accesible. El acceso remoto también muestra un icono.
En Ajustes → Control remoto se puede activar o desactivar el cliente.
Guardar la desactivación oculta su acceso y cierra la sesión remota.
Las instalaciones anteriores conservan el acceso visible al actualizar.

Al importar pestañas se elige un identificador libre entre todas las pestañas
de todos los perfiles. Al importar perfiles se elige un identificador libre
entre los perfiles existentes. Se reutiliza el asignador existente.
Esto evita reutilizar identificadores al borrar e importar de nuevo.

La carga de configuración repara identificadores duplicados de perfiles y
pestañas creados por versiones anteriores. Conserva las rutas y los datos de
los botones y ajusta sus identificadores y la referencia a la pestaña activa.
El disparo de un botón de la rejilla busca dentro del perfil activo.
La activación de un perfil importado aplica el volumen y la precarga con el
mismo recorrido que la selección del perfil desde su menú.

Los formatos .bdelf y .bdeplf conservan las rutas de audio y los metadatos de
edición. La prueba de exportación e importación de esta pasada usa los mismos
archivos en la misma instalación Android, como indicó el autor.
