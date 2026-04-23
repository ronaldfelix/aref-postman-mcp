//! # utils::errors
//!
//! Helpers para convertir `anyhow::Error` en [`rmcp::ErrorData`] sin boilerplate.
//!
//! ## Uso típico
//!
//! ```rust
//! use crate::utils::errors::to_internal_err;
//!
//! service.client.list_collections()
//!     .await
//!     .map_err(to_internal_err("list_collections failed"))?;
//! ```

use rmcp::ErrorData;

/// Devuelve un closure que convierte cualquier `Display` en un
/// [`ErrorData::internal_error`], prefijando el mensaje con `context`.
///
/// # Arguments
///
/// * `context` – Etiqueta fija que identifica la operación fallida.
pub fn to_internal_err(context: &'static str) -> impl Fn(anyhow::Error) -> ErrorData {
    move |e| ErrorData::internal_error(format!("{context}: {e:#}"), None)
}

/// Crea directamente un [`ErrorData::internal_error`] a partir de cualquier
/// valor [`Display`] (útil cuando no se usa `map_err`).
#[allow(dead_code)]
pub fn internal_err(msg: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(msg.to_string(), None)
}

