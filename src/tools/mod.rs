//! # tools
//!
//! Implementaciones de los tools MCP expuestos por el servidor de Postman.
//!
//! Cada módulo agrupa los tools relacionados con una entidad o funcionalidad y
//! expone una función `register_tools(router) -> router` que los registra en el
//! [`ToolRouter`], desacoplando completamente `server.rs` de los structs internos.
//!
//! | Módulo               | Tools                                                          |
//! |----------------------|----------------------------------------------------------------|
//! | [`collections`]      | `list/get/create/update/delete_collection`                     |
//! | [`requests`]         | `create/update/delete_request`                                 |
//! | [`environments`]     | `list/get_environment`                                         |
//! | [`request_executor`] | `execute_request`                                              |
//! | [`runner`]           | `run_collection`                                               |
//! | [`variables`]        | `set/delete_environment_variable`, `set/delete_collection_variable`, globals |
//!
//! El módulo [`common`] contiene tipos compartidos (`CrudOutput`, `HeaderEntry`,
//! `build_request_payload`) utilizados por `collections` y `requests`.

pub mod collections;
pub mod common;
pub mod environments;
pub mod request_executor;
pub mod requests;
pub mod runner;
pub mod variables;
