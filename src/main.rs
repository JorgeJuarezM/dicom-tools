//! DCM Tool Kit - Herramientas para uso y manejo de archivos DICOM.
//!
//! Uso: `dcmtk [opciones] <archivo.dcm> [archivo2.dcm ...]`
//! Opción `-q/--query`: columnas a mostrar (keywords separados por coma).

mod columns;
mod options;

use clap::Parser;
use columns::{extract_row_from_path, print_table, QueryColumn};
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

/// Parsea argumentos con clap. Mantiene mensajes de uso coherentes con la especificación.
fn parse_args() -> Result<CliOptions, String> {
    let opts = CliOptions::parse();
    if opts.files.is_empty() {
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
        None => Ok(QueryColumn::default_columns().to_vec()),
    }
}
