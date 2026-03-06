# DICOM Tool Kit

Herramientas en Rust para el uso y manejo de archivos DICOM.

## Requisitos

- Rust (edición 2021). Dependencia principal: crate `dicom` 0.9 (DICOM-rs).

## Compilación

```bash
cargo build --release
```

El ejecutable se genera como `dcmtk` en `target/release/dcmtk`.

## Uso básico

### Obtener información de archivos DICOM

```bash
dcmtk myfile.dcm
dcmtk file1.dcm file2.dcm file3.dcm
```

Puedes pasar uno o más archivos. Se procesan en el orden indicado.

### Elegir columnas con `-q` / `--query`

Puedes indicar qué columnas mostrar y en qué orden con la opción `-q` (keywords DICOM separados por coma):

```bash
dcmtk -q PatientName,AccessionNumber,StudyID myfile.dcm
dcmtk -q SOPInstanceUID,Modality,SeriesDescription myfile.dcm
```

**Keywords:** Cualquier tag del estándar DICOM (case-sensitive), por ejemplo: `PatientName`, `AccessionNumber`, `Modality`, `StudyID`, `SOPInstanceUID`, `SeriesDescription`. La columna compuesta **"Date"** (Study Date + Study Time) también se puede pedir como keyword `Date`. Si no usas `-q`, se muestran por defecto: Accession Number, Patient Name, Modality, Date.

**Ancho de columna:** Para cada columna, el ancho es **10 caracteres o la longitud del nombre del tag** (el que sea mayor). Los valores se recortan a ese ancho (por caracteres, UTF-8 safe), sin puntos suspensivos.

**Salida:** una tabla con una línea de cabecera y una fila por archivo:

- La primera línea son los títulos de columna; las siguientes son una fila por cada archivo.
- Cada columna tiene un ancho fijo (mín. 10, o el largo del nombre del tag). Si un valor (o el título) es más largo, se recorta a ese ancho (por caracteres, respetando UTF-8), sin puntos suspensivos.
- Si falta un tag en el DICOM, la celda se muestra vacía.
- La fecha se muestra como `YYYY.MM.DD HH:MM:SS` a partir de Study Date y Study Time.

Ejemplo (sin `-q`, columnas por defecto):

```
  Accession Number  Patient Name  Modality    Date      
  g-04167-2         CAMBEROS ALM  US          2026.01.12 18:30:07   
```

## Errores

- **Sin argumentos:** se imprime en stderr el mensaje de uso y se sale con código 1.
- **Archivo inexistente:** mensaje `Error: el archivo no existe: <ruta>` y salida 1. Con varios archivos, se comprueba que todos existan antes de abrir ninguno.
- **Archivo no DICOM o corrupto:** mensaje `Error al abrir el archivo DICOM: <detalle>` y salida 1.
- **Keyword desconocido en `-q`:** mensaje `Columna desconocida: <keyword>` y salida 1.
