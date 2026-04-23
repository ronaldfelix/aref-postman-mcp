//! # tools::collections
//!
//! Tools MCP para todas las operaciones sobre colecciones Postman:
//! lectura (`list`, `get`) y escritura (`create`, `update`, `delete`).

use std::borrow::Cow;

use rmcp::handler::server::router::tool::{AsyncTool, ToolBase, ToolRouter};
use rmcp::ErrorData;
use rmcp::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::server::PostmanServer;
use crate::utils::errors::to_internal_err;
use crate::utils::items::{count_requests, summarize_items};

use super::common::CrudOutput;


pub struct ListCollectionsTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct ListCollectionsInput {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListCollectionsOutput {
    pub count: usize,
    pub collections: Vec<CollectionSummaryOutput>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CollectionSummaryOutput {
    pub uid: String,
    pub name: String,
    pub updated_at: String,
}

impl ToolBase for ListCollectionsTool {
    type Parameter = ListCollectionsInput;
    type Output = ListCollectionsOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "list_collections".into() }

    fn description() -> Option<Cow<'static, str>> {
        Some("List all Postman collections in the workspace. Returns collection names, UIDs, and last update dates.".into())
    }
}

impl AsyncTool<PostmanServer> for ListCollectionsTool {
    async fn invoke(service: &PostmanServer, _param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let response = service.client.list_collections().await
            .map_err(to_internal_err("list_collections failed"))?;

        let collections = response.collections.into_iter()
            .map(|c| CollectionSummaryOutput { uid: c.uid, name: c.name, updated_at: c.updated_at })
            .collect::<Vec<_>>();

        Ok(ListCollectionsOutput { count: collections.len(), collections })
    }
}


pub struct GetCollectionTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct GetCollectionInput {
    /// UID o ID de la colección a recuperar.
    pub collection_id: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GetCollectionOutput {
    pub name: String,
    pub description: String,
    pub request_count: usize,
    pub structure: String,
    pub variables: Vec<CollectionVariableOutput>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CollectionVariableOutput {
    pub key: String,
    pub value: String,
    pub variable_type: String,
}

impl ToolBase for GetCollectionTool {
    type Parameter = GetCollectionInput;
    type Output = GetCollectionOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "get_collection".into() }

    fn description() -> Option<Cow<'static, str>> {
        Some("Get full details of a specific Postman collection including its requests, folders, and variables. Provide the collection UID.".into())
    }
}

impl AsyncTool<PostmanServer> for GetCollectionTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let response = service.client.get_collection(&param.collection_id).await
            .map_err(to_internal_err("get_collection failed"))?;

        let detail = &response.collection;
        let structure = summarize_items(&detail.item, 0);
        let request_count = count_requests(&detail.item);

        let variables = detail.variable.iter().map(|v| CollectionVariableOutput {
            key:           v.get("key").and_then(|k| k.as_str()).unwrap_or("").to_string(),
            value:         v.get("value").and_then(|k| k.as_str()).unwrap_or("").to_string(),
            variable_type: v.get("type").and_then(|k| k.as_str()).unwrap_or("default").to_string(),
        }).collect();

        Ok(GetCollectionOutput {
            name: detail.info.name.clone(),
            description: detail.info.description.clone().unwrap_or_else(|| "(no description)".into()),
            request_count,
            structure,
            variables,
        })
    }
}


pub struct CreateCollectionTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct CreateCollectionInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

impl ToolBase for CreateCollectionTool {
    type Parameter = CreateCollectionInput;
    type Output = CrudOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "create_collection".into() }

    fn description() -> Option<Cow<'static, str>> {
        Some("Create a new empty Postman collection with an optional description.".into())
    }
}

impl AsyncTool<PostmanServer> for CreateCollectionTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let data = service.client
            .create_collection(&param.name, param.description.as_deref())
            .await
            .map_err(to_internal_err("create_collection failed"))?;

        Ok(CrudOutput::success(data))
    }
}

pub struct UpdateCollectionTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct UpdateCollectionInput {
    pub collection_id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl ToolBase for UpdateCollectionTool {
    type Parameter = UpdateCollectionInput;
    type Output = CrudOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "update_collection".into() }

    fn description() -> Option<Cow<'static, str>> {
        Some("Update the name and/or description of an existing Postman collection.".into())
    }
}

impl AsyncTool<PostmanServer> for UpdateCollectionTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        if param.name.is_none() && param.description.is_none() {
            return Err(ErrorData::invalid_params(
                "Provide at least 'name' or 'description' to update.",
                None,
            ));
        }
        let data = service.client
            .update_collection(&param.collection_id, param.name.as_deref(), param.description.as_deref())
            .await
            .map_err(to_internal_err("update_collection failed"))?;

        Ok(CrudOutput::success(data))
    }
}

pub struct DeleteCollectionTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct DeleteCollectionInput {
    /// UID de la colección a eliminar. Esta acción no se puede deshacer.
    pub collection_id: String,
}

impl ToolBase for DeleteCollectionTool {
    type Parameter = DeleteCollectionInput;
    type Output = CrudOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "delete_collection".into() }

    fn description() -> Option<Cow<'static, str>> {
        Some("Permanently delete a Postman collection by its UID. This action cannot be undone.".into())
    }
}

impl AsyncTool<PostmanServer> for DeleteCollectionTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let data = service.client
            .delete_collection(&param.collection_id)
            .await
            .map_err(to_internal_err("delete_collection failed"))?;

        Ok(CrudOutput::success(data))
    }
}

/// Registra todos los tools de este módulo en el [`ToolRouter`].
///
/// Para añadir un nuevo tool de colecciones basta con agregar una línea aquí;
/// `server.rs` no necesita cambios.
pub fn register_tools(router: ToolRouter<PostmanServer>) -> ToolRouter<PostmanServer> {
    router
        .with_async_tool::<ListCollectionsTool>()
        .with_async_tool::<GetCollectionTool>()
        .with_async_tool::<CreateCollectionTool>()
        .with_async_tool::<UpdateCollectionTool>()
        .with_async_tool::<DeleteCollectionTool>()
}
