//! # utils
//!
//! Utilidades reutilizables compartidas entre los distintos módulos del servidor.
//!
//! | Módulo    | Contenido                                                        |
//! |-----------|------------------------------------------------------------------|
//! | [`errors`]| Helper `to_internal_err` para convertir `anyhow::Error` en MCP  |
//! | [`vars`]  | `resolve_vars` — interpolación de variables `{{key}}`           |
//! | [`auth`]  | `apply_auth` — construcción de cabeceras de autenticación        |
//! | [`items`] | Helpers para árboles de [`CollectionItem`]                       |

pub mod auth;
pub mod errors;
pub mod executor;
pub mod items;
pub mod vars;

