//! Tipos y helpers compartidos por todos los tools de escritura (CRUD).

use rmcp::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Respuesta genérica para operaciones de escritura. Descarta metadatos internos de Postman.
#[derive(Debug, Serialize, JsonSchema)]
pub struct CrudOutput {
    /// `true` si la operación fue exitosa.
    pub ok: bool,
    /// Mensaje de resultado (`"OK"` en caso de éxito).
    pub message: String,
    /// ID del recurso creado o modificado.
    pub id: String,
    /// Nombre del recurso (si aplica).
    pub name: String,
    /// Acción ejecutada: `"create"`, `"update"`, `"delete"`, etc.
    pub action: String,
}

impl CrudOutput {
    /// Extrae los campos relevantes (id, name, action) de la respuesta JSON de Postman.
    pub fn success(data: serde_json::Value) -> Self {
        let id = data
            .pointer("/collection/id")
            .or_else(|| data.pointer("/model_id"))
            .or_else(|| data.pointer("/data/id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let name = data
            .pointer("/collection/info/name")
            .or_else(|| data.pointer("/data/name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let action = data
            .pointer("/meta/action")
            .and_then(|v| v.as_str())
            .unwrap_or("ok")
            .to_string();

        Self {
            ok: true,
            message: "OK".into(),
            id,
            name,
            action,
        }
    }
}

/// Par clave-valor que representa un header HTTP en los inputs CRUD.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct HeaderEntry {
    /// Nombre del header (ej. `Content-Type`).
    pub key: String,
    /// Valor del header (ej. `application/json`).
    pub value: String,
}

/// Construye el payload JSON de un request para la API REST de Postman v1.
/// Los headers se codifican como string `"Key: Value\n"` (formato plano requerido por la API).
///
/// * `name`           – Nombre del request.
/// * `method`         – Método HTTP en mayúsculas.
/// * `url`            – URL completa.
/// * `headers`        – Headers HTTP opcionales.
/// * `body_mode`      – Modo del body: `"raw"`, `"urlencoded"`, `"formdata"` o `None`.
/// * `body_raw`       – Contenido del body en modo `"raw"`.
/// * `body_language`  – Lenguaje del body raw: `"json"`, `"xml"`, `"text"`.
/// * `description`    – Descripción opcional del request.
pub fn build_request_payload(
    name: &str,
    method: &str,
    url: &str,
    headers: Option<&[HeaderEntry]>,
    body_mode: Option<&str>,
    body_raw: Option<&str>,
    body_language: Option<&str>,
    description: Option<&str>,
) -> serde_json::Value {
    let headers_str: String = headers
        .unwrap_or(&[])
        .iter()
        .map(|h| format!("{}: {}\n", h.key, h.value))
        .collect();

    let body_json: serde_json::Value = match body_mode {
        Some("raw") => {
            let lang = body_language.unwrap_or("text");
            serde_json::json!({
                "mode": "raw",
                "raw": body_raw.unwrap_or(""),
                "options": { "raw": { "language": lang } }
            })
        }
        Some("urlencoded") => serde_json::json!({ "mode": "urlencoded", "urlencoded": [] }),
        Some("formdata") => serde_json::json!({ "mode": "formdata", "formdata": [] }),
        _ => serde_json::Value::Null,
    };

    let mut payload = serde_json::json!({
        "name": name,
        "method": method.to_uppercase(),
        "url": url,
        "headers": headers_str,
    });

    if let Some(desc) = description {
        payload["description"] = serde_json::json!(desc);
    }
    if !body_json.is_null() {
        payload["body"] = body_json;
    }

    payload
}
