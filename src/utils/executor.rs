//! # utils::executor
//!
//! Este módulo es el corazón del runner local: prepara la URL, resuelve variables,
//! aplica autenticación y envía la petición HTTP real. Está desacoplado de cualquier
//! tool MCP concreta para que tanto [`ExecuteRequestTool`] como [`RunCollectionLocalTool`]
//! puedan reutilizarlo

use std::collections::HashMap;
use std::time::Instant;

use reqwest::multipart;

use crate::models::collection::CollectionItem;
use crate::utils::auth::apply_auth;
use crate::utils::vars::resolve_vars;

/// Resultado de ejecutar un único request de una colección.
#[derive(Debug)]
pub struct SingleRunResult {
    /// Nombre del item en Postman.
    pub name: String,
    /// Método HTTP usado (`GET`, `POST`, etc.).
    pub method: String,
    /// URL final (con variables ya resueltas).
    pub url: String,
    /// Código de status HTTP (0 si hubo error de red).
    pub status: u16,
    /// Texto del status HTTP (`OK`, `Not Found`, etc.).
    pub status_text: String,
    /// Tiempo de respuesta en milisegundos.
    pub elapsed_ms: u128,
    /// `true` si el status es 2xx.
    pub passed: bool,
    /// Cuerpo de la respuesta (posiblemente truncado).
    pub response_body: String,
    /// `true` si el body fue truncado por `max_body`.
    pub body_truncated: bool,
    /// Tamaño total del body en caracteres (antes del truncado).
    pub body_total_chars: usize,
    /// Headers enviados en el request (para depuración).
    pub request_headers_sent: String,
    /// Headers clave de la respuesta (content-type, content-length, location…).
    pub response_headers: String,
    /// Error de red u otro error de construcción del request (`None` si fue OK).
    pub error: Option<String>,
}

/// Ejecuta un [`CollectionItem`] que debe ser un request (no una carpeta).
/// # Arguments
///
/// * `http_client`     – Cliente HTTP ya construido (sin API key de Postman).
/// * `item`            – Item con el request a ejecutar.
/// * `all_vars`        – Variables resueltas (colección + entorno, en ese orden de prioridad).
/// * `collection_auth` – Auth a nivel colección (fallback cuando el request usa `noauth`).
/// * `max_body`        – Límite de caracteres del body de respuesta. `0` = sin límite.
pub async fn execute_item(
    http_client: &reqwest::Client,
    item: &CollectionItem,
    all_vars: &HashMap<String, String>,
    collection_auth: Option<&serde_json::Value>,
    max_body: usize,
) -> SingleRunResult {
    let req_detail = match &item.request {
        Some(r) => r,
        None => return err_result(&item.name, "", "", "Item is a folder, not a request"),
    };

    let method = req_detail.method.clone().unwrap_or_else(|| "GET".into());

    let url_obj = match req_detail.url.as_ref() {
        Some(u) => u,
        None => return err_result(&item.name, &method, "", "Request has no URL"),
    };
    let raw_url = match url_obj.get("raw").and_then(|r| r.as_str()) {
        Some(u) => u,
        None => return err_result(&item.name, &method, "", "URL missing 'raw' field"),
    };

    let mut url = resolve_vars(raw_url, all_vars);

    if let Some(pvars) = url_obj.get("variable").and_then(|v| v.as_array()) {
        for pv in pvars {
            if let (Some(k), Some(v)) = (
                pv.get("key").and_then(|k| k.as_str()),
                pv.get("value").and_then(|v| v.as_str()),
            ) {
                let rv = resolve_vars(v, all_vars);
                url = url.replace(&format!(":{k}"), &rv);
                url = url.replace(&format!("{{{{{k}}}}}"), &rv);
            }
        }
    }

    let extra_query: Vec<(String, String)> = url_obj
        .get("query")
        .and_then(|q| q.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|q| !q.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false))
                .filter_map(|q| {
                    let k = q.get("key").and_then(|k| k.as_str())?;
                    let v = q.get("value").and_then(|v| v.as_str()).unwrap_or("");
                    Some((k.to_string(), resolve_vars(v, all_vars)))
                })
                .collect()
        })
        .unwrap_or_default();

    let http_method = match reqwest::Method::from_bytes(method.as_bytes()) {
        Ok(m) => m,
        Err(e) => {
            return err_result(&item.name, &method, &url, &format!("Invalid HTTP method: {e}"))
        }
    };

    let mut builder = http_client.request(http_method, &url);
    if !extra_query.is_empty() {
        builder = builder.query(&extra_query);
    }

    let mut sent_headers: Vec<String> = Vec::new();
    if let Some(headers) = &req_detail.header {
        for h in headers {
            if h.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false) {
                continue;
            }
            if let (Some(key), Some(val)) = (
                h.get("key").and_then(|v| v.as_str()),
                h.get("value").and_then(|v| v.as_str()),
            ) {
                let rv = resolve_vars(val, all_vars);
                sent_headers.push(format!("{key}: {rv}"));
                builder = builder.header(key, rv);
            }
        }
    }

    let auth_result = apply_auth(builder, req_detail.auth.as_ref(), collection_auth, all_vars);
    builder = auth_result.builder;
    if let Some((k, v)) = auth_result.apikey_query {
        builder = builder.query(&[(k, v)]);
    }

    if let Some(body) = &req_detail.body {
        builder = apply_body(builder, body, all_vars);
    }

    let start = Instant::now();
    let response = match builder.send().await {
        Ok(r) => r,
        Err(e) => {
            return err_result(&item.name, &method, &url, &format!("Network error: {e:#}"))
        }
    };
    let elapsed_ms = start.elapsed().as_millis();

    let status = response.status();
    let status_code = status.as_u16();
    let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();
    let passed = status.is_success();

    let response_ref_headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("?").to_string()))
        .collect();

    let raw_body = response.text().await.unwrap_or_else(|_| "(unreadable body)".into());
    let total_chars = raw_body.len();
    let (body_text, truncated) = if max_body > 0 && total_chars > max_body {
        (
            format!(
                "{}\n[TRUNCADO: {total_chars} chars. Usa body_limit=0 para ver completo.]",
                &raw_body[..max_body]
            ),
            true,
        )
    } else {
        (raw_body, false)
    };

    let resp_headers = response_ref_headers
        .iter()
        .filter(|(k, _)| matches!(
            k.as_str(),
            "content-type" | "content-length" | "location" |
            "x-request-id" | "x-correlation-id" | "retry-after" | "www-authenticate"
        ))
        .map(|(k, v)| format!("{k}: {v}"))
        .collect::<Vec<_>>()
        .join("\n");

    SingleRunResult {
        name: item.name.clone(),
        method,
        url,
        status: status_code,
        status_text,
        elapsed_ms,
        passed,
        response_body: body_text,
        body_truncated: truncated,
        body_total_chars: total_chars,
        request_headers_sent: sent_headers.join("\n"),
        response_headers: resp_headers,
        error: None,
    }
}

/// Aplica el body al builder según el modo configurado en Postman.
fn apply_body(
    mut builder: reqwest::RequestBuilder,
    body: &serde_json::Value,
    all_vars: &HashMap<String, String>,
) -> reqwest::RequestBuilder {
    match body.get("mode").and_then(|m| m.as_str()).unwrap_or("") {
        "raw" => {
            if let Some(raw) = body.get("raw").and_then(|r| r.as_str()) {
                let resolved = resolve_vars(raw, all_vars);
                let lang = body
                    .get("options")
                    .and_then(|o| o.get("raw"))
                    .and_then(|r| r.get("language"))
                    .and_then(|l| l.as_str())
                    .unwrap_or("");
                builder = match lang {
                    "json" => builder.header("Content-Type", "application/json").body(resolved),
                    "xml" => builder.header("Content-Type", "application/xml").body(resolved),
                    _ => builder.body(resolved),
                };
            }
        }
        "urlencoded" => {
            if let Some(fields) = body.get("urlencoded").and_then(|f| f.as_array()) {
                let params: Vec<(String, String)> = fields
                    .iter()
                    .filter(|f| !f.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false))
                    .filter_map(|f| {
                        let k = f.get("key").and_then(|k| k.as_str())?.to_string();
                        let v = resolve_vars(
                            f.get("value").and_then(|v| v.as_str()).unwrap_or(""),
                            all_vars,
                        );
                        Some((k, v))
                    })
                    .collect();
                builder = builder.form(&params);
            }
        }
        "formdata" => {
            if let Some(fields) = body.get("formdata").and_then(|f| f.as_array()) {
                let mut form = multipart::Form::new();
                for field in fields {
                    if field.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false) {
                        continue;
                    }
                    let key = field
                        .get("key")
                        .and_then(|k| k.as_str())
                        .unwrap_or("")
                        .to_string();
                    if field.get("type").and_then(|t| t.as_str()).unwrap_or("text") == "text" {
                        let val = resolve_vars(
                            field.get("value").and_then(|v| v.as_str()).unwrap_or(""),
                            all_vars,
                        );
                        form = form.text(key, val);
                    }
                }
                builder = builder.multipart(form);
            }
        }
        "graphql" => {
            if let Some(gql) = body.get("graphql") {
                let query_str = gql.get("query").and_then(|q| q.as_str()).unwrap_or("");
                let variables_v = gql.get("variables").and_then(|v| v.as_str()).unwrap_or("{}");
                let payload = format!(
                    r#"{{"query":{},"variables":{}}}"#,
                    serde_json::to_string(query_str).unwrap_or_default(),
                    variables_v
                );
                builder = builder
                    .header("Content-Type", "application/json")
                    .body(payload);
            }
        }
        _ => {}
    }
    builder
}

/// Construye un resultado de error (sin llegar a enviar el request).
fn err_result(name: &str, method: &str, url: &str, msg: &str) -> SingleRunResult {
    SingleRunResult {
        name: name.to_string(),
        method: method.to_string(),
        url: url.to_string(),
        status: 0,
        status_text: String::new(),
        elapsed_ms: 0,
        passed: false,
        response_body: String::new(),
        body_truncated: false,
        body_total_chars: 0,
        request_headers_sent: String::new(),
        response_headers: "".to_string(),
        error: Some(msg.to_string()),
    }
}




