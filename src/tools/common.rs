//! # tools::common
//!
//! Tipos y helpers compartidos por todos los tools de escritura (CRUD).

use rmcp::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Respuesta genérica para todas las operaciones de escritura.
///
/// Solo expone los campos útiles para el LLM, descartando metadatos internos
/// de Postman (`lastRevision`, `owner`, `helperAttributes`).
#[derive(Debug, Serialize, JsonSchema)]
pub struct CrudOutput {
    /// `true` si la operación fue exitosa.
    pub ok: bool,
    /// Mensaje de resultado (`"OK"` en caso de éxito).
    pub message: String,
    /// Identificador del recurso creado o modificado.
    pub id: String,
    /// Nombre del recurso (si aplica).
    pub name: String,
    /// Acción ejecutada: `"create"`, `"update"`, `"delete"`, etc.
    pub action: String,
}

impl CrudOutput {
    /// Extrae solo los campos relevantes de la respuesta JSON de Postman.
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
///
/// La API v1 (`api.getpostman.com/collections/{id}/requests`) usa un formato
/// **plano** donde los headers van en el campo `"headers"` (plural) como un
/// **string** con líneas `"Key: Value\n"`.
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

