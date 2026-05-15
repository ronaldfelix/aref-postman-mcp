//! Tipos de datos para las respuestas de entornos Postman (listado y detalle de variables).

use serde::{Deserialize, Serialize};

/// Respuesta del endpoint `GET /environments`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEnvironmentsResponse {
    pub environments: Vec<EnvironmentSummary>,
}

/// Resumen de un entorno en el listado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSummary {
    /// Identificador interno.
    pub id: String,
    /// UID completo con prefijo de workspace.
    pub uid: String,
    /// Nombre visible del entorno.
    pub name: String,
    /// ID del propietario (workspace o usuario).
    #[serde(default)]
    pub owner: String,
    /// Fecha de creación (ISO 8601).
    #[serde(default, rename = "createdAt")]
    pub created_at: String,
    /// Fecha de última modificación (ISO 8601).
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

/// Detalle completo de un entorno, incluyendo todas sus variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentDetail {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub values: Vec<EnvironmentValue>,
}

/// Variable de un entorno Postman.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentValue {
    /// Nombre de la variable (usado como `{{key}}`).
    pub key: String,
    #[serde(default)]
    pub value: String,
    /// `true` si la variable está activa.
    #[serde(default)]
    pub enabled: bool,
    /// Tipo: `"default"` o `"secret"`.
    #[serde(default, rename = "type")]
    pub value_type: Option<String>,
}
