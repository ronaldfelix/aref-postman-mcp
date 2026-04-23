//! # server
//!
//! Define [`PostmanServer`], el handler MCP que expone todos los tools
//! ## Cómo agregar un nuevo dominio de tools
//!
//! 1. Crear el módulo en `tools/` con los structs de tool y una función
//!    `pub fn register_tools(router: ToolRouter<PostmanServer>) -> ToolRouter<PostmanServer>`.
//! 2. Añadir `pub mod nuevo_modulo;` en `tools/mod.rs`.
//! 3. Llamar `tools::nuevo_modulo::register_tools(r)` en [`PostmanServer::tool_router`].
//!
//! `server.rs` no necesita conocer los structs internos de cada tool.

use std::future::Future;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, InitializeResult,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities,
};
use rmcp::service::{MaybeSendFuture, RequestContext, RoleServer};
use rmcp::{ErrorData, ServerHandler};

use crate::client::PostmanApiClient;
use crate::tools;

/// Servidor MCP de Postman.
///
/// Mantiene una referencia al cliente HTTP de Postman y un [`ToolRouter`]
/// que despacha las llamadas entrantes al tool correspondiente.
pub struct PostmanServer {
    pub client: PostmanApiClient,
    tool_router: ToolRouter<Self>,
}

impl PostmanServer {
    /// Crea un nuevo servidor MCP registrando todos los tools disponibles.
    pub fn new(client: PostmanApiClient) -> Self {
        let tool_router = Self::tool_router();
        Self { client, tool_router }
    }

    /// Construye el [`ToolRouter`] delegando el registro a cada módulo de tools.
    ///
    /// Cada módulo expone `register_tools(router) -> router` siguiendo el
    /// patrón Open/Closed: agregar un nuevo dominio solo requiere una línea aquí.
    fn tool_router() -> ToolRouter<Self> {
        let r = ToolRouter::new();
        let r = tools::collections::register_tools(r);
        let r = tools::requests::register_tools(r);
        let r = tools::environments::register_tools(r);
        let r = tools::request_executor::register_tools(r);
        let r = tools::runner::register_tools(r);
        tools::variables::register_tools(r)
    }
}

impl ServerHandler for PostmanServer {
    fn get_info(&self) -> InitializeResult {
        InitializeResult::new(
            ServerCapabilities::builder().enable_tools().build(),
        )
        .with_server_info(Implementation::new("postman-mcp", env!("CARGO_PKG_VERSION")))
        .with_instructions(
            "Postman MCP Server — interact with your Postman workspace: \
             list/get/create/update/delete collections and requests, \
             list/get environments, set/delete environment and collection variables, \
             execute individual requests, and run collections. \
             Requires POSTMAN_API_KEY environment variable."
                .to_string(),
        )
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + MaybeSendFuture + '_ {
        let tools = self.tool_router.list_all();
        std::future::ready(Ok(ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        }))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, ErrorData>> + MaybeSendFuture + '_ {
        let tool_context = rmcp::handler::server::tool::ToolCallContext::new(
            self, request, context,
        );
        self.tool_router.call(tool_context)
    }

    fn get_tool(&self, name: &str) -> Option<rmcp::model::Tool> {
        self.tool_router.get(name).cloned()
    }
}
