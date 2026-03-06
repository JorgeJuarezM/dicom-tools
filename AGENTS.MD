# Especificación para agentes AI — DCM Tool Kit

Este documento es el **punto de partida** para cualquier agente de IA que trabaje en este repositorio. Su objetivo es permitir **reproducir a la perfección** el comportamiento del programa a partir de esta especificación y del código existente.

## Propósito del proyecto

- **Lenguaje:** Rust (edición 2021).
- **Objetivo:** Aplicación de línea de comandos que proporciona herramientas para el uso y manejo de archivos DICOM.
- **Documentación para usuarios:** El archivo `README.MD` es la referencia de uso y comportamiento para usuarios finales. Debe mantenerse coherente con esta especificación.

## Referencia técnica para implementación

A continuación se describe el comportamiento que el programa debe tener para que un agente pueda replicarlo o mantenerlo de forma consistente.

### Entorno y dependencias

- **Cargo:** nombre del paquete `dcm-toolkit`, edición `2021`.
- **Ejecutable:** nombre del binario `dcmtk` (definido en `Cargo.toml` con `[[bin]] name = "dcmtk"`).
- **Dependencia DICOM:** `dicom = "0.9"` (ecosistema DICOM-rs). Se usan:
  - `dicom::object::open_file` para abrir el archivo.
  - `dicom::object::DicomObject` y `dicom::object::DicomAttribute` (traits).
  - `dicom::dictionary_std::tags` para los tags estándar.
  - `dicom::dump::dump_file` para el modo `-s` (listar todos los tags del archivo).

### Interfaz de línea de comandos

- **Argumentos:** opciones `-q/--query`, `-s/--show-tags`, `-t/--single-tag` y `-d/--diff` opcionales; uno o más argumentos posicionales (rutas de archivos DICOM). Se procesan en el orden indicado.
- **Opción `-s` / `--show-tags`:** muestra todos los tags contenidos en el archivo DICOM (meta + dataset). En este modo **solo se acepta un archivo**. Si no se pasa archivo: mensaje "Uso con -s: dcmtk -s <archivo.dcm>" y "Con -s solo se acepta un archivo.", exit 1. Si se pasan dos o más archivos: "Con -s/--show-tags solo se acepta un archivo.", exit 1. La salida se hace con `dicom::dump::dump_file(&obj)` (formato legible con tag, nombre, VR, tamaño y valor).
- **Ayuda:** la opción `-h` / `--help` (proporcionada por clap) debe mostrar el uso estándar y, a continuación, una lista de **tags DICOM comunes** utilizables con `-q`, agrupados por categoría (Paciente, Estudio, Serie, Instancia/Imagen) y el keyword especial `Date`, más ejemplos de uso (incluido `-s`).
- **Sin argumentos:** imprimir en stderr exactamente:
  - `Uso: dcmtk <archivo.dcm> [archivo2.dcm ...]`
  - `Ejemplo: dcmtk myfile.dcm`
  - Salir con código 1.
- **Archivo inexistente:** imprimir en stderr `Error: el archivo no existe: ` seguido de la ruta (tal como se pasó). Salir con código 1.
- **Error al abrir/parsear DICOM:** imprimir en stderr `Error al abrir el archivo DICOM: ` seguido del mensaje de error. Salir con código 1.
- **Múltiples archivos:** si se pasan varias rutas, se valida que todos existan antes de abrir ninguno. Si alguna no existe, se reporta y se sale con 1. Luego se abren y parsean en orden; si alguno falla al abrir, se reporta y se sale con 1. La salida es una sola cabecera y una fila por archivo (mismo formato de columnas).

### Tags DICOM utilizados

| Tag estándar (nombre)   | Tag (grupo,elemento) | Uso en la tabla   |
|-------------------------|------------------------|-------------------|
| ACCESSION_NUMBER        | (0008,0050)            | Columna "Accession Number" |
| PATIENT_NAME            | (0010,0010)            | Columna "Patient Name"     |
| MODALITY                | (0008,0060)            | Columna "Modality"         |
| STUDY_DATE              | (0008,0020)            | Parte fecha de "Date"      |
| STUDY_TIME              | (0008,0030)            | Parte hora de "Date"       |
| STUDY_ID                | (0020,0010)            | Columna "Study ID"         |

- Lectura con `attr_opt(tag)`; si el elemento no está presente, tratar el valor como vacío (no fallar).

### Formato de la salida (stdout)

- **Líneas:** primera línea = cabecera (títulos de columna). A continuación, una línea por cada archivo DICOM pasado como argumento, con los mismos anchos y formato de columnas.
- **Prefijo de línea:** dos espacios al inicio de cada línea (`  `).
- **Columnas y anchos fijos (en caracteres):**

  | Título de columna   | Ancho |
  |---------------------|-------|
  | Accession Number    | 20    |
  | Patient Name        | 28    |
  | Modality            | 10    |
  | Date                | 22    |
  | Study ID            | 40    |

- **Alineación:** todas las celdas alineadas a la izquierda dentro de su ancho.
- **Truncado:** si el texto (título o valor) supera el ancho de la columna, se trunca a exactamente ese número de **caracteres** (no bytes), de forma segura para UTF-8. No se añade "..." ni otro sufijo.
- **Valores ausentes:** si un tag no está presente o está vacío, la celda correspondiente se imprime vacía (respetando igualmente el ancho de columna).

### Formato del campo "Date"

- Se construye a partir de **Study Date** (YYYYMMDD) y **Study Time** (HHMMSS o HHMMSS.frac).
- Formato de salida: `YYYY.MM.DD HH:MM:SS`.
  - Año = 4 primeros caracteres de Study Date, mes = 2 siguientes, día = 2 siguientes.
  - Hora = 2 primeros de Study Time, minutos = 2 siguientes, segundos = 2 siguientes (si Study Time tiene menos de 6 caracteres, usar "00" para segundos).
- Si tanto Study Date como Study Time están ausentes o vacíos, la celda "Date" se deja vacía (no imprimir un placeholder).


### Modificadores

#### `-q` / `--query`

- **Propósito:** elegir qué columnas mostrar y en qué orden. Sin `-q` se usan las columnas por defecto: Accession Number, Patient Name, Modality, Date.
- **Sintaxis:** `-q KEYWORD1,KEYWORD2,...` (keywords DICOM separados por coma, sin espacios o con espacios que se ignoran).
- **Keywords admitidos (case-sensitive):** Cualquier tag definido en el estándar DICOM. Resolución con el diccionario estándar (`StandardDataDictionary`, `DataDictionary::by_name`): el keyword se resuelve al tag y al nombre para la cabecera (p. ej. "PatientName" → "Patient Name"). Keyword especial: **"Date"** (columna compuesta StudyDate + StudyTime, formato `YYYY.MM.DD HH:MM:SS`). Ejemplos de keywords: PatientName, AccessionNumber, Modality, StudyID, SOPInstanceUID, SeriesDescription.
- **Ancho de columna:** para cada columna, **máximo entre 10 caracteres y la longitud del nombre del tag** (en caracteres). Los valores se truncan a ese ancho (UTF-8 safe), sin sufijo tipo "...".
- **Salida:** primera línea = cabecera (títulos de columna en el orden indicado); siguientes líneas = una fila por archivo, mismas columnas y anchos. Prefijo de línea: dos espacios. Celdas vacías si el tag no está presente en el DICOM.
- **Errores:** si un keyword no existe en el diccionario, mensaje en stderr "Columna desconocida: <keyword>" y salida con código 1.
- **Ejemplos de uso:**
  - `dcmtk -q PatientName,AccessionNumber,StudyID myfile.dcm` → columnas Patient Name, Accession Number, Study ID.
  - `dcmtk -q Modality,Date file1.dcm file2.dcm` → dos columnas y dos filas (una por archivo).
  - `dcmtk -q SOPInstanceUID,SeriesDescription myfile.dcm` → columnas SOP Instance UID y Series Description.

#### `-s` / `--show-tags`

- **Propósito:** mostrar todos los tags contenidos en el archivo DICOM (meta-información y dataset), en formato legible.
- **Sintaxis:** `-s <archivo.dcm>`. Solo se acepta **un** archivo.
- **Comportamiento:** se abre el archivo con `open_file`, se vuelca con `dicom::dump::dump_file(&obj)`. La salida incluye primero la meta (Media Storage SOP Class/Instance UID, Transfer Syntax, etc.) y luego cada elemento del dataset con tag `(gggg,eeee)`, nombre del atributo, VR, tamaño y valor; las secuencias se muestran anidadas con indentación.
- **Salida:** texto en stdout, formato del módulo `dicom::dump` (líneas con tag, nombre, VR, longitud y valor; secuencias con ítems indentados).
- **Errores:** sin archivo → "Uso con -s: dcmtk -s <archivo.dcm>" y "Con -s solo se acepta un archivo.", exit 1. Más de un archivo → "Con -s/--show-tags solo se acepta un archivo.", exit 1. Archivo inexistente o no DICOM → mensaje en stderr y exit 1.
- **Ejemplos de uso:**
  - `dcmtk -s myfile.dcm` → listado completo de tags del archivo.
  - `dcmtk -s` (sin archivo) → error de uso.
  - `dcmtk -s a.dcm b.dcm` → error "solo se acepta un archivo".

#### `-t` / `--single-tag`

- **Propósito:** mostrar solo el valor de **un** tag por archivo: una línea por archivo, **sin cabecera** de columna.
- **Sintaxis:** `-t [opciones] <archivo.dcm> [archivo2.dcm ...]`. Se aceptan uno o más archivos.
- **Tag por defecto:** StudyInstanceUID. Si se combina con `-q`, el tag es el indicado en `-q`; en ese caso **solo se permite un keyword** en `-q` (si hay más de uno, error "Con -t/--single-tag y -q solo se permite un tag.", exit 1).
- **Salida:** una línea por archivo con únicamente el valor del tag elegido (sin título de columna).
- **Errores:** sin archivos → mensaje "Uso con -t: dcmtk -t [opciones] <archivo.dcm> ..." y "Por defecto se muestra SOPInstanceUID; use -q TAG para otro tag (solo uno).", exit 1. Con `-q` y más de un keyword → "Con -t/--single-tag y -q solo se permite un tag.", exit 1.
- **Ejemplos de uso:**
  - `dcmtk -t myfile.dcm` → una línea con el StudyInstanceUID del archivo.
  - `dcmtk -t -q PatientName myfile.dcm` → una línea con el Patient Name.
  - `dcmtk -t myfile.dcm otro.dcm` → dos líneas (una por archivo, valor del StudyInstanceUID).
  - `dcmtk -t -q PatientName,AccessionNumber myfile.dcm` → error "solo se permite un tag".
- **Uso con xargs:** la salida (una línea por archivo, sin cabecera) se puede canalizar a `xargs` para concatenar cada valor con texto o pasarlo a otro comando; por ejemplo: `dcmtk -t *.dcm | xargs -I {} echo "StudyUID: {}"` (cada línea sustituye `{}`). Ver README para más ejemplos.

#### `-d` / `--diff`

- **Propósito:** comparar dos archivos DICOM a nivel de tags (metadatos). Se omite el tag **PixelData** (7FE0,0010). La salida se imprime en formato diff (estilo unificado).
- **Sintaxis:** `-d <archivo1.dcm> <archivo2.dcm>` o `--diff <archivo1.dcm> <archivo2.dcm>`. Se requieren **exactamente 2** archivos.
- **Comportamiento:** se abren ambos archivos con `open_file`; se recorre el dataset de cada uno (iteración sobre los elementos), se omite el elemento con tag PixelData; cada tag se asocia a un valor representado como string (si el valor no es convertible a string, se usa `<...>`). Se comparan los dos conjuntos tag→valor y se imprime:
  - Primera línea: `--- <ruta_archivo1>`
  - Segunda línea: `+++ <ruta_archivo2>`
  - Para cada tag solo en el primer archivo: línea con prefijo `- ` y `(gggg,eeee) nombre: valor`.
  - Para cada tag solo en el segundo archivo: línea con prefijo `+ ` y `(gggg,eeee) nombre: valor`.
  - Para cada tag en ambos con valor distinto: una línea `- (gggg,eeee) nombre: valor1` y una línea `+ (gggg,eeee) nombre: valor2`.
  - Los tags presentes en ambos con el mismo valor no se listan (o a criterio: no imprimir líneas para no inflar la salida).
- **Nombre del tag:** se obtiene del diccionario estándar DICOM (`StandardDataDictionary::by_tag`); si no existe, se usa el formato `(gggg,eeee)`.
- **Errores:** sin archivos o con un solo archivo o con más de dos archivos → mensaje "Uso con -d/--diff: dcmtk --diff <archivo1.dcm> <archivo2.dcm>" y "Se requieren exactamente 2 archivos.", exit 1. Archivo inexistente o no DICOM → mensaje en stderr y exit 1.
- **Ejemplos de uso:**
  - `dcmtk --diff a.dcm b.dcm` → diff de tags entre `a.dcm` y `b.dcm` (sin PixelData).
  - `dcmtk -d file1.dcm file2.dcm` → igual.

### Resumen de comportamiento esperado

1. Parsear argumentos (p. ej. con `clap`): opciones `-q/--query`, `-s/--show-tags`, `-t/--single-tag` y `-d/--diff` opcionales, uno o más archivos posicionales. Si no hay `-s`, `-t`, `-d` ni archivos, imprimir en stderr "Uso: dcmtk <archivo.dcm> [archivo2.dcm ...]" y "Ejemplo: dcmtk myfile.dcm", y exit 1.
2. **Si hay `-s`:** validar que se haya pasado exactamente un archivo; si no hay archivo o hay más de uno, mensaje en stderr y exit 1. Abrir el archivo, llamar `dicom::dump::dump_file(&obj)` y terminar.
3. **Si hay `-d`:** validar que se hayan pasado exactamente 2 archivos; si no, mensaje "Uso con -d/--diff: dcmtk --diff <archivo1.dcm> <archivo2.dcm>" y "Se requieren exactamente 2 archivos.", exit 1. Abrir ambos archivos, recolectar tag→valor (omitir PixelData), comparar e imprimir en formato diff; terminar.
4. **Si hay `-t`:** validar que haya al menos un archivo; si se usó `-q`, validar que `-q` tenga exactamente un keyword (si no, error y exit 1). Resolver el tag único (por defecto StudyInstanceUID, o el de `-q`). Para cada archivo, extraer ese tag e imprimir solo su valor (una línea por archivo, sin cabecera); terminar.
5. Si hay `-q`, resolver la lista de keywords a columnas con el diccionario estándar DICOM (`by_name`); si algún keyword no existe en el diccionario, mensaje "Columna desconocida: <keyword>" y exit 1. Si no hay `-q`, usar columnas por defecto (AccessionNumber, PatientName, Modality, Date).
6. Validar que todos los paths existan; si alguno no existe, mensaje en stderr y exit 1.
7. Abrir cada archivo DICOM con la API del crate `dicom` en orden; si alguno falla, mensaje en stderr y exit 1.
8. Para cada archivo: extraer los valores de las columnas elegidas (tags con `attr_opt` o campo compuesto Date); truncar cada valor al ancho de su columna (por caracteres, UTF-8 safe).
9. Imprimir una línea de cabecera (títulos truncados al ancho de columna) y luego una línea por archivo (prefijo `  `, columnas separadas por espacios, anchos fijos).

### Arquitectura del código (escalable y reutilizable)

- **Opciones CLI:** el módulo `options` define `CliOptions` (p. ej. con `clap::Parser`). Las opciones se tratan de forma abstracta: nuevos modificadores se añaden como campos y se resuelven en un solo lugar.
- **Columnas:** el módulo `columns` define la estructura `QueryColumn` (cabecera, ancho, y un `QueryColumnKind`: tag DICOM o columna compuesta "Date"). Las columnas se resuelven desde keywords con el diccionario estándar (`StandardDataDictionary::by_name`); el nombre para la cabecera se obtiene del alias del tag (p. ej. "PatientName" → "Patient Name"). Ancho = max(10, longitud del nombre). La extracción se hace por `attr_opt(tag)` o, para Date, combinando StudyDate y StudyTime.
- **Tabla:** `print_table(columns, rows)` y `extract_row_from_path(path, columns)` son genéricos respecto a la lista de columnas; el formato y el truncado usan la cabecera y el ancho de cada `QueryColumn`.
- **Diff:** el módulo `diff` implementa la comparación de dos archivos DICOM: recorre el dataset de cada uno (omitiendo PixelData), asocia cada tag a un valor string y genera salida en formato diff (unificado: `---`/`+++` y líneas `-`/`+` para diferencias).
- **Flujo en `main`:** parsear opciones → si `-s` ejecutar run_show_tags; si `-d` ejecutar run_diff (exactamente 2 archivos); si `-t` ejecutar run_single_tag; si no, resolver columnas (por defecto o desde `-q`) → validar paths → extraer una fila por archivo → imprimir tabla. Errores unificados: mensaje en stderr y exit 1.

Cualquier cambio de comportamiento debe reflejarse aquí y en `README.MD` para mantener la coherencia entre especificación, código y documentación de usuario.
