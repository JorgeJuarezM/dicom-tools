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

### Obtener información de un archivo DICOM

```bash
dcmtk myfile.dcm
```

**Salida:** una tabla de una fila con las columnas:

| Columna           | Origen DICOM        | Ancho (caracteres) |
|-------------------|---------------------|--------------------|
| Accession Number  | (0008,0050)         | 20                 |
| Patient Name      | (0010,0010)         | 28                 |
| Modality          | (0008,0060)         | 10                 |
| Date              | (0008,0020) + (0008,0030) | 22          |

- La primera línea son los títulos de columna, la segunda los valores del archivo.
- Cada columna tiene un ancho fijo. Si un valor (o el título) es más largo, se recorta a ese ancho (por caracteres, respetando UTF-8), sin puntos suspensivos.
- Si falta un tag en el DICOM, la celda se muestra vacía.
- La fecha se muestra como `YYYY.MM.DD HH:MM:SS` a partir de Study Date y Study Time.

Ejemplo:

```
  Accession Number      Patient Name                  Modality    Date                  
  g-04167-2             CAMBEROS ALMADA^MARIA INES^^  US          2026.01.12 18:30:07   
```

## Errores

- **Sin argumentos:** se imprime en stderr el mensaje de uso y se sale con código 1.
- **Archivo inexistente:** mensaje `Error: el archivo no existe: <ruta>` y salida 1.
- **Archivo no DICOM o corrupto:** mensaje `Error al abrir el archivo DICOM: <detalle>` y salida 1.
