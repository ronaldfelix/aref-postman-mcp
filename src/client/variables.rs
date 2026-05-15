//! Variables globales de workspace y variables locales de colección.

use anyhow::{Result, anyhow};

use super::{PostmanApiClient, POSTMAN_API_BASE};

impl PostmanApiClient {
    /// Actualiza el array `variable` de una colección (PATCH). Requiere la lista completa de variables.
    ///
    /// * `collection_id` – UID de la colección a actualizar.
    /// * `name`          – Nombre actual de la colección (requerido en `info`).
    /// * `_postman_id`   – ID interno de Postman (reservado).
    /// * `variables`     – Lista completa de variables en formato JSON.
    pub async fn update_collection_variables(
        &self,
        collection_id: &str,
        name: &str,
        _postman_id: &str,
        variables: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "collection": {
                "info": { "name": name },
                "variable": variables
            }
        });
        self.send(
            self.http
                .patch(format!("{POSTMAN_API_BASE}/collections/{collection_id}"))
                .json(&body),
        )
        .await
    }

    /// Obtiene todas las variables globales del workspace activo.
    pub async fn get_globals(&self) -> Result<serde_json::Value> {
        let wid = self.workspace_id().await?;
        self.send(
            self.http
                .get(format!("{POSTMAN_API_BASE}/workspaces/{wid}/global-variables")),
        )
        .await
    }

    /// Reemplaza el conjunto completo de variables globales del workspace (PUT total).
    pub async fn update_globals(&self, values: serde_json::Value) -> Result<serde_json::Value> {
        let wid = self.workspace_id().await?;
        let body = serde_json::json!({ "values": values });
        self.send(
            self.http
                .put(format!(
                    "{POSTMAN_API_BASE}/workspaces/{wid}/global-variables"
                ))
                .json(&body),
        )
        .await
    }

    /// Obtiene el UID del workspace activo.
    /// Usa `POSTMAN_WORKSPACE_ID` si está definida; si no, toma el primer workspace de la API.
    async fn workspace_id(&self) -> Result<String> {
        if let Ok(id) = std::env::var("POSTMAN_WORKSPACE_ID") {
            if !id.is_empty() {
                return Ok(id);
            }
        }

        let resp: serde_json::Value = self
            .send(self.http.get(format!("{POSTMAN_API_BASE}/workspaces")))
            .await?;

        resp.pointer("/workspaces/0/id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("No se encontró ningún workspace. Verifica tu POSTMAN_API_KEY."))
    }
}

