//! # tools::request_executor
//!
//! Tool MCP [`ExecuteRequestTool`] que localiza un request en una colección,
//! resuelve variables, aplica autenticación y lo ejecuta contra el servidor real.
//!
//! La lógica HTTP vive en [`crate::utils::executor::execute_item`],
//! compartida con [`RunCollectionLocalTool`].

use std::borrow::Cow;
use std::collections::HashMap;

use rmcp::handler::server::router::tool::{AsyncTool, ToolBase, ToolRouter};
use rmcp::ErrorData;
use rmcp::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::server::PostmanServer;
use crate::utils::errors::to_internal_err;
use crate::utils::executor::execute_item;
use crate::utils::items::{find_request_by_name, list_request_names};

pub struct ExecuteRequestTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct ExecuteRequestInput {
    pub collection_id: String,
    pub request_name: String,
    #[serde(default)]
    pub environment_id: Option<String>,
    /// Si true, devuelve TODOS los response headers. Por defecto false.
    #[serde(default)]
    pub full_headers: Option<bool>,
    /// Límite de chars del body. Por defecto 4000. Usa 0 para sin límite.
    #[serde(default)]
    pub body_limit: Option<usize>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ExecuteRequestOutput {
    pub request_name: String,
    pub method: String,
    pub url: String,
    pub request_headers_sent: String,
    pub status: u16,
    pub status_text: String,
    pub elapsed_ms: u128,
    pub response_headers: String,
    pub response_body: String,
    pub body_truncated: bool,
    pub body_total_chars: usize,
}

impl ToolBase for ExecuteRequestTool {
    type Parameter = ExecuteRequestInput;
    type Output = ExecuteRequestOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "execute_request".into() }

    fn description() -> Option<Cow<'static, str>> {
        Some(
            "Execute a specific request from a Postman collection by name. \
             Optionally resolve {{variables}} using a Postman environment. \
             Returns HTTP status, response headers, body, and elapsed time. \
             Use full_headers=true to get all response headers (default: key headers only). \
             Use body_limit=0 to disable body truncation (default limit: 4000 chars)."
                .into(),
        )
    }
}

impl AsyncTool<PostmanServer> for ExecuteRequestTool {
    async fn invoke(
        service: &PostmanServer,
        param: Self::Parameter,
    ) -> Result<Self::Output, Self::Error> {
        let collection = service.client.get_collection(&param.collection_id).await
            .map_err(to_internal_err("get_collection failed"))?;

        let found = find_request_by_name(&collection.collection.item, &param.request_name)
            .ok_or_else(|| ErrorData::invalid_params(
                format!(
                    "Request '{}' not found. Available: {}",
                    param.request_name,
                    list_request_names(&collection.collection.item).join(", ")
                ),
                None,
            ))?;

        if found.request.is_none() {
            return Err(ErrorData::invalid_params(
                format!("'{}' is a folder, not a request", param.request_name),
                None,
            ));
        }

        let mut all_vars: HashMap<String, String> = collection.collection.variable.iter()
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

        let max_body = param.body_limit.unwrap_or(4_000);
        let r = execute_item(
            service.client.http_client(),
            found,
            &all_vars,
            collection.collection.auth.as_ref(),
            max_body,
        ).await;

        if let Some(err) = r.error {
            return Err(ErrorData::internal_error(err, None));
        }

        let response_headers = if param.full_headers.unwrap_or(false) {
            format!("[full_headers] {}", r.response_headers)
        } else {
            r.response_headers
        };

        Ok(ExecuteRequestOutput {
            request_name: r.name,
            method: r.method,
            url: r.url,
            request_headers_sent: r.request_headers_sent,
            status: r.status,
            status_text: r.status_text,
            elapsed_ms: r.elapsed_ms,
            response_headers,
            response_body: r.response_body,
            body_truncated: r.body_truncated,
            body_total_chars: r.body_total_chars,
        })
    }
}

pub fn register_tools(router: ToolRouter<PostmanServer>) -> ToolRouter<PostmanServer> {
    router.with_async_tool::<ExecuteRequestTool>()
}
