//! # utils::items
//!
//! Helpers para navegar árboles de [`CollectionItem`] de Postman.
//!
//! Las funciones de este módulo operan únicamente sobre los modelos del dominio
//! y son reutilizables por cualquier tool que necesite recorrer una colección.

use crate::models::collection::CollectionItem;

/// Busca recursivamente un request por nombre (insensible a mayúsculas).
///
/// Devuelve `None` si no existe ningún item con ese nombre y tipo request.
pub fn find_request_by_name<'a>(
    items: &'a [CollectionItem],
    name: &str,
) -> Option<&'a CollectionItem> {
    let name_lower = name.to_lowercase();
    for item in items {
        if item.name.to_lowercase() == name_lower && item.request.is_some() {
            return Some(item);
        }
        if let Some(sub) = &item.item {
            if let Some(found) = find_request_by_name(sub, name) {
                return Some(found);
            }
        }
    }
    None
}

/// Recopila recursivamente los nombres de todos los requests de una colección.
pub fn list_request_names(items: &[CollectionItem]) -> Vec<String> {
    let mut names = Vec::new();
    for item in items {
        if item.request.is_some() {
            names.push(item.name.clone());
        }
        if let Some(sub) = &item.item {
            names.extend(list_request_names(sub));
        }
    }
    names
}

/// Genera una representación textual indentada del árbol de items de una colección.
///
/// Las carpetas se prefijan con `[folder]` y los requests con `[METHOD]`.
///
/// # Arguments
///
/// * `items` – Slice de items del nivel actual.
/// * `depth` – Profundidad de indentación (cada nivel añade dos espacios).
pub fn summarize_items(items: &[CollectionItem], depth: usize) -> String {
    let indent = "  ".repeat(depth);
    items
        .iter()
        .map(|item| {
            if let Some(sub_items) = &item.item {
                let folder_line = format!("{indent}[folder] {}", item.name);
                let children = summarize_items(sub_items, depth + 1);
                if children.is_empty() {
                    folder_line
                } else {
                    format!("{folder_line}\n{children}")
                }
            } else if let Some(req) = &item.request {
                let method = req.method.as_deref().unwrap_or("???");
                format!("{indent}  [{method}] {}", item.name)
            } else {
                format!("{indent}  - {}", item.name)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Recopila recursivamente todos los items que son requests (excluye carpetas),
/// respetando el orden en que aparecen en la colección.
///
/// Útil para el runner local que necesita ejecutar TODOS los requests en orden.
pub fn collect_requests(items: &[CollectionItem]) -> Vec<&CollectionItem> {
    let mut result = Vec::new();
    for item in items {
        if item.request.is_some() {
            result.push(item);
        } else if let Some(sub) = &item.item {
            result.extend(collect_requests(sub));
        }
    }
    result
}

/// Recopila recursivamente el total de requests en una colección (sin contar carpetas).
pub fn count_requests(items: &[CollectionItem]) -> usize {
    items
        .iter()
        .map(|item| {
            if let Some(sub) = &item.item {
                count_requests(sub)
            } else if item.request.is_some() {
                1
            } else {
                0
            }
        })
        .sum()
}

