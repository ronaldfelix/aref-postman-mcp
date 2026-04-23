//! # tools::runner
//!
//! Tools MCP de ejecución de colecciones Postman:
//!
//! - [`RunCollectionTool`] — usa el Cloud Runner de Postman (requiere plan Enterprise).
//! - [`RunCollectionLocalTool`] — ejecuta la colección **localmente** sin API premium,
//!   descargando la colección desde Postman API o leyendo un archivo `.json` exportado.

use std::borrow::Cow;
use std::collections::HashMap;

use rmcp::handler::server::router::tool::{AsyncTool, ToolBase, ToolRouter};
use rmcp::ErrorData;
use rmcp::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::collection::{CollectionDetail, GetCollectionResponse};
use crate::server::PostmanServer;
use crate::utils::errors::to_internal_err;
use crate::utils::executor::execute_item;
use crate::utils::items::collect_requests;

pub struct RunCollectionTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct RunCollectionInput {
    /// UID o ID de la colección a ejecutar.
    pub collection_id: String,
    /// UID opcional del entorno a inyectar en la ejecución.
    #[serde(default)]
    pub environment_id: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RunCollectionOutput {
    pub run_id: String,
    pub status: String,
    pub stats: String,
    pub failures: String,
}

impl ToolBase for RunCollectionTool {
    type Parameter = RunCollectionInput;
    type Output = RunCollectionOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "run_collection".into() }

    fn description() -> Option<Cow<'static, str>> {
        Some("Run a Postman collection using the Postman Cloud Runner. Optionally specify an environment UID. Returns run status, stats, and any failures. NOTE: requires Enterprise plan. For free accounts use run_collection_local instead.".into())
    }
}

impl AsyncTool<PostmanServer> for RunCollectionTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let response = service.client
            .run_collection(&param.collection_id, param.environment_id.as_deref())
            .await
            .map_err(to_internal_err("run_collection failed"))?;

        let run = &response.run;
        let failures = match &run.failures {
            Some(f) if !f.is_empty() => {
                format!("{} failure(s):\n{}", f.len(), serde_json::to_string_pretty(f).unwrap_or_default())
            }
            _ => "No failures".into(),
        };
        let stats = match &run.stats {
            Some(s) => serde_json::to_string_pretty(s).unwrap_or_default(),
            None => "No stats available".into(),
        };

        Ok(RunCollectionOutput {
            run_id: run.id.clone().unwrap_or_else(|| "N/A".into()),
            status: run.status.clone().unwrap_or_else(|| "unknown".into()),
            stats,
            failures,
        })
    }
}

/// Tool MCP que ejecuta todos los requests de una colección **localmente**,
/// Fuentes de colección (mutuamente excluyentes, en orden de prioridad):
/// 1. `collection_id`   — descarga la colección desde la API de Postman.
/// 2. `collection_file` — lee un archivo `.json` exportado desde Postman.
///
/// El archivo `.json` puede ser en formato de API `{"collection":{...}}` o en
/// formato de exportación directo `{"info":{...},"item":[...]}`.
pub struct RunCollectionLocalTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct RunCollectionLocalInput {
    /// UID de la colección en Postman. La colección se descarga y ejecuta localmente.
    /// Usar este campo O `collection_file`, no ambos a la vez.
    #[serde(default)]
    pub collection_id: Option<String>,

    /// Ruta absoluta al archivo `.json` exportado desde Postman.
    /// Soporta formato API `{"collection":{...}}` y formato de exportación `{"info":{...},"item":[...]}`.
    #[serde(default)]
    pub collection_file: Option<String>,

    /// UID del entorno Postman para resolver `{{variables}}`.
    #[serde(default)]
    pub environment_id: Option<String>,

    /// Detener la ejecución al primer request con status no 2xx. Por defecto `false`.
    #[serde(default)]
    pub stop_on_failure: Option<bool>,

    /// Límite de caracteres del body de respuesta por request. Por defecto `2000`. `0` = sin límite.
    #[serde(default)]
    pub body_limit: Option<usize>,
}

/// Resumen de la ejecución local de toda la colección.
#[derive(Debug, Serialize, JsonSchema)]
pub struct RunCollectionLocalOutput {
    /// Nombre de la colección ejecutada.
    pub collection_name: String,
    /// Total de requests ejecutados.
    pub total: usize,
    /// Requests con status 2xx.
    pub passed: usize,
    /// Requests con status no 2xx o con error de red.
    pub failed: usize,
    /// Tiempo total de ejecución en milisegundos.
    pub total_elapsed_ms: u128,
    /// Resultados individuales de cada request.
    pub results: Vec<RequestRunResult>,
}

/// Resultado de un único request dentro de la ejecución local.
#[derive(Debug, Serialize, JsonSchema)]
pub struct RequestRunResult {
    pub name: String,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub status_text: String,
    pub elapsed_ms: u128,
    pub passed: bool,
    pub response_body: String,
    pub body_truncated: bool,
    /// `null` si no hubo error de red.
    pub error: Option<String>,
}

impl ToolBase for RunCollectionLocalTool {
    type Parameter = RunCollectionLocalInput;
    type Output = RunCollectionLocalOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "run_collection_local".into() }

    fn description() -> Option<Cow<'static, str>> {
        Some(
            "Execute all requests in a Postman collection LOCALLY (no Enterprise plan needed). \
             Resolves {{variables}}, applies auth, and runs each request in sequence. \
             Source: use 'collection_id' to download from Postman API, or 'collection_file' \
             for a local exported .json file. Returns per-request status, body and timing."
                .into(),
        )
    }
}

impl AsyncTool<PostmanServer> for RunCollectionLocalTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let collection_detail = load_collection(service, &param).await?;

        let mut all_vars: HashMap<String, String> = collection_detail
            .variable
            .iter()
            .filter_map(|v| {
                let key = v.get("key").and_then(|k| k.as_str())?;
                let val = v.get("value").and_then(|v| v.as_str()).unwrap_or("");
                Some((key.to_string(), val.to_string()))
            })
            .collect();

        if let Some(env_id) = &param.environment_id {
            let env = service.client.get_environment(env_id).await
                .map_err(to_internal_err("get_environment failed"))?;
            for v in env.environment.values.into_iter().filter(|v| v.enabled) {
                all_vars.insert(v.key, v.value);
            }
        }

        let items = collect_requests(&collection_detail.item);
        if items.is_empty() {
            return Ok(RunCollectionLocalOutput {
                collection_name: collection_detail.info.name.clone(),
                total: 0,
                passed: 0,
                failed: 0,
                total_elapsed_ms: 0,
                results: vec![],
            });
        }

        let max_body = param.body_limit.unwrap_or(2_000);
        let stop_on_fail = param.stop_on_failure.unwrap_or(false);
        let http = service.client.http_client();
        let col_auth = collection_detail.auth.as_ref();

        let mut results: Vec<RequestRunResult> = Vec::new();
        let mut total_elapsed: u128 = 0;
        let mut passed = 0usize;
        let mut failed = 0usize;

        for item in &items {
            let r = execute_item(http, item, &all_vars, col_auth, max_body).await;

            total_elapsed += r.elapsed_ms;
            if r.passed && r.error.is_none() {
                passed += 1;
            } else {
                failed += 1;
            }

            let stop = stop_on_fail && !r.passed;

            results.push(RequestRunResult {
                name: r.name,
                method: r.method,
                url: r.url,
                status: r.status,
                status_text: r.status_text,
                elapsed_ms: r.elapsed_ms,
                passed: r.passed,
                response_body: r.response_body,
                body_truncated: r.body_truncated,
                error: r.error,
            });

            if stop {
                break;
            }
        }

        Ok(RunCollectionLocalOutput {
            collection_name: collection_detail.info.name.clone(),
            total: results.len(),
            passed,
            failed,
            total_elapsed_ms: total_elapsed,
            results,
        })
    }
}

async fn load_collection(
    service: &PostmanServer,
    param: &RunCollectionLocalInput,
) -> Result<CollectionDetail, ErrorData> {
    if let Some(col_id) = &param.collection_id {
        return service.client.get_collection(col_id).await
            .map(|r| r.collection)
            .map_err(to_internal_err("get_collection failed"));
    }

    if let Some(file_path) = &param.collection_file {
        let json = std::fs::read_to_string(file_path).map_err(|e| {
            ErrorData::invalid_params(
                format!("Cannot read file '{}': {e}", file_path),
                None,
            )
        })?;

        if let Ok(api_resp) = serde_json::from_str::<GetCollectionResponse>(&json) {
            return Ok(api_resp.collection);
        }

        return serde_json::from_str::<CollectionDetail>(&json).map_err(|e| {
            ErrorData::invalid_params(
                format!("Invalid Postman collection JSON in '{}': {e}", file_path),
                None,
            )
        });
    }

    Err(ErrorData::invalid_params(
        "Provide either 'collection_id' (Postman API) or 'collection_file' (local .json path).",
        None,
    ))
}

pub fn register_tools(router: ToolRouter<PostmanServer>) -> ToolRouter<PostmanServer> {
    router
        .with_async_tool::<RunCollectionTool>()
        .with_async_tool::<RunCollectionLocalTool>()
}
