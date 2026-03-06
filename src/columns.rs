//! Definición de columnas consultables, extracción de valores y formato de tabla.
//!
//! Cualquier tag del estándar DICOM puede usarse vía -q; el diccionario estándar
//! resuelve el keyword al tag y al nombre para la cabecera. Ancho = max(10, len(nombre)).

use dicom::core::Tag;
use dicom::core::dictionary::DataDictionary;
use dicom::core::dictionary::DataDictionaryEntry;
use dicom::dictionary_std::data_element::StandardDataDictionary;
use dicom::dictionary_std::tags;
use dicom::object::open_file;
use dicom::object::DefaultDicomObject;
use dicom::object::DicomAttribute as _;
use dicom::object::DicomObject as _;
use std::path::Path;

const DEFAULT_WIDTH_MIN: usize = 10;

/// Tipo de columna: un tag DICOM o la columna compuesta "Date".
#[derive(Debug, Clone)]
pub enum QueryColumnKind {
    Tag(Tag),
    Date,
}

/// Columna consultable: cabecera, ancho (max(10, len(cabecera))) y forma de extraer el valor.
#[derive(Debug, Clone)]
pub struct QueryColumn {
    pub header: String,
    pub width: usize,
    kind: QueryColumnKind,
}

impl QueryColumn {
    pub fn header(&self) -> &str {
        &self.header
    }

    pub fn width(&self) -> usize {
        self.width
    }

    /// Extrae el valor de esta columna desde un objeto DICOM abierto.
    pub fn extract(&self, obj: &DefaultDicomObject) -> String {
        match &self.kind {
            QueryColumnKind::Tag(tag) => get_str_attr(obj, *tag),
            QueryColumnKind::Date => format_study_datetime(obj),
        }
    }

    /// Lista de columnas por defecto cuando no se usa -q.
    pub fn default_columns() -> Vec<QueryColumn> {
        let dict = StandardDataDictionary;
        vec![
            column_from_tag(&dict, tags::ACCESSION_NUMBER),
            column_from_tag(&dict, tags::PATIENT_NAME),
            column_from_tag(&dict, tags::MODALITY),
            default_date_column(),
        ]
    }

    /// Resuelve un único tag para el modo -t/--single-tag. Por defecto StudyInstanceUID; si se pasa un keyword (p. ej. desde -q), debe ser solo uno.
    pub fn single_tag_column(keywords: Option<&[String]>) -> Result<QueryColumn, String> {
        let kw = match keywords {
            None => return Ok(column_from_tag(&StandardDataDictionary, tags::SOP_INSTANCE_UID)),
            Some(k) if k.is_empty() => return Ok(column_from_tag(&StandardDataDictionary, tags::STUDY_INSTANCE_UID)),
            Some(k) if k.len() == 1 => k[0].as_str(),
            Some(_) => return Err("Con -t/--single-tag y -q solo se permite un tag.".into()),
        };
        let cols = Self::from_keywords(&[kw])?;
        Ok(cols.into_iter().next().unwrap())
    }

    /// Resuelve una lista de keywords (como en -q) a columnas usando el diccionario estándar.
    /// "Date" es una columna compuesta (StudyDate + StudyTime); el resto se buscan por keyword.
    pub fn from_keywords(keywords: &[&str]) -> Result<Vec<QueryColumn>, String> {
        let dict = StandardDataDictionary;
        let mut cols = Vec::with_capacity(keywords.len());
        for k in keywords {
            let s = k.trim();
            if s.is_empty() {
                continue;
            }
            if s.eq_ignore_ascii_case("Date") {
                cols.push(default_date_column());
                continue;
            }
            let entry = dict
                .by_name(s)
                .ok_or_else(|| format!("Columna desconocida: {}", k))?;
            cols.push(column_from_entry(entry));
        }
        if cols.is_empty() {
            return Err("No se especificó ninguna columna válida".into());
        }
        Ok(cols)
    }
}

/// Construye una columna desde una entrada del diccionario. Ancho = max(10, len(header)).
fn column_from_entry(entry: &impl DataDictionaryEntry) -> QueryColumn {
    let alias = entry.alias();
    let header = format_keyword_display(alias);
    let width = DEFAULT_WIDTH_MIN.max(header.chars().count());
    QueryColumn {
        header,
        width,
        kind: QueryColumnKind::Tag(entry.tag()),
    }
}

fn column_from_tag(dict: &StandardDataDictionary, tag: Tag) -> QueryColumn {
    let entry = dict
        .by_tag(tag)
        .expect("tag estándar debe existir en el diccionario");
    column_from_entry(entry)
}

fn default_date_column() -> QueryColumn {
    QueryColumn {
        header: "Date".to_string(),
        width: DEFAULT_WIDTH_MIN.max("Date".len()),
        kind: QueryColumnKind::Date,
    }
}

/// Convierte un keyword DICOM (alias) en texto para cabecera: "PatientName" -> "Patient Name".
fn format_keyword_display(alias: &str) -> String {
    let mut s = String::with_capacity(alias.len() + 8);
    let mut prev_lower = false;
    for c in alias.chars() {
        if c.is_uppercase() && prev_lower {
            s.push(' ');
        }
        prev_lower = c.is_lowercase() || c.is_numeric();
        if s.is_empty() {
            s.extend(c.to_uppercase());
        } else {
            s.push(c);
        }
    }
    s
}

fn get_str_attr(obj: &DefaultDicomObject, tag: Tag) -> String {
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
