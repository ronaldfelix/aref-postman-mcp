//! # tools::requests
//!
//! Tools MCP para operaciones CRUD sobre requests dentro de una colección Postman:
//! [`CreateRequestTool`], [`UpdateRequestTool`] y [`DeleteRequestTool`].

use std::borrow::Cow;

use rmcp::handler::server::router::tool::{AsyncTool, ToolBase, ToolRouter};
use rmcp::ErrorData;
use rmcp::schemars;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::server::PostmanServer;
use crate::utils::errors::to_internal_err;

use super::common::{CrudOutput, HeaderEntry, build_request_payload};

pub struct CreateRequestTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct CreateRequestInput {
    /// UID de la colección destino.
    pub collection_id: String,
    /// Nombre del nuevo request.
    pub name: String,
    /// Método HTTP (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`).
    pub method: String,
    /// URL completa del request.
    pub url: String,
    /// Lista opcional de headers HTTP.
    #[serde(default)]
    pub headers: Option<Vec<HeaderEntry>>,
    /// Modo de body: `"raw"`, `"urlencoded"`, `"formdata"`. Omitir para sin body.
    #[serde(default)]
    pub body_mode: Option<String>,
    /// Contenido del body en modo `"raw"`.
    #[serde(default)]
    pub body_raw: Option<String>,
    /// Lenguaje del body raw: `"json"`, `"xml"`, `"text"`. Por defecto `"text"`.
    #[serde(default)]
    pub body_language: Option<String>,
    /// UID de la carpeta destino dentro de la colección (opcional).
    #[serde(default)]
    pub folder_id: Option<String>,
    /// Descripción opcional del request.
    #[serde(default)]
    pub description: Option<String>,
}

impl ToolBase for CreateRequestTool {
    type Parameter = CreateRequestInput;
    type Output = CrudOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "create_request".into() }

    fn description() -> Option<Cow<'static, str>> {
        Some(
            "Add a new request to an existing Postman collection. \
             Supports headers, raw/urlencoded/formdata body, and folder placement."
                .into(),
        )
    }
}

impl AsyncTool<PostmanServer> for CreateRequestTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let body = build_request_payload(
            &param.name,
            &param.method,
            &param.url,
            param.headers.as_deref(),
            param.body_mode.as_deref(),
            param.body_raw.as_deref(),
            param.body_language.as_deref(),
            param.description.as_deref(),
        );

        let data = service.client
            .create_request(&param.collection_id, body, param.folder_id.as_deref())
            .await
            .map_err(to_internal_err("create_request failed"))?;

        Ok(CrudOutput::success(data))
    }
}

pub struct UpdateRequestTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct UpdateRequestInput {
    /// UID de la colección que contiene el request.
    pub collection_id: String,
    /// UID del request a actualizar.
    pub request_id: String,
    /// Nuevo nombre del request.
    pub name: String,
    /// Método HTTP (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS`).
    pub method: String,
    /// URL completa del request.
    pub url: String,
    /// Lista de headers HTTP (reemplaza los existentes).
    #[serde(default)]
    pub headers: Option<Vec<HeaderEntry>>,
    /// Modo de body: `"raw"`, `"urlencoded"`, `"formdata"`. Omitir para sin body.
    #[serde(default)]
    pub body_mode: Option<String>,
    /// Contenido del body en modo `"raw"`.
    #[serde(default)]
    pub body_raw: Option<String>,
    /// Lenguaje del body raw: `"json"`, `"xml"`, `"text"`.
    #[serde(default)]
    pub body_language: Option<String>,
    /// Descripción opcional del request.
    #[serde(default)]
    pub description: Option<String>,
}

impl ToolBase for UpdateRequestTool {
    type Parameter = UpdateRequestInput;
    type Output = CrudOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "update_request".into() }

    fn description() -> Option<Cow<'static, str>> {
        Some(
            "Update an existing request inside a Postman collection. \
             Replaces name, method, URL, headers, and body. \
             Use get_collection first to retrieve the request UID."
                .into(),
        )
    }
}

impl AsyncTool<PostmanServer> for UpdateRequestTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let body = build_request_payload(
            &param.name,
            &param.method,
            &param.url,
            param.headers.as_deref(),
            param.body_mode.as_deref(),
            param.body_raw.as_deref(),
            param.body_language.as_deref(),
            param.description.as_deref(),
        );

        let data = service.client
            .update_request(&param.collection_id, &param.request_id, body)
            .await
            .map_err(to_internal_err("update_request failed"))?;

        Ok(CrudOutput::success(data))
    }
}

pub struct DeleteRequestTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct DeleteRequestInput {
    /// UID de la colección que contiene el request.
    pub collection_id: String,
    /// UID del request a eliminar. Esta acción no se puede deshacer.
    pub request_id: String,
}

impl ToolBase for DeleteRequestTool {
    type Parameter = DeleteRequestInput;
    type Output = CrudOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> { "delete_request".into() }

    fn description() -> Option<Cow<'static, str>> {
        Some(
            "Permanently delete a request from a Postman collection by its UID. \
             Use get_collection first to retrieve the request UID."
                .into(),
        )
    }
}

impl AsyncTool<PostmanServer> for DeleteRequestTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter) -> Result<Self::Output, Self::Error> {
        let data = service.client
            .delete_request(&param.collection_id, &param.request_id)
            .await
            .map_err(to_internal_err("delete_request failed"))?;

        Ok(CrudOutput::success(data))
    }
}

/// Registra todos los tools de requests en el [`ToolRouter`].
pub fn register_tools(router: ToolRouter<PostmanServer>) -> ToolRouter<PostmanServer> {
    router
        .with_async_tool::<CreateRequestTool>()
        .with_async_tool::<UpdateRequestTool>()
        .with_async_tool::<DeleteRequestTool>()
}

