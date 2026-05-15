//! Capa HTTP base: inyecta `x-api-key`, verifica el status y deserializa la respuesta.

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;

use super::PostmanApiClient;

impl PostmanApiClient {
    /// Envía una petición HTTP, inyecta `x-api-key` y deserializa la respuesta en `T`.
    pub(super) async fn send<T>(&self, req: reqwest::RequestBuilder) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let resp = req
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .context("Error al conectar con Postman API")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Postman API respondió con status {status}: {body}");
        }

        resp.json::<T>()
            .await
            .context("Error al deserializar la respuesta de Postman API")
    }
}

