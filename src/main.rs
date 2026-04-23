//! # postman-mcp
//!
//! Punto de entrada del servidor MCP de Postman.

mod client;
mod models;
mod server;
mod tools;
mod utils;

use anyhow::Context;
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

use crate::client::PostmanApiClient;
use crate::server::PostmanServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting Postman MCP Server v{}", env!("CARGO_PKG_VERSION"));

    let client = PostmanApiClient::new()
        .context("Failed to initialize Postman API client")?;

    let server = PostmanServer::new(client);
    let transport = (tokio::io::stdin(), tokio::io::stdout());

    tracing::info!("Postman MCP Server ready, waiting for connections via stdio...");

    let service = server.serve(transport).await
        .context("Failed to start MCP server")?;

    tokio::select! {
        result = service.waiting() => {
            if let Err(e) = result {
                tracing::warn!("MCP transport closed: {e:#}");
            } else {
                tracing::info!("MCP transport closed cleanly by client.");
            }
        }
        _ = shutdown_signal() => {
            tracing::info!("Shutdown signal received, stopping server...");
        }
    }

    tracing::info!("Postman MCP Server stopped.");

    std::process::exit(0);
}

/// Escucha señales de cierre del proceso.
///
/// En todas las plataformas espera `Ctrl+C`. En sistemas Unix también
/// intercepta `SIGTERM`, lo que permite que orquestadores de contenedores
/// (Docker, Kubernetes) terminen el proceso de forma limpia
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
    };

    #[cfg(unix)]
    {
        let mut sigterm = tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate(),
        )
        .expect("Failed to listen for SIGTERM");

        tokio::select! {
            _ = ctrl_c => {}
            _ = sigterm.recv() => {}
        }
    }

    #[cfg(not(unix))]
    ctrl_c.await;
}


