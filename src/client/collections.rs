//! # client::collections
//!
//! Operaciones CRUD sobre colecciones Postman y ejecución en cloud runner.

use anyhow::Result;

use crate::models::collection::{
    GetCollectionResponse, ListCollectionsResponse, RunCollectionResponse,
};
use super::{PostmanApiClient, POSTMAN_API_BASE};

impl PostmanApiClient {
    /// Lista todas las colecciones del workspace activo.
    pub async fn list_collections(&self) -> Result<ListCollectionsResponse> {
        self.send(self.http.get(format!("{POSTMAN_API_BASE}/collections")))
            .await
    }

    /// Obtiene el detalle completo de una colección (items, carpetas, variables, auth).
    pub async fn get_collection(&self, collection_id: &str) -> Result<GetCollectionResponse> {
        self.send(
            self.http
                .get(format!("{POSTMAN_API_BASE}/collections/{collection_id}")),
        )
        .await
    }
    
    /// Crea una nueva colección vacía en el workspace activo.
    pub async fn create_collection(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut info = serde_json::json!({
            "name": name,
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        });
        if let Some(desc) = description {
            info["description"] = serde_json::json!(desc);
        }
        let body = serde_json::json!({ "collection": { "info": info, "item": [] } });

        self.send(
            self.http
                .post(format!("{POSTMAN_API_BASE}/collections"))
                .json(&body),
        )
        .await
    }

    /// Actualiza el nombre y/o descripción de una colección (PATCH parcial).
    pub async fn update_collection(
        &self,
        collection_id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<serde_json::Value> {
        let mut info = serde_json::json!({});
        if let Some(n) = name {
            info["name"] = serde_json::json!(n);
        }
        if let Some(d) = description {
            info["description"] = serde_json::json!(d);
        }
        let body = serde_json::json!({ "collection": { "info": info } });

        self.send(
            self.http
                .patch(format!("{POSTMAN_API_BASE}/collections/{collection_id}"))
                .json(&body),
        )
        .await
    }

    /// Elimina permanentemente una colección por su UID.
    pub async fn delete_collection(&self, collection_id: &str) -> Result<serde_json::Value> {
        self.send(
            self.http
                .delete(format!("{POSTMAN_API_BASE}/collections/{collection_id}")),
        )
        .await
    }

    /// Ejecuta una colección usando el Collection Runner en la nube de Postman.
    pub async fn run_collection(
        &self,
        collection_id: &str,
        environment_id: Option<&str>,
    ) -> Result<RunCollectionResponse> {
        let mut body = serde_json::json!({});
        if let Some(env_id) = environment_id {
            body["environment"] = serde_json::json!({ "id": env_id });
        }

        self.send(
            self.http
                .post(format!(
                    "{POSTMAN_API_BASE}/collections/{collection_id}/runs"
                ))
                .json(&body),
        )
        .await
    }
}

