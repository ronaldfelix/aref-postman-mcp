//! # client::http
//!
//! Capa HTTP base: inyecta la API key, verifica el status y deserializa.
//!
//! El método [`PostmanApiClient::send`] es el único punto de salida de red del
//! cliente; todos los métodos de dominio lo invocan para evitar repetir la
//! lógica de error.

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;

use super::PostmanApiClient;

impl PostmanApiClient {
    /// Envía una petición HTTP, inyecta `x-api-key` y deserializa la respuesta.
    /// # Type parameters
    ///
    /// * `T` – Tipo al que se deserializa el cuerpo JSON de la respuesta.
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

