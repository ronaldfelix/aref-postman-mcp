//! Resolución de variables Postman con sintaxis `{{nombre}}`.

use std::collections::HashMap;

/// Reemplaza todas las ocurrencias de `{{key}}` en `input` por su valor en `vars`.
/// Las referencias sin valor se dejan sin cambiar.
///
/// * `input` – Texto con posibles referencias `{{key}}`.
/// * `vars`  – Mapa `clave → valor` ya resuelto por el llamador.
pub fn resolve_vars(input: &str, vars: &HashMap<String, String>) -> String {
    let mut result = input.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{key}}}}}"), value);
    }
    result
}
