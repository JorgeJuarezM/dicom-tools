# DICOM Tool Kit

Herramientas en Rust para el uso y manejo de archivos DICOM.

## Requisitos

- Rust (edición 2021). Dependencia principal: crate `dicom` 0.9 (DICOM-rs).

## Compilación

```bash
cargo build --release
```

El ejecutable se genera como `dcmtk` en `target/release/dcmtk`.

## Ayuda

Para ver el uso y la lista de tags DICOM comunes utilizables con `-q`:

```bash
dcmtk --help
# o
dcmtk -h
```

La ayuda muestra las opciones y, al final, una lista de tags comunes agrupados por categoría (Paciente, Estudio, Serie, Instancia/Imagen) y el keyword especial `Date`.

### Mostrar todos los tags del archivo (`-s` / `--show-tags`)

Con `-s` se listan **todos** los tags contenidos en el archivo DICOM (incluido el grupo de meta-información y el dataset). En este modo **solo se acepta un archivo**.

```bash
dcmtk -s myfile.dcm
```

Si no se pasa archivo o se pasan varios, se muestra un error y se sale con código 1.

Ejemplo de salida con `-s` (inicio del listado):

```
Media Storage SOP Class UID: 1.2.840.10008.5.1.4.1.1.6.1
Media Storage SOP Instance UID: 1.2.840.113663.1500.1.527525534.3.1.20260112.183027.299
Transfer Syntax: 1.2.840.10008.1.2.4.70 (JPEG Lossless, Non-Hierarchical, First-Order Prediction)
Implementation Class UID: 1.2.40.0.13.1.3
Implementation version name: dcm4che-5.23.2

----------------------------------------------------------
(0008,0005) SpecificCharacterSet         CS (1, 10 bytes): "ISO_IR 100"
(0008,0008) ImageType                    CS (3, 18 bytes): ["ORIGINAL", "PRIMARY", ""]
(0008,0012) InstanceCreationDate         DA (1,  8 bytes): 2026-01-12
(0008,0013) InstanceCreationTime         TM (1,  6 bytes): 18:30:27
(0008,0016) SOPClassUID                  UI (1, 28 bytes): "1.2.840.10008.5.1.4.1.1.6.1"
(0008,0018) SOPInstanceUID               UI (1, 56 bytes): "1.2.840.113663.1500.1.527525534.3.1.20260112.183027.2...
(0008,0020) StudyDate                    DA (1,  8 bytes): 2026-01-12
(0008,0021) SeriesDate                   DA (1,  8 bytes): 2026-01-12
(0008,0050) AccessionNumber              SH (1, 10 bytes): "g-04167-2"
(0008,0060) Modality                     CS (1,  2 bytes): "US"
(0010,0010) PatientName                  PN (1, 30 bytes): "CAMBEROS ALMADA^MARIA INES^^^"
(0010,0020) PatientID                   LO (1, 12 bytes): "CAMI0410190"
...
```

Las secuencias (SQ) se muestran con ítems indentados; el listado continúa hasta el final del dataset.

### Un solo tag por archivo (`-t` / `--single-tag`)

Con `-t` se muestra **solo el valor de un tag** por archivo: **una línea por archivo, sin cabecera**. Por defecto se usa **StudyInstanceUID**. Se puede combinar con `-q` para elegir otro tag; en ese caso **solo se permite un keyword** en `-q`.

```bash
dcmtk -t myfile.dcm
dcmtk -t -q PatientName myfile.dcm
dcmtk -t file1.dcm file2.dcm
```

Ejemplo de salida con `-t` (por defecto, SOPInstanceUID):

```bash
dcmtk -t myfile.dcm
```

```
2.25.179733568928423661964122720616650769519
```

Ejemplo con `-t -q PatientName` (un archivo):

```bash
dcmtk -t -q PatientName myfile.dcm
```

```
CAMBEROS ALMADA^MARIA INES^^^
```

Ejemplo con `-t` y varios archivos (una línea por archivo, mismo tag):

```bash
dcmtk -t myfile.dcm myfile.dcm
```

```
2.25.179733568928423661964122720616650769519
2.25.179733568928423661964122720616650769519
```

Si usas `-t` con `-q` y más de un tag (p. ej. `-q PatientName,AccessionNumber`), se muestra error: "Con -t/--single-tag y -q solo se permite un tag."

**Uso con `xargs`:** al imprimir una línea por archivo (sin cabecera), la salida de `-t` se puede canalizar a `xargs` para concatenar cada valor con otro texto o usarlo como argumento de otro comando:

```bash
# Añadir un prefijo a cada UID
dcmtk -t *.dcm | xargs -I {} echo "StudyUID: {}"

# Añadir sufijo (p. ej. para etiquetar)
dcmtk -t *.dcm | xargs -I {} echo "{} -- procesado"

# Pasar cada UID a otro comando (ejemplo: imprimir con formato)
dcmtk -t file1.dcm file2.dcm | xargs -I {} printf "UID=%s\n" "{}"

# Con otro tag (-q): cada Patient Name con un prefijo
dcmtk -t -q PatientName *.dcm | xargs -I {} echo "Paciente: {}"
```

Con `-I {}` cada línea de la salida de `dcmtk -t` sustituye a `{}` en el comando que sigue; así puedes combinar el valor del tag con el texto que quieras.

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

Ejemplo con `-q` (columnas elegidas: Patient Name, Accession Number, Study ID):

```bash
dcmtk -q PatientName,AccessionNumber,StudyID myfile.dcm
```

```
  Patient Name  Accession Number  Study ID  
  CAMBEROS ALM  g-04167-2         USG       
```

Ejemplo con `-q` (varias columnas y varios archivos):

```bash
dcmtk -q Modality,SeriesDescription,SOPInstanceUID myfile.dcm myfile.dcm
```

```
  Modality    Series Description  SOP Instance UID        
  US         USG                 1.2.840.113663.1500...  
  US         USG                 1.2.840.113663.1500...  
```

Ejemplo sin `-q` (columnas por defecto):

```
  Accession Number  Patient Name  Modality    Date      
  g-04167-2         CAMBEROS ALM  US          2026.01.12 18:30:07   
```

## Errores

- **Sin argumentos:** se imprime en stderr el mensaje de uso y se sale con código 1.
- **Archivo inexistente:** mensaje `Error: el archivo no existe: <ruta>` y salida 1. Con varios archivos, se comprueba que todos existan antes de abrir ninguno.
- **Archivo no DICOM o corrupto:** mensaje `Error al abrir el archivo DICOM: <detalle>` y salida 1.
- **Keyword desconocido en `-q`:** mensaje `Columna desconocida: <keyword>` y salida 1.
- **Con `-s`:** sin archivo → "Uso con -s: dcmtk -s <archivo.dcm>". Más de un archivo → "Con -s/--show-tags solo se acepta un archivo.". Salida 1.
- **Con `-t`:** sin archivos → mensaje de uso con -t. Con `-q` y más de un tag → "Con -t/--single-tag y -q solo se permite un tag.". Salida 1.
