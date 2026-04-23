//! # client::environments
//!
//! Operaciones de lectura y actualización de entornos Postman.

use anyhow::Result;

use crate::models::environment::{GetEnvironmentResponse, ListEnvironmentsResponse};
use super::{PostmanApiClient, POSTMAN_API_BASE};

impl PostmanApiClient {
    /// Lista todos los entornos del workspace activo.
    pub async fn list_environments(&self) -> Result<ListEnvironmentsResponse> {
        self.send(self.http.get(format!("{POSTMAN_API_BASE}/environments")))
            .await
    }

    /// Obtiene el detalle de un entorno, incluyendo todas sus variables.
    pub async fn get_environment(&self, environment_id: &str) -> Result<GetEnvironmentResponse> {
        self.send(
            self.http
                .get(format!("{POSTMAN_API_BASE}/environments/{environment_id}")),
        )
        .await
    }

    /// Reemplaza el conjunto completo de variables de un entorno
    ///
    /// La API de Postman requiere enviar TODAS las variables en el PUT; el
    /// llamador es responsable de construir la lista completa ya fusionada.
    ///
    /// # Arguments
    ///
    /// * `environment_id` – UID del entorno a actualizar.
    /// * `name`           – Nombre actual del entorno (requerido por la API).
    /// * `values`         – Lista completa de variables en formato JSON.
    pub async fn update_environment_variables(
        &self,
        environment_id: &str,
        name: &str,
        values: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "environment": {
                "name": name,
                "values": values
            }
        });
        self.send(
            self.http
                .put(format!("{POSTMAN_API_BASE}/environments/{environment_id}"))
                .json(&body),
        )
        .await
    }
}

