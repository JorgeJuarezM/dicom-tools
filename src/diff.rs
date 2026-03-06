//! Comparación de dos archivos DICOM a nivel de tags (sin PixelData). Salida en formato diff.

use dicom::core::dictionary::DataDictionary;
use dicom::core::Tag;
use dicom::dictionary_std::data_element::StandardDataDictionary;
use dicom::object::open_file;
use dicom::object::DefaultDicomObject;
use std::collections::BTreeMap;
use std::path::Path;

/// Tag PixelData (7FE0,0010) — se omite en la comparación.
const PIXEL_DATA: Tag = Tag(0x7FE0, 0x0010);

/// Recolecta todos los tags del dataset (excepto PixelData) con su valor como string.
fn collect_tag_values(obj: &DefaultDicomObject) -> BTreeMap<Tag, String> {
    let mut map = BTreeMap::new();
    for elem in obj.iter() {
        if elem.header().tag == PIXEL_DATA {
            continue;
        }
        let value = elem
            .to_str()
            .map(|cow| cow.into_owned())
            .unwrap_or_else(|_| "<...>".to_string());
        map.insert(elem.header().tag, value);
    }
    map
}

/// Devuelve el nombre del tag (keyword) para mostrar, o el tag en formato (gggg,eeee).
fn tag_display_name(tag: Tag) -> String {
    let dict = StandardDataDictionary;
    dict.by_tag(tag)
        .map(|e: &_| e.alias.to_string())
        .unwrap_or_else(|| format!("({:04X},{:04X})", tag.0, tag.1))
}

/// Imprime la diferencia entre dos conjuntos de tags en formato diff (unificado).
fn print_diff(
    map1: &BTreeMap<Tag, String>,
    map2: &BTreeMap<Tag, String>,
    name1: &str,
    name2: &str,
) {
    println!("--- {}", name1);
    println!("+++ {}", name2);

    let all_tags: std::collections::BTreeSet<_> =
        map1.keys().chain(map2.keys()).cloned().collect();

    for tag in all_tags {
        let v1 = map1.get(&tag);
        let v2 = map2.get(&tag);
        let name = tag_display_name(tag);
        let tag_fmt = format!("({:04X},{:04X})", tag.0, tag.1);

        match (v1, v2) {
            (Some(a), Some(b)) if a == b => {
                // Mismo valor: no mostramos nada (o podríamos mostrar "  " para contexto)
            }
            (Some(a), Some(b)) => {
                println!("- {} {}: {}", tag_fmt, name, a);
                println!("+ {} {}: {}", tag_fmt, name, b);
            }
            (Some(a), None) => {
                println!("- {} {}: {}", tag_fmt, name, a);
            }
            (None, Some(b)) => {
                println!("+ {} {}: {}", tag_fmt, name, b);
            }
            (None, None) => {}
        }
    }
}

/// Compara dos archivos DICOM (sin PixelData) e imprime el resultado en formato diff.
pub fn run_diff(path1: &Path, path2: &Path) -> Result<(), String> {
    let obj1 = open_file(path1).map_err(|e| format!("Error al abrir {}: {}", path1.display(), e))?;
    let obj2 = open_file(path2).map_err(|e| format!("Error al abrir {}: {}", path2.display(), e))?;

    let name1 = path1.display().to_string();
    let name2 = path2.display().to_string();

    let map1 = collect_tag_values(&obj1);
    let map2 = collect_tag_values(&obj2);

    print_diff(&map1, &map2, &name1, &name2);
    Ok(())
}
