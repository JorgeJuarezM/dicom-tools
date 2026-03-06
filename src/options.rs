//! Opciones de línea de comandos y configuración abstracta de la aplicación.

use clap::Parser;
use std::path::PathBuf;

/// Opciones de la aplicación: modificadores y argumentos posicionales.
/// Diseñado para extenderse con nuevos modificadores sin cambiar la firma.
#[derive(Debug, Parser)]
#[command(name = "dcmtk", about = "Herramientas para archivos DICOM")]
pub struct CliOptions {
    /// Columnas a mostrar (keywords DICOM separados por coma). Si no se especifica, se usan las columnas por defecto.
    #[arg(short = 'q', long = "query", value_name = "COLUMNAS")]
    pub query: Option<String>,

    /// Uno o más archivos DICOM a procesar.
    #[arg(value_name = "archivo.dcm")]
    pub files: Vec<PathBuf>,
}

impl CliOptions {
    /// Rutas de archivos a procesar (en orden).
    pub fn files(&self) -> &[PathBuf] {
        &self.files
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
