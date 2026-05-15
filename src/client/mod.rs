//! Cliente HTTP modular para la API REST de Postman. `PostmanApiClient` es el punto de entrada único.

mod http;
mod collections;
mod requests;
mod environments;
mod variables;

use anyhow::{Context, Result};
use reqwest::Client;

/// URL base de la API REST de Postman.
pub(self) const POSTMAN_API_BASE: &str = "https://api.getpostman.com";

/// Cliente HTTP para la API de Postman. Construir con [`PostmanApiClient::new`] (lee `POSTMAN_API_KEY`).
#[derive(Debug, Clone)]
pub struct PostmanApiClient {
    pub(super) http: Client,
    pub(super) api_key: String,
}

impl PostmanApiClient {
    /// Expone el cliente HTTP interno (sin API key) para realizar requests directos.
    pub fn http_client(&self) -> &Client {
        &self.http
    }

    /// Crea un nuevo cliente leyendo `POSTMAN_API_KEY` del entorno.
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("POSTMAN_API_KEY").context(
            "Variable de entorno POSTMAN_API_KEY no encontrada. \
             Configúrala con tu API key de Postman.",
        )?;

        let http = Client::builder()
            .user_agent(concat!("postman-mcp/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("Error al crear el cliente HTTP")?;

        Ok(Self { http, api_key })
    }
}

