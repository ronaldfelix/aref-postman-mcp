//! # utils::auth
//!
//! Aplicación de esquemas de autenticación Postman sobre un [`reqwest::RequestBuilder`].
//!
//! Soporta: `basic`, `bearer`, `apikey` (header / query), `digest` (tratado como basic).

use std::collections::HashMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use reqwest::RequestBuilder;

use crate::utils::vars::resolve_vars;

/// Resultado de aplicar la autenticación sobre un builder.
pub struct AuthResult {
    /// Builder con cabeceras/query de auth ya aplicados.
    pub builder: RequestBuilder,
    /// Parámetro de query extra para `apikey` con `in=query`.
    pub apikey_query: Option<(String, String)>,
}

/// Aplica el objeto `auth` de Postman sobre `builder`, resolviendo variables con `vars`.
///
/// Si `auth` es `None` o su tipo es `"noauth"`, devuelve el builder sin modificar.
///
/// # Arguments
///
/// * `builder`          – Builder HTTP en construcción.
/// * `auth`             – Objeto de autenticación extraído de la colección o del request.
/// * `collection_auth`  – Auth a nivel colección (fallback si el request usa `"noauth"`).
/// * `vars`             – Mapa de variables ya resueltas.
pub fn apply_auth(
    mut builder: RequestBuilder,
    request_auth: Option<&serde_json::Value>,
    collection_auth: Option<&serde_json::Value>,
    vars: &HashMap<String, String>,
) -> AuthResult {
    let effective_auth = match request_auth {
        Some(a) if a.get("type").and_then(|t| t.as_str()) == Some("noauth") => collection_auth,
        Some(a) => Some(a),
        None => collection_auth,
    };

    let get_val = |list: &serde_json::Value, key: &str| -> String {
        list.as_array()
            .and_then(|arr| {
                arr.iter()
                    .find(|i| i.get("key").and_then(|k| k.as_str()) == Some(key))
            })
            .and_then(|i| i.get("value").and_then(|v| v.as_str()))
            .map(|s| resolve_vars(s, vars))
            .unwrap_or_default()
    };

    let mut apikey_query: Option<(String, String)> = None;

    if let Some(auth) = effective_auth {
        match auth.get("type").and_then(|t| t.as_str()).unwrap_or("") {
            "basic" | "digest" => {
                let key = if auth.get("type").and_then(|t| t.as_str()) == Some("basic") {
                    "basic"
                } else {
                    "digest"
                };
                if let Some(list) = auth.get(key) {
                    let user = get_val(list, "username");
                    let pass = get_val(list, "password");
                    let encoded = BASE64.encode(format!("{user}:{pass}"));
                    builder = builder.header("Authorization", format!("Basic {encoded}"));
                }
            }
            "bearer" => {
                if let Some(list) = auth.get("bearer") {
                    let token = get_val(list, "token");
                    builder = builder.header("Authorization", format!("Bearer {token}"));
                }
            }
            "apikey" => {
                if let Some(list) = auth.get("apikey") {
                    let key_name = get_val(list, "key");
                    let key_val = get_val(list, "value");
                    let location = list
                        .as_array()
                        .and_then(|arr| {
                            arr.iter().find(|i| {
                                i.get("key").and_then(|k| k.as_str()) == Some("in")
                            })
                        })
                        .and_then(|i| i.get("value").and_then(|v| v.as_str()))
                        .unwrap_or("header");

                    if location == "query" {
                        apikey_query = Some((key_name, key_val));
                    } else {
                        builder = builder.header(key_name, key_val);
                    }
                }
            }
            _ => {}
        }
    }

    AuthResult { builder, apikey_query }
}

