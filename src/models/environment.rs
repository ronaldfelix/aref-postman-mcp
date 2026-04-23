//! # models::environment
//!
//! Tipos de datos que representan las respuestas de la API de Postman
//! relacionadas con entornos: listado y detalle de variables.

use serde::{Deserialize, Serialize};

/// Respuesta del endpoint `GET /environments`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEnvironmentsResponse {
    pub environments: Vec<EnvironmentSummary>,
}

/// Resumen de un entorno tal como lo devuelve el listado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSummary {
    /// Identificador interno del entorno.
    pub id: String,
    /// UID completo con prefijo de workspace (`{8hex}-{uuid}`).
    pub uid: String,
    /// Nombre visible del entorno.
    pub name: String,
    /// ID del propietario (workspace o usuario).
    #[serde(default)]
    pub owner: String,
    /// Fecha de creación en formato ISO 8601.
    #[serde(default, rename = "createdAt")]
    pub created_at: String,
    /// Fecha de última actualización en formato ISO 8601.
    #[serde(default, rename = "updatedAt")]
    pub updated_at: String,
    /// Indica si el entorno es público en el workspace.
    #[serde(default, rename = "isPublic")]
    pub is_public: bool,
}

/// Respuesta del endpoint `GET /environments/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEnvironmentResponse {
    pub environment: EnvironmentDetail,
}

/// Detalle completo de un entorno Postman, incluyendo todas sus variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentDetail {
    /// Identificador interno del entorno.
    pub id: String,
    /// Nombre visible del entorno.
    pub name: String,
    /// Lista de variables definidas en el entorno.
    #[serde(default)]
    pub values: Vec<EnvironmentValue>,
}

/// Variable de un entorno Postman.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentValue {
    /// Nombre de la variable (usado como `{{key}}` en las plantillas).
    pub key: String,
    /// Valor actual de la variable.
    #[serde(default)]
    pub value: String,
    /// Indica si la variable está activa (`true`) o deshabilitada (`false`).
    #[serde(default)]
    pub enabled: bool,
    /// Tipo de variable: `"default"` o `"secret"`.
    #[serde(default, rename = "type")]
    pub value_type: Option<String>,
}
