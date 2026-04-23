//! # client
//!
//! Cliente HTTP modular para la [API REST de Postman](https://www.postman.com/postman/postman-public-workspace/).
//!
//! [`PostmanApiClient`] es el punto de entrada único. Cada dominio extiende el
//! struct en su propio submódulo mediante bloques `impl` adicionales, lo que
//! permite añadir nuevos dominios (mocks, monitors, workspaces…)
//! | Módulo            | Operaciones                                                  |
//! |-------------------|--------------------------------------------------------------|
//! | [`http`]          | `send<T>` genérico + constante base URL                      |
//! | [`collections`]   | list, get, create, update, delete, run                       |
//! | [`requests`]      | create, update, delete (+ helper `collection_uuid`)          |
//! | [`environments`]  | list, get, update variables de entorno                       |
//! | [`variables`]     | variables globales + variables de colección                  |

mod http;
mod collections;
mod requests;
mod environments;
mod variables;

use anyhow::{Context, Result};
use reqwest::Client;

/// URL base de la API REST de Postman.
///
/// Accesible a los submódulos a través de `super::POSTMAN_API_BASE`.
pub(self) const POSTMAN_API_BASE: &str = "https://api.getpostman.com";

/// Cliente HTTP para la API de Postman.
///
/// Construir con [`PostmanApiClient::new`], que lee la variable de entorno
/// `POSTMAN_API_KEY`.  La instancia es barata de clonar (`Arc` interno de
/// `reqwest::Client`).
#[derive(Debug, Clone)]
pub struct PostmanApiClient {
    pub(super) http: Client,
    pub(super) api_key: String,
}

impl PostmanApiClient {
    /// Expone el cliente HTTP interno para que los tools puedan realizar
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

