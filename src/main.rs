//! DCM Tool Kit - Herramientas para uso y manejo de archivos DICOM.
//!
//! Uso básico: `dcmtk <archivo.dcm>` muestra información resumida del estudio.

use dicom::dictionary_std::tags;
use dicom::object::open_file;
use dicom::object::DicomAttribute as _;
use dicom::object::DicomObject as _;
use std::env;
use std::path::Path;

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

    // Anchos de columna para alinear con el ejemplo del README
    const W_ACCESSION: usize = 12;
    const W_NAME: usize = 24;
    const W_MODALITY: usize = 10;

    println!(
        "  {:<W_ACCESSION$}  {:<W_NAME$}  {:<W_MODALITY$}  {}",
        "Accession Number",
        "Patient Name",
        "Modality",
        "Date"
    );
    println!(
        "  {:<W_ACCESSION$}  {:<W_NAME$}  {:<W_MODALITY$}  {}",
        accession,
        patient_name,
        modality,
        date_display
    );
}
