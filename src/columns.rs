//! Definición de columnas consultables, extracción de valores y formato de tabla.
//!
//! Diseño genérico: una columna tiene keyword, cabecera, ancho y una forma de
//! extraer el valor de un objeto DICOM. Nuevas columnas se añaden al enum y a
//! `from_keyword`.

use dicom::dictionary_std::tags;
use dicom::object::open_file;
use dicom::object::DefaultDicomObject;
use dicom::object::DicomAttribute as _;
use dicom::object::DicomObject as _;
use std::path::Path;

/// Columna consultable: keyword para -q, cabecera para la tabla y ancho fijo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryColumn {
    AccessionNumber,
    PatientName,
    Modality,
    Date,
    StudyId,
}

impl QueryColumn {
    /// Keywords aceptados por -q (sin espacios). Añadir aquí al extender columnas.
    #[allow(dead_code)]
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::AccessionNumber => "AccessionNumber",
            Self::PatientName => "PatientName",
            Self::Modality => "Modality",
            Self::Date => "Date",
            Self::StudyId => "StudyId",
        }
    }

    /// Cabecera mostrada en la tabla.
    pub fn header(&self) -> &'static str {
        match self {
            Self::AccessionNumber => "Accession Number",
            Self::PatientName => "Patient Name",
            Self::Modality => "Modality",
            Self::Date => "Date",
            Self::StudyId => "Study ID",
        }
    }

    /// Ancho fijo de la columna (caracteres).
    pub fn width(&self) -> usize {
        match self {
            Self::AccessionNumber => 20,
            Self::PatientName => 28,
            Self::Modality => 10,
            Self::Date => 22,
            Self::StudyId => 40,
        }
    }

    /// Parsea un keyword (como en -q) al enum. Case-sensitive.
    pub fn from_keyword(s: &str) -> Option<Self> {
        match s.trim() {
            "AccessionNumber" => Some(Self::AccessionNumber),
            "PatientName" => Some(Self::PatientName),
            "Modality" => Some(Self::Modality),
            "Date" => Some(Self::Date),
            "StudyId" => Some(Self::StudyId),
            _ => None,
        }
    }

    /// Lista de columnas por defecto cuando no se usa -q.
    pub fn default_columns() -> &'static [QueryColumn] {
        &[
            QueryColumn::AccessionNumber,
            QueryColumn::PatientName,
            QueryColumn::Modality,
            QueryColumn::Date,
        ]
    }

    /// Resuelve una lista de keywords a columnas. Devuelve error si algún keyword es desconocido.
    pub fn from_keywords(keywords: &[&str]) -> Result<Vec<QueryColumn>, String> {
        let mut cols = Vec::with_capacity(keywords.len());
        for k in keywords {
            match Self::from_keyword(k) {
                Some(c) => cols.push(c),
                None => return Err(format!("Columna desconocida: {}", k)),
            }
        }
        Ok(cols)
    }

    /// Extrae el valor de esta columna desde un objeto DICOM abierto.
    pub fn extract(&self, obj: &DefaultDicomObject) -> String {
        match self {
            Self::AccessionNumber => get_str_attr(obj, tags::ACCESSION_NUMBER),
            Self::PatientName => get_str_attr(obj, tags::PATIENT_NAME),
            Self::Modality => get_str_attr(obj, tags::MODALITY),
            Self::Date => format_study_datetime(obj),
            Self::StudyId => get_str_attr(obj, tags::STUDY_ID),
        }
    }
}

fn get_str_attr(obj: &DefaultDicomObject, tag: dicom::core::Tag) -> String {
    obj.attr_opt(tag)
        .ok()
        .flatten()
        .and_then(|a| a.to_str().ok().map(String::from))
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn format_study_datetime(obj: &DefaultDicomObject) -> String {
    let date = get_str_attr(obj, tags::STUDY_DATE);
    let time = get_str_attr(obj, tags::STUDY_TIME);
    if date.is_empty() && time.is_empty() {
        return String::new();
    }
    let date = date.trim();
    let time = time.trim();
    if date.len() >= 8 && time.len() >= 6 {
        let y = &date[0..4];
        let m = &date[4..6];
        let d = &date[6..8];
        let h = &time[0..2];
        let min = &time[2..4];
        let sec = if time.len() >= 6 { &time[4..6] } else { "00" };
        format!("{}.{}.{} {}:{}:{}", y, m, d, h, min, sec)
    } else {
        format!("{} {}", date, time).trim().to_string()
    }
}

/// Trunca `s` a `max_chars` caracteres (respeta UTF-8).
pub fn truncate_to_width(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect()
    }
}

const LINE_PREFIX: &str = "  ";

/// Imprime la tabla: una línea de cabecera y una por cada fila.
/// Las celdas se truncan al ancho de cada columna.
pub fn print_table(columns: &[QueryColumn], rows: &[Vec<String>]) {
    print_header_line(columns);
    for row in rows {
        print_data_line(columns, row);
    }
}

fn print_header_line(columns: &[QueryColumn]) {
    let parts: Vec<String> = columns
        .iter()
        .map(|c| truncate_to_width(c.header(), c.width()))
        .collect();
    print_row(&parts, columns);
}

fn print_data_line(columns: &[QueryColumn], row: &[String]) {
    let parts: Vec<String> = columns
        .iter()
        .zip(row.iter())
        .map(|(c, val)| truncate_to_width(val, c.width()))
        .collect();
    print_row(&parts, columns);
}

fn print_row(parts: &[String], columns: &[QueryColumn]) {
    let line: String = parts
        .iter()
        .zip(columns.iter())
        .map(|(s, c)| format!("{:<width$}", s, width = c.width()))
        .collect::<Vec<_>>()
        .join("  ");
    println!("{}{}", LINE_PREFIX, line);
}

/// Abre un archivo DICOM y extrae una fila (un valor por columna).
pub fn extract_row_from_path(
    path: &Path,
    columns: &[QueryColumn],
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let obj = open_file(path)?;
    Ok(columns.iter().map(|c| c.extract(&obj)).collect())
}
