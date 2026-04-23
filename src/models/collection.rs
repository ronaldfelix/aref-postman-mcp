use serde::{Deserialize, Serialize};

// ─── Respuesta de listado de colecciones ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCollectionsResponse {
    pub collections: Vec<CollectionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSummary {
    pub id: String,
    pub uid: String,
    pub name: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default, rename = "createdAt")]
    pub created_at: String,
    #[serde(default, rename = "updatedAt")]
    pub updated_at: String,
    #[serde(default)]
    pub fork: Option<serde_json::Value>,
}

// ─── Respuesta de detalle de colección ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCollectionResponse {
    pub collection: CollectionDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionDetail {
    pub info: CollectionInfo,
    #[serde(default)]
    pub item: Vec<CollectionItem>,
    #[serde(default)]
    pub variable: Vec<serde_json::Value>,
    #[serde(default)]
    pub auth: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    #[serde(rename = "_postman_id")]
    pub postman_id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub schema: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionItem {
    pub name: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub item: Option<Vec<CollectionItem>>,
    #[serde(default)]
    pub request: Option<RequestDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDetail {
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub url: Option<serde_json::Value>,
    #[serde(default)]
    pub header: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub body: Option<serde_json::Value>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub auth: Option<serde_json::Value>,
}

// ─── Respuesta de ejecución de colección ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCollectionResponse {
    pub run: RunDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDetail {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub info: Option<serde_json::Value>,
    #[serde(default)]
    pub stats: Option<serde_json::Value>,
    #[serde(default)]
    pub executions: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub failures: Option<Vec<serde_json::Value>>,
}

