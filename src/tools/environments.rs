//! # tools::environments
//!
//! Tools MCP de solo lectura para entornos Postman:
//! [`ListEnvironmentsTool`] y [`GetEnvironmentTool`].

use std::borrow::Cow;

use rmcp::handler::server::router::tool::{AsyncTool, ToolBase, ToolRouter};
use rmcp::ErrorData;
use rmcp::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::server::PostmanServer;
use crate::utils::errors::to_internal_err;

/// Tool MCP que lista todos los entornos del workspace.
pub struct ListEnvironmentsTool;

/// Parámetros de entrada para `list_environments` (no requiere ninguno).
#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct ListEnvironmentsInput {}

/// Respuesta de `list_environments`.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ListEnvironmentsOutput {
    /// Número total de entornos encontrados.
    pub count: usize,
    /// Lista resumida de entornos.
    pub environments: Vec<EnvironmentSummaryOutput>,
}

/// Resumen de un entorno devuelto por `list_environments`.
#[derive(Debug, Serialize, JsonSchema)]
pub struct EnvironmentSummaryOutput {
    /// UID completo del entorno.
    pub uid: String,
    /// Nombre visible del entorno.
    pub name: String,
    /// Fecha de última actualización en formato ISO 8601.
    pub updated_at: String,
}

impl ToolBase for ListEnvironmentsTool {
    type Parameter = ListEnvironmentsInput;
    type Output = ListEnvironmentsOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> {
        "list_environments".into()
    }

    fn description() -> Option<Cow<'static, str>> {
        Some("List all Postman environments in the workspace. Returns environment names, UIDs, and last update dates.".into())
    }
}

impl AsyncTool<PostmanServer> for ListEnvironmentsTool {
    async fn invoke(
        service: &PostmanServer,
        _param: Self::Parameter,
    ) -> Result<Self::Output, Self::Error> {
        let response = service.client.list_environments().await
            .map_err(to_internal_err("list_environments failed"))?;

        let environments = response
            .environments
            .into_iter()
            .map(|e| EnvironmentSummaryOutput {
                uid: e.uid,
                name: e.name,
                updated_at: e.updated_at,
            })
            .collect::<Vec<_>>();

        Ok(ListEnvironmentsOutput {
            count: environments.len(),
            environments,
        })
    }
}

/// Tool MCP que obtiene el detalle de un entorno, incluyendo todas sus variables.
pub struct GetEnvironmentTool;

/// Parámetros de entrada para `get_environment`.
#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct GetEnvironmentInput {
    /// UID o ID del entorno a recuperar.
    pub environment_id: String,
}

/// Respuesta de `get_environment`.
#[derive(Debug, Serialize, JsonSchema)]
pub struct GetEnvironmentOutput {
    /// Nombre del entorno.
    pub name: String,
    /// ID interno del entorno.
    pub id: String,
    /// Lista de variables definidas en el entorno.
    pub variables: Vec<EnvironmentVariableOutput>,
}

/// Variable de un entorno devuelta por `get_environment`.
#[derive(Debug, Serialize, JsonSchema)]
pub struct EnvironmentVariableOutput {
    /// Nombre de la variable.
    pub key: String,
    /// Valor actual de la variable.
    pub value: String,
    /// `true` si la variable está activa.
    pub enabled: bool,
    /// Tipo de variable: `"default"` o `"secret"`.
    pub variable_type: String,
}

impl ToolBase for GetEnvironmentTool {
    type Parameter = GetEnvironmentInput;
    type Output = GetEnvironmentOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> {
        "get_environment".into()
    }

    fn description() -> Option<Cow<'static, str>> {
        Some("Get details of a specific Postman environment including all its variables (key, value, enabled status, type). Provide the environment UID.".into())
    }
}

impl AsyncTool<PostmanServer> for GetEnvironmentTool {
    async fn invoke(
        service: &PostmanServer,
        param: Self::Parameter,
    ) -> Result<Self::Output, Self::Error> {
        let response = service.client.get_environment(&param.environment_id).await
            .map_err(to_internal_err("get_environment failed"))?;

        let env = response.environment;

        let variables = env
            .values
            .into_iter()
            .map(|v| EnvironmentVariableOutput {
                key: v.key,
                value: v.value,
                enabled: v.enabled,
                variable_type: v.value_type.unwrap_or_else(|| "default".into()),
            })
            .collect();

        Ok(GetEnvironmentOutput {
            name: env.name,
            id: env.id,
            variables,
        })
    }
}

/// Registra todos los tools de entornos en el [`ToolRouter`].
pub fn register_tools(router: ToolRouter<PostmanServer>) -> ToolRouter<PostmanServer> {
    router
        .with_async_tool::<ListEnvironmentsTool>()
        .with_async_tool::<GetEnvironmentTool>()
}

