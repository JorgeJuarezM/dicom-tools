//! Opciones de línea de comandos y configuración abstracta de la aplicación.

use clap::Parser;
use std::path::PathBuf;

/// Texto que se muestra tras la ayuda estándar (lista de tags DICOM comunes).
const AFTER_HELP: &str = r#"DICOM TAGS COMUNES (utilizables con -q, case-sensitive):

  Paciente:
    PatientName, PatientID, PatientBirthDate, PatientSex, PatientAge
    PatientWeight, PatientSize, ReferringPhysicianName

  Estudio:
    StudyDate, StudyTime, StudyID, StudyInstanceUID, StudyDescription
    AccessionNumber, RequestedProcedureID, RequestedProcedureDescription

  Serie:
    Modality, SeriesNumber, SeriesDescription, SeriesInstanceUID
    SeriesDate, SeriesTime, ProtocolName, BodyPartExamined

  Instancia / Imagen:
    SOPInstanceUID, InstanceNumber, ImageType, Rows, Columns
    NumberOfFrames, PhotometricInterpretation, BitsAllocated, BitsStored

  Columna compuesta (keyword especial):
    Date   (equivale a StudyDate + StudyTime formateados como YYYY.MM.DD HH:MM:SS)

  Opción -s/--show-tags:
    Muestra todos los tags contenidos en el archivo DICOM. Solo se acepta un archivo.
    Ejemplo: dcmtk -s myfile.dcm

  Opción -t/--single-tag:
    Muestra solo el valor de un tag por archivo (una línea por archivo, sin cabecera).
    Por defecto usa SOPInstanceUID. Con -q se usa el tag indicado (solo uno).
    Uso con xargs: dcmtk -t *.dcm | xargs -I {} echo "UID: {}"

  Opción -d/--diff:
    Compara dos archivos DICOM a nivel de tags (se omite PixelData). Salida en formato diff.
    Ejemplo: dcmtk --diff file1.dcm file2.dcm

  Ejemplos:
    dcmtk myfile.dcm
    dcmtk -q PatientName,AccessionNumber,StudyID myfile.dcm
    dcmtk -q Modality,SeriesDescription,SOPInstanceUID file1.dcm file2.dcm
    dcmtk -s myfile.dcm
"#;

/// Opciones de la aplicación: modificadores y argumentos posicionales.
/// Diseñado para extenderse con nuevos modificadores sin cambiar la firma.
#[derive(Debug, Parser)]
#[command(
    name = "dcmtk",
    about = "Herramientas para archivos DICOM",
    after_help = AFTER_HELP
)]
pub struct CliOptions {
    /// Muestra todos los tags contenidos en el archivo DICOM (solo se acepta un archivo).
    #[arg(short = 's', long = "show-tags")]
    pub show_tags: bool,

    /// Muestra solo un tag por archivo (una línea por archivo, sin cabecera). Por defecto SOPInstanceUID; con -q se usa el tag indicado (solo uno).
    #[arg(short = 't', long = "single-tag")]
    pub single_tag: bool,

    /// Compara dos archivos DICOM a nivel de tags (se omite PixelData). Salida en formato diff. Requiere exactamente 2 archivos.
    #[arg(short = 'd', long = "diff")]
    pub diff: bool,

    /// Columnas a mostrar (keywords DICOM separados por coma). Si no se especifica, se usan las columnas por defecto.
    #[arg(short = 'q', long = "query", value_name = "COLUMNAS")]
    pub query: Option<String>,

    /// Uno o más archivos DICOM a procesar. Con -s/--show-tags solo se acepta un archivo.
    #[arg(value_name = "archivo.dcm")]
    pub files: Vec<PathBuf>,
}

impl CliOptions {
    /// Rutas de archivos a procesar (en orden).
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    /// Si se usó -s/--show-tags (listar todos los tags del archivo).
    pub fn show_tags(&self) -> bool {
        self.show_tags
    }

    /// Si se usó -t/--single-tag (mostrar solo un tag por archivo, sin cabecera).
    pub fn single_tag(&self) -> bool {
        self.single_tag
    }

    /// Si se usó -d/--diff (comparar dos archivos DICOM).
    pub fn diff(&self) -> bool {
        self.diff
    }

    /// Si se usó -q/--query, devuelve la lista de keywords en orden; si no, None.
    pub fn query_keywords(&self) -> Option<Vec<String>> {
        self.query.as_deref().map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect()
        })
    }
}
