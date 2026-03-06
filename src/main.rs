//! DCM Tool Kit - Herramientas para uso y manejo de archivos DICOM.
//!
//! Uso básico: `dcmtk <archivo.dcm>` muestra información resumida del estudio.

use dicom::dictionary_std::tags;
use dicom::object::open_file;
use dicom::object::DicomAttribute as _;
use dicom::object::DicomObject as _;
use std::env;
use std::path::Path;

/// Anchos de columna: mínimo = longitud del título, máximo = valor usado para truncar datos.
const W_ACCESSION: usize = 20;  // título "Accession Number" = 17
const W_NAME: usize = 28;       // título "Patient Name" = 12
const W_MODALITY: usize = 10;   // título "Modality" = 8
const W_DATE: usize = 22;      // título "Date" = 4, formato "YYYY.MM.DD HH:MM:SS"

/// Trunca `s` a `max_chars` caracteres (respeta UTF-8).
fn truncate_to_width(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect()
    }
}

/// Formatea fecha DICOM (YYYYMMDD) y hora (HHMMSS o HHMMSS.frac) al formato mostrado en README.
fn format_datetime(date_str: &str, time_str: &str) -> String {
    let date = date_str.trim();
    let time = time_str.trim();
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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: dcmtk <archivo.dcm>");
        eprintln!("Ejemplo: dcmtk myfile.dcm");
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    if !path.exists() {
        eprintln!("Error: el archivo no existe: {}", path.display());
        std::process::exit(1);
    }

    let obj = match open_file(path) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error al abrir el archivo DICOM: {}", e);
            std::process::exit(1);
        }
    };

    let accession = obj
        .attr_opt(tags::ACCESSION_NUMBER)
        .ok()
        .flatten()
        .and_then(|a| a.to_str().ok().map(String::from))
        .unwrap_or_default()
        .trim()
        .to_string();

    let patient_name = obj
        .attr_opt(tags::PATIENT_NAME)
        .ok()
        .flatten()
        .and_then(|a| a.to_str().ok().map(String::from))
        .unwrap_or_default()
        .trim()
        .to_string();

    let modality = obj
        .attr_opt(tags::MODALITY)
        .ok()
        .flatten()
        .and_then(|a| a.to_str().ok().map(String::from))
        .unwrap_or_default()
        .trim()
        .to_string();

    let study_date = obj
        .attr_opt(tags::STUDY_DATE)
        .ok()
        .flatten()
        .and_then(|a| a.to_str().ok().map(String::from))
        .unwrap_or_default()
        .trim()
        .to_string();

    let study_time = obj
        .attr_opt(tags::STUDY_TIME)
        .ok()
        .flatten()
        .and_then(|a| a.to_str().ok().map(String::from))
        .unwrap_or_default()
        .trim()
        .to_string();

    let date_display = if study_date.is_empty() && study_time.is_empty() {
        String::new()
    } else {
        format_datetime(&study_date, &study_time)
    };

    // Títulos y datos truncados al ancho máximo de cada columna
    let h_acc = truncate_to_width("Accession Number", W_ACCESSION);
    let h_name = truncate_to_width("Patient Name", W_NAME);
    let h_mod = truncate_to_width("Modality", W_MODALITY);
    let h_date = truncate_to_width("Date", W_DATE);

    println!(
        "  {:<W_ACCESSION$}  {:<W_NAME$}  {:<W_MODALITY$}  {:<W_DATE$}",
        h_acc, h_name, h_mod, h_date
    );
    println!(
        "  {:<W_ACCESSION$}  {:<W_NAME$}  {:<W_MODALITY$}  {:<W_DATE$}",
        truncate_to_width(&accession, W_ACCESSION),
        truncate_to_width(&patient_name, W_NAME),
        truncate_to_width(&modality, W_MODALITY),
        truncate_to_width(&date_display, W_DATE)
    );
}
