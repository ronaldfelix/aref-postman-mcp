//! # tools::variables
//!
//! Tools MCP para gestión de variables locales (colección/entorno) y globales.
//!
//! | Tool                             | Scope      | Operación                                   |
//! |----------------------------------|------------|---------------------------------------------|
//! | [`SetEnvironmentVariableTool`]   | Entorno    | Crea o actualiza una variable               |
//! | [`DeleteEnvironmentVariableTool`]| Entorno    | Elimina una variable por clave              |
//! | [`SetCollectionVariableTool`]    | Colección  | Crea o actualiza una variable local         |
//! | [`DeleteCollectionVariableTool`] | Colección  | Elimina una variable local por clave        |
//! | [`ListGlobalVariablesTool`]      | Global     | Lista todas las variables globales          |
//! | [`SetGlobalVariableTool`]        | Global     | Crea o actualiza una variable global        |
//! | [`DeleteGlobalVariableTool`]     | Global     | Elimina una variable global por clave       |

use std::borrow::Cow;

use rmcp::handler::server::router::tool::{AsyncTool, ToolBase, ToolRouter};
use rmcp::ErrorData;
use rmcp::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::server::PostmanServer;
use crate::utils::errors::to_internal_err;

/// Respuesta devuelta por todas las operaciones de escritura de variables.
#[derive(Debug, Serialize, JsonSchema)]
pub struct VariableOutput {
    pub ok: bool,
    pub message: String,
    /// Identificador del recurso afectado (environment UID, collection UID o `"globals"`).
    pub resource_id: String,
    pub key: String,
    /// Acción ejecutada: `"set"` o `"delete"`.
    pub action: String,
}

impl VariableOutput {
    fn set(resource_id: impl Into<String>, key: impl Into<String>) -> Self {
        Self { ok: true, message: "OK".into(), resource_id: resource_id.into(), key: key.into(), action: "set".into() }
    }
    fn delete(resource_id: impl Into<String>, key: impl Into<String>) -> Self {
        Self { ok: true, message: "OK".into(), resource_id: resource_id.into(), key: key.into(), action: "delete".into() }
    }
}

pub struct SetEnvironmentVariableTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct SetEnvironmentVariableInput {
    /// UID del entorno a modificar.
    pub environment_id: String,
    /// Nombre de la variable (`{{key}}`).
    pub key: String,
    /// Nuevo valor.
    pub value: String,
    /// `"default"` o `"secret"`. Por defecto `"default"`.
    #[serde(default)]
    pub variable_type: Option<String>,
    /// Activar/desactivar la variable. Por defecto `true`.
    #[serde(default)]
    pub enabled: Option<bool>,
}

impl ToolBase for SetEnvironmentVariableTool {
    type Parameter = SetEnvironmentVariableInput;
    type Output = VariableOutput;
    type Error = ErrorData;
    fn name() -> Cow<'static, str> { "set_environment_variable".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Create or update a variable inside a Postman environment (local scope). If the key exists it is replaced; otherwise added.".into())
    }
}

impl AsyncTool<PostmanServer> for SetEnvironmentVariableTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let env = service.client.get_environment(&param.environment_id).await
            .map_err(to_internal_err("get_environment failed"))?
            .environment;

        let env_name = env.name.clone();
        let var_type = param.variable_type.as_deref().unwrap_or("default").to_string();
        let enabled = param.enabled.unwrap_or(true);

        let mut found = false;
        let mut new_values: Vec<serde_json::Value> = env.values.into_iter().map(|v| {
            if v.key == param.key {
                found = true;
                serde_json::json!({"key": param.key, "value": param.value, "enabled": enabled, "type": var_type})
            } else {
                serde_json::json!({"key": v.key, "value": v.value, "enabled": v.enabled, "type": v.value_type.unwrap_or_else(|| "default".into())})
            }
        }).collect();

        if !found {
            new_values.push(serde_json::json!({"key": param.key, "value": param.value, "enabled": enabled, "type": var_type}));
        }

        service.client.update_environment_variables(&param.environment_id, &env_name, serde_json::Value::Array(new_values)).await
            .map_err(to_internal_err("update_environment_variables failed"))?;

        Ok(VariableOutput::set(param.environment_id, param.key))
    }
}

pub struct DeleteEnvironmentVariableTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct DeleteEnvironmentVariableInput {
    /// UID del entorno.
    pub environment_id: String,
    /// Clave de la variable a eliminar.
    pub key: String,
}

impl ToolBase for DeleteEnvironmentVariableTool {
    type Parameter = DeleteEnvironmentVariableInput;
    type Output = VariableOutput;
    type Error = ErrorData;
    fn name() -> Cow<'static, str> { "delete_environment_variable".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Delete a variable from a Postman environment by its key name.".into())
    }
}

impl AsyncTool<PostmanServer> for DeleteEnvironmentVariableTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let env = service.client.get_environment(&param.environment_id).await
            .map_err(to_internal_err("get_environment failed"))?
            .environment;

        let env_name = env.name.clone();
        let new_values: Vec<serde_json::Value> = env.values.into_iter()
            .filter(|v| v.key != param.key)
            .map(|v| serde_json::json!({"key": v.key, "value": v.value, "enabled": v.enabled, "type": v.value_type.unwrap_or_else(|| "default".into())}))
            .collect();

        service.client.update_environment_variables(&param.environment_id, &env_name, serde_json::Value::Array(new_values)).await
            .map_err(to_internal_err("update_environment_variables failed"))?;

        Ok(VariableOutput::delete(param.environment_id, param.key))
    }
}

pub struct SetCollectionVariableTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct SetCollectionVariableInput {
    /// UID de la colección a modificar.
    pub collection_id: String,
    /// Nombre de la variable (`{{key}}`).
    pub key: String,
    /// Valor de la variable.
    pub value: String,
    /// `"default"` o `"secret"`. Por defecto `"default"`.
    #[serde(default)]
    pub variable_type: Option<String>,
}

impl ToolBase for SetCollectionVariableTool {
    type Parameter = SetCollectionVariableInput;
    type Output = VariableOutput;
    type Error = ErrorData;
    fn name() -> Cow<'static, str> { "set_collection_variable".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Create or update a LOCAL variable inside a Postman collection. Accessible as {{key}} in all requests of the collection. For shared variables use set_environment_variable; for workspace-wide use set_global_variable.".into())
    }
}

impl AsyncTool<PostmanServer> for SetCollectionVariableTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let col = service.client.get_collection(&param.collection_id).await
            .map_err(to_internal_err("get_collection failed"))?
            .collection;

        let col_name = col.info.name.clone();
        let col_postman_id = col.info.postman_id.clone();
        let var_type = param.variable_type.as_deref().unwrap_or("default").to_string();

        let mut found = false;
        let mut new_vars: Vec<serde_json::Value> = col.variable.into_iter().map(|v| {
            if v.get("key").and_then(|k| k.as_str()) == Some(&param.key) {
                found = true;
                serde_json::json!({"key": param.key, "value": param.value, "type": var_type})
            } else { v }
        }).collect();

        if !found {
            new_vars.push(serde_json::json!({"key": param.key, "value": param.value, "type": var_type}));
        }

        service.client.update_collection_variables(&param.collection_id, &col_name, &col_postman_id, serde_json::Value::Array(new_vars)).await
            .map_err(to_internal_err("update_collection_variables failed"))?;

        Ok(VariableOutput::set(param.collection_id, param.key))
    }
}

pub struct DeleteCollectionVariableTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct DeleteCollectionVariableInput {
    /// UID de la colección.
    pub collection_id: String,
    /// Clave de la variable a eliminar.
    pub key: String,
}

impl ToolBase for DeleteCollectionVariableTool {
    type Parameter = DeleteCollectionVariableInput;
    type Output = VariableOutput;
    type Error = ErrorData;
    fn name() -> Cow<'static, str> { "delete_collection_variable".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Delete a LOCAL variable from a Postman collection by its key name.".into())
    }
}

impl AsyncTool<PostmanServer> for DeleteCollectionVariableTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let col = service.client.get_collection(&param.collection_id).await
            .map_err(to_internal_err("get_collection failed"))?
            .collection;

        let col_name = col.info.name.clone();
        let col_postman_id = col.info.postman_id.clone();

        let new_vars: Vec<serde_json::Value> = col.variable.into_iter()
            .filter(|v| v.get("key").and_then(|k| k.as_str()) != Some(&param.key))
            .collect();

        service.client.update_collection_variables(&param.collection_id, &col_name, &col_postman_id, serde_json::Value::Array(new_vars)).await
            .map_err(to_internal_err("update_collection_variables failed"))?;

        Ok(VariableOutput::delete(param.collection_id, param.key))
    }
}

pub struct ListGlobalVariablesTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct ListGlobalVariablesInput {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListGlobalVariablesOutput {
    pub count: usize,
    pub variables: Vec<GlobalVariableItem>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GlobalVariableItem {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    /// `"default"` o `"secret"`.
    pub variable_type: String,
}

impl ToolBase for ListGlobalVariablesTool {
    type Parameter = ListGlobalVariablesInput;
    type Output = ListGlobalVariablesOutput;
    type Error = ErrorData;
    fn name() -> Cow<'static, str> { "list_global_variables".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("List all GLOBAL variables of the Postman workspace. Global variables are accessible from every collection and environment as {{key}}.".into())
    }
}

impl AsyncTool<PostmanServer> for ListGlobalVariablesTool {
    async fn invoke(service: &PostmanServer, _param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let resp = service.client.get_globals().await
            .map_err(to_internal_err("get_globals failed"))?;

        let values = resp.pointer("/values")
            .and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let variables = values.into_iter().map(|v| GlobalVariableItem {
            key:           v.get("key").and_then(|k| k.as_str()).unwrap_or("").to_string(),
            value:         v.get("value").and_then(|k| k.as_str()).unwrap_or("").to_string(),
            enabled:       v.get("enabled").and_then(|k| k.as_bool()).unwrap_or(true),
            variable_type: v.get("type").and_then(|k| k.as_str()).unwrap_or("default").to_string(),
        }).collect::<Vec<_>>();

        Ok(ListGlobalVariablesOutput { count: variables.len(), variables })
    }
}

pub struct SetGlobalVariableTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct SetGlobalVariableInput {
    /// Nombre de la variable global.
    pub key: String,
    /// Valor de la variable.
    pub value: String,
    /// `"default"` o `"secret"`. Por defecto `"default"`.
    #[serde(default)]
    pub variable_type: Option<String>,
    /// Activar/desactivar. Por defecto `true`.
    #[serde(default)]
    pub enabled: Option<bool>,
}

impl ToolBase for SetGlobalVariableTool {
    type Parameter = SetGlobalVariableInput;
    type Output = VariableOutput;
    type Error = ErrorData;
    fn name() -> Cow<'static, str> { "set_global_variable".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Create or update a GLOBAL variable in the Postman workspace. Global variables are accessible as {{key}} from any collection or environment.".into())
    }
}

impl AsyncTool<PostmanServer> for SetGlobalVariableTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let resp = service.client.get_globals().await
            .map_err(to_internal_err("get_globals failed"))?;

        let current: Vec<serde_json::Value> = resp.pointer("/values")
            .and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let var_type = param.variable_type.as_deref().unwrap_or("default").to_string();
        let enabled = param.enabled.unwrap_or(true);

        let mut found = false;
        let mut new_values: Vec<serde_json::Value> = current.into_iter().map(|v| {
            if v.get("key").and_then(|k| k.as_str()) == Some(&param.key) {
                found = true;
                serde_json::json!({"key": param.key, "value": param.value, "enabled": enabled, "type": var_type})
            } else { v }
        }).collect();

        if !found {
            new_values.push(serde_json::json!({"key": param.key, "value": param.value, "enabled": enabled, "type": var_type}));
        }

        service.client.update_globals(serde_json::Value::Array(new_values)).await
            .map_err(to_internal_err("update_globals failed"))?;

        Ok(VariableOutput::set("globals", param.key))
    }
}

pub struct DeleteGlobalVariableTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct DeleteGlobalVariableInput {
    /// Clave de la variable global a eliminar.
    pub key: String,
}

impl ToolBase for DeleteGlobalVariableTool {
    type Parameter = DeleteGlobalVariableInput;
    type Output = VariableOutput;
    type Error = ErrorData;
    fn name() -> Cow<'static, str> { "delete_global_variable".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Delete a GLOBAL variable from the Postman workspace by its key name.".into())
    }
}

impl AsyncTool<PostmanServer> for DeleteGlobalVariableTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let resp = service.client.get_globals().await
            .map_err(to_internal_err("get_globals failed"))?;

        let current: Vec<serde_json::Value> = resp.pointer("/values")
            .and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let new_values: Vec<serde_json::Value> = current.into_iter()
            .filter(|v| v.get("key").and_then(|k| k.as_str()) != Some(&param.key))
            .collect();

        service.client.update_globals(serde_json::Value::Array(new_values)).await
            .map_err(to_internal_err("update_globals failed"))?;

        Ok(VariableOutput::delete("globals", param.key))
    }
}

/// Registra todos los tools de variables en el [`ToolRouter`].
pub fn register_tools(router: ToolRouter<PostmanServer>) -> ToolRouter<PostmanServer> {
    router
        .with_async_tool::<SetEnvironmentVariableTool>()
        .with_async_tool::<DeleteEnvironmentVariableTool>()
        .with_async_tool::<SetCollectionVariableTool>()
        .with_async_tool::<DeleteCollectionVariableTool>()
        .with_async_tool::<ListGlobalVariablesTool>()
        .with_async_tool::<SetGlobalVariableTool>()
        .with_async_tool::<DeleteGlobalVariableTool>()
}
