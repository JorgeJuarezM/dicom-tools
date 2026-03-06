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

### Interfaz de línea de comandos

- **Argumentos:** opción `-q/--query` opcional; uno o más argumentos posicionales (rutas de archivos DICOM). Se procesan en el orden indicado.
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

- **`-q` / `--query`:** lista de columnas a mostrar (keywords DICOM separados por coma). Si no se especifica, se usan las columnas por defecto (Accession Number, Patient Name, Modality, Date).
- **Keywords admitidos (case-sensitive):** Cualquier tag definido en el estándar DICOM. Se debe usar el diccionario estándar (p. ej. `StandardDataDictionary` y `DataDictionary::by_name`) para resolver el keyword al tag y al nombre para la cabecera. El ancho de la columna por defecto debe ser **10 caracteres o el tamaño del nombre del tag (el que resulte mayor)**. La columna compuesta "Date" (StudyDate + StudyTime) se admite como keyword "Date"; el resto se buscan en el diccionario (p. ej. "PatientName", "StudyID", "SOPInstanceUID").
- **Ejemplo:** `dcmtk -q PatientName,AccessionNumber,StudyId myfile.dcm`  
  Salida: una línea de cabecera con "Patient Name", "Accession Number", "Study ID", y una fila por archivo con los valores (mismo formato de anchos y truncado). Si un keyword es desconocido, se imprime en stderr "Columna desconocida: <keyword>" y se sale con 1.

### Resumen de comportamiento esperado

1. Parsear argumentos (p. ej. con `clap`): opción `-q/--query` opcional, uno o más archivos posicionales. Validar que haya al menos un archivo; si no, imprimir en stderr "Uso: dcmtk <archivo.dcm> [archivo2.dcm ...]" y "Ejemplo: dcmtk myfile.dcm", y exit 1.
2. Si hay `-q`, resolver la lista de keywords a columnas con el diccionario estándar DICOM (`by_name`); si algún keyword no existe en el diccionario, mensaje "Columna desconocida: <keyword>" y exit 1. Si no hay `-q`, usar columnas por defecto (AccessionNumber, PatientName, Modality, Date).
3. Validar que todos los paths existan; si alguno no existe, mensaje en stderr y exit 1.
4. Abrir cada archivo DICOM con la API del crate `dicom` en orden; si alguno falla, mensaje en stderr y exit 1.
5. Para cada archivo: extraer los valores de las columnas elegidas (tags con `attr_opt` o campo compuesto Date); truncar cada valor al ancho de su columna (por caracteres, UTF-8 safe).
6. Imprimir una línea de cabecera (títulos truncados al ancho de columna) y luego una línea por archivo (prefijo `  `, columnas separadas por espacios, anchos fijos).

### Arquitectura del código (escalable y reutilizable)

- **Opciones CLI:** el módulo `options` define `CliOptions` (p. ej. con `clap::Parser`). Las opciones se tratan de forma abstracta: nuevos modificadores se añaden como campos y se resuelven en un solo lugar.
- **Columnas:** el módulo `columns` define la estructura `QueryColumn` (cabecera, ancho, y un `QueryColumnKind`: tag DICOM o columna compuesta "Date"). Las columnas se resuelven desde keywords con el diccionario estándar (`StandardDataDictionary::by_name`); el nombre para la cabecera se obtiene del alias del tag (p. ej. "PatientName" → "Patient Name"). Ancho = max(10, longitud del nombre). La extracción se hace por `attr_opt(tag)` o, para Date, combinando StudyDate y StudyTime.
- **Tabla:** `print_table(columns, rows)` y `extract_row_from_path(path, columns)` son genéricos respecto a la lista de columnas; el formato y el truncado usan la cabecera y el ancho de cada `QueryColumn`.
- **Flujo en `main`:** parsear opciones → resolver columnas (por defecto o desde `-q`) → validar paths → extraer una fila por archivo → imprimir tabla. Errores unificados: mensaje en stderr y exit 1.

Cualquier cambio de comportamiento debe reflejarse aquí y en `README.MD` para mantener la coherencia entre especificación, código y documentación de usuario.
