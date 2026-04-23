//! # client::requests
//!
//! Operaciones CRUD sobre requests dentro de una colección Postman.

use anyhow::Result;

use super::{PostmanApiClient, POSTMAN_API_BASE};

impl PostmanApiClient {
    /// Crea un nuevo request dentro de una colección.
    ///
    /// # Arguments
    ///
    /// * `collection_id`  – UID de la colección destino.
    /// * `request_body`   – Payload JSON con la definición del request.
    /// * `folder_id`      – UID opcional de la carpeta donde se ubicará.
    pub async fn create_request(
        &self,
        collection_id: &str,
        request_body: serde_json::Value,
        folder_id: Option<&str>,
    ) -> Result<serde_json::Value> {
        let cid = collection_uuid(collection_id);
        let mut url = format!("{POSTMAN_API_BASE}/collections/{cid}/requests");
        if let Some(fid) = folder_id {
            url = format!("{url}?folder={fid}");
        }

        self.send(self.http.post(url).json(&request_body)).await
    }

    /// Reemplaza completamente un request existente dentro de una colección.
    pub async fn update_request(
        &self,
        collection_id: &str,
        request_id: &str,
        request_body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let cid = collection_uuid(collection_id);
        self.send(
            self.http
                .put(format!(
                    "{POSTMAN_API_BASE}/collections/{cid}/requests/{request_id}"
                ))
                .json(&request_body),
        )
        .await
    }

    /// Elimina permanentemente un request de una colección.
    pub async fn delete_request(
        &self,
        collection_id: &str,
        request_id: &str,
    ) -> Result<serde_json::Value> {
        let cid = collection_uuid(collection_id);
        self.send(
            self.http.delete(format!(
                "{POSTMAN_API_BASE}/collections/{cid}/requests/{request_id}"
            )),
        )
        .await
    }
}

/// Extrae el UUID puro de una colección a partir de su UID completo.
///
/// La API de Postman devuelve UIDs con el formato `{8hex}-{collection-uuid}`.
/// Los endpoints de mutación de requests esperan únicamente el UUID de colección.
fn collection_uuid(uid: &str) -> &str {
    if let Some(pos) = uid.find('-') {
        if pos == 8 && uid[..pos].chars().all(|c| c.is_ascii_hexdigit()) {
            return &uid[pos + 1..];
        }
    }
    uid
}

