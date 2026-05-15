//! Helpers para convertir `anyhow::Error` en [`rmcp::ErrorData`] sin boilerplate.

use rmcp::ErrorData;

/// Devuelve un closure que convierte cualquier error en [`ErrorData::internal_error`].
/// * `context` – Etiqueta que identifica la operación fallida.
pub fn to_internal_err(context: &'static str) -> impl Fn(anyhow::Error) -> ErrorData {
    move |e| ErrorData::internal_error(format!("{context}: {e:#}"), None)
}

/// Crea directamente un [`ErrorData::internal_error`] a partir de cualquier valor `Display`.
#[allow(dead_code)]
pub fn internal_err(msg: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(msg.to_string(), None)
}
