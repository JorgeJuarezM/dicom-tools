//! DCM Tool Kit - Herramientas para uso y manejo de archivos DICOM.
//!
//! Uso: `dcmtk [opciones] <archivo.dcm> [archivo2.dcm ...]`
//! Opción `-q/--query`: columnas a mostrar (keywords separados por coma).
//! Opción `-s/--show-tags`: muestra todos los tags del archivo (solo un archivo).

mod columns;
mod diff;
mod options;

use clap::Parser;
use columns::{extract_row_from_path, print_table, QueryColumn};
use dicom::dump::dump_file;
use dicom::object::open_file;
use options::CliOptions;
use std::process::exit;

fn main() {
    let opts = match parse_args() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };

    if opts.show_tags() {
        if let Err(e) = run_show_tags(&opts) {
            eprintln!("{}", e);
            exit(1);
        }
        return;
    }

    if opts.diff() {
        if let Err(e) = run_diff(&opts) {
            eprintln!("{}", e);
            exit(1);
        }
        return;
    }

    if opts.single_tag() {
        if let Err(e) = run_single_tag(&opts) {
            eprintln!("{}", e);
            exit(1);
        }
        return;
    }

    let columns = match resolve_columns(&opts) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };

    for path in opts.files() {
        if !path.exists() {
            eprintln!("Error: el archivo no existe: {}", path.display());
            exit(1);
        }
    }

    let mut rows = Vec::with_capacity(opts.files().len());
    for path in opts.files() {
        match extract_row_from_path(path, &columns) {
            Ok(row) => rows.push(row),
            Err(e) => {
                eprintln!("Error al abrir el archivo DICOM: {}", e);
                exit(1);
            }
        }
    }

    print_table(&columns, &rows);
}

/// Modo -t: muestra solo un tag por archivo (una línea por archivo, sin cabecera). Por defecto SOPInstanceUID; con -q un solo tag.
fn run_single_tag(opts: &CliOptions) -> Result<(), String> {
    let files = opts.files();
    if files.is_empty() {
        return Err("Uso con -t: dcmtk -t [opciones] <archivo.dcm> [archivo2.dcm ...]\nPor defecto se muestra SOPInstanceUID; use -q TAG para otro tag (solo uno).".into());
    }
    let column = QueryColumn::single_tag_column(opts.query_keywords().as_deref())
        .map_err(|e| e.to_string())?;
    let columns = [column];
    for path in files {
        if !path.exists() {
            return Err(format!("Error: el archivo no existe: {}", path.display()));
        }
        let row = extract_row_from_path(path, &columns)
            .map_err(|e| format!("Error al abrir el archivo DICOM: {}", e))?;
        let value = row.first().map(String::as_str).unwrap_or("");
        println!("{}", value);
    }
    Ok(())
}

/// Modo -d: compara dos archivos DICOM a nivel de tags (sin PixelData). Salida en formato diff.
fn run_diff(opts: &CliOptions) -> Result<(), String> {
    let files = opts.files();
    if files.len() != 2 {
        return Err(
            "Uso con -d/--diff: dcmtk --diff <archivo1.dcm> <archivo2.dcm>\nSe requieren exactamente 2 archivos.".into(),
        );
    }
    let (p1, p2) = (&files[0], &files[1]);
    if !p1.exists() {
        return Err(format!("Error: el archivo no existe: {}", p1.display()));
    }
    if !p2.exists() {
        return Err(format!("Error: el archivo no existe: {}", p2.display()));
    }
    diff::run_diff(p1, p2)
}

/// Modo -s: muestra todos los tags del archivo. Solo se acepta un archivo.
fn run_show_tags(opts: &CliOptions) -> Result<(), String> {
    let files = opts.files();
    if files.is_empty() {
        return Err("Uso con -s: dcmtk -s <archivo.dcm>\nCon -s solo se acepta un archivo.".into());
    }
    if files.len() > 1 {
        return Err("Con -s/--show-tags solo se acepta un archivo.".into());
    }
    let path = &files[0];
    if !path.exists() {
        return Err(format!("Error: el archivo no existe: {}", path.display()));
    }
    let obj = open_file(path).map_err(|e| format!("Error al abrir el archivo DICOM: {}", e))?;
    dump_file(&obj).map_err(|e| format!("Error al volcar tags: {}", e))?;
    Ok(())
}

/// Parsea argumentos con clap. Mantiene mensajes de uso coherentes con la especificación.
fn parse_args() -> Result<CliOptions, String> {
    let opts = CliOptions::parse();
    if opts.files.is_empty() && !opts.show_tags && !opts.single_tag && !opts.diff {
        return Err("Uso: dcmtk <archivo.dcm> [archivo2.dcm ...]\nEjemplo: dcmtk myfile.dcm".into());
    }
    Ok(opts)
}

/// Resuelve la lista de columnas: desde -q si está presente, si no las por defecto.
fn resolve_columns(opts: &CliOptions) -> Result<Vec<QueryColumn>, String> {
    match opts.query_keywords() {
        Some(kw) => {
            let keywords: Vec<&str> = kw.iter().map(String::as_str).collect();
            QueryColumn::from_keywords(&keywords)
        }
        None => Ok(QueryColumn::default_columns()),
    }
}
