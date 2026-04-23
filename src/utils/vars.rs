//! # utils::vars
//!
//! Resolución de variables Postman con la sintaxis `{{nombre}}`.
//!
//! Las variables se aplican en orden de prioridad (colección → entorno → global)
//! ya que el llamador es responsable de construir el mapa con esa precedencia.

use std::collections::HashMap;

/// Reemplaza todas las ocurrencias de `{{key}}` en `input` por su valor
/// correspondiente en `vars`. Las referencias sin valor se dejan sin cambiar.
///
/// # Arguments
///
/// * `input` – Cadena de texto con posibles referencias `{{key}}`.
/// * `vars`  – Mapa `clave → valor` ya resuelto por el llamador.
pub fn resolve_vars(input: &str, vars: &HashMap<String, String>) -> String {
    let mut result = input.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{key}}}}}"), value);
    }
    result
}

