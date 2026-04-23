# aref-postman-mcp

Servidor **MCP (Model Context Protocol)** escrito en Rust que expone la [API REST de Postman](https://www.postman.com/postman/postman-public-workspace/) como un conjunto de *tools* que cualquier agente de IA compatible con MCP (GitHub Copilot, Claude, Cursor, etc.) puede invocar directamente desde el IDE. Permite el uso de funciones que el postman oficial no tiene como
el CRUD de requests individuales

---

## Tabla de contenidos

1. [Características — Tools disponibles](#características--tools-disponibles)
2. [Requisitos](#requisitos)
3. [Instalación y compilación](#instalación-y-compilación)
4. [Configuración del servidor MCP](#configuración-del-servidor-mcp)
5. [Arquitectura del proyecto](#arquitectura-del-proyecto)
   - [Estructura de directorios](#estructura-de-directorios)
   - [Diagrama de capas](#diagrama-de-capas)
   - [Módulo `client/`](#módulo-client)
   - [Módulo `utils/`](#módulo-utils)
   - [Módulo `tools/`](#módulo-tools)
   - [Patrón ToolRegistrar](#patrón-toolregistrar)
6. [Cómo agregar una nueva funcionalidad paso a paso](#cómo-agregar-una-nueva-funcionalidad-paso-a-paso)
7. [Autenticación soportada en `execute_request`](#autenticación-soportada-en-execute_request)
8. [Dependencias principales](#dependencias-principales)
9. [Licencia](#licencia)

---

## Características — Tools disponibles

### Colecciones

| Tool | Descripción |
|------|-------------|
| `list_collections` | Lista todas las colecciones del workspace con nombre, UID y fecha de actualización |
| `get_collection` | Obtiene el detalle completo: árbol de carpetas/requests, variables y autenticación |
| `create_collection` | Crea una nueva colección vacía con nombre y descripción opcional |
| `update_collection` | Actualiza el nombre y/o descripción de una colección existente |
| `delete_collection` | Elimina permanentemente una colección (irreversible) |

### Requests

| Tool | Descripción |
|------|-------------|
| `create_request` | Añade un nuevo request a una colección; soporta headers, body raw/urlencoded/formdata y carpetas |
| `update_request` | Reemplaza completamente un request existente dentro de una colección |
| `delete_request` | Elimina permanentemente un request (irreversible) |

### Entornos

| Tool | Descripción |
|------|-------------|
| `list_environments` | Lista todos los entornos del workspace |
| `get_environment` | Obtiene el detalle de un entorno con todas sus variables (clave, valor, tipo, estado) |

### Variables

| Tool | Descripción |
|------|-------------|
| `set_environment_variable` | Crea o actualiza una variable en un entorno Postman |
| `delete_environment_variable` | Elimina una variable de un entorno por su clave |
| `set_collection_variable` | Crea o actualiza una variable local en una colección |
| `delete_collection_variable` | Elimina una variable local de una colección |
| `list_global_variables` | Lista todas las variables globales del workspace |
| `set_global_variable` | Crea o actualiza una variable global |
| `delete_global_variable` | Elimina una variable global por su clave |

### Ejecución

| Tool | Descripción |
|------|-------------|
| `execute_request` | Ejecuta un request real resolviendo `{{variables}}`, aplicando auth heredada y devolviendo status, headers y body |
| `run_collection` | Lanza una ejecución completa en el cloud runner de Postman. **Requiere plan Enterprise.** Para cuentas gratuitas usa `run_collection_local` |
| `run_collection_local` | Ejecuta todos los requests de una colección **localmente** sin plan premium. Fuente: `collection_id` (descarga fresca desde la nube) o `collection_file` (archivo `.json` exportado). Soporta `stop_on_failure` y `body_limit` por request |

---

## Requisitos

- **Rust** ≥ 1.75 (edición 2021) — instalar desde [rustup.rs](https://rustup.rs)
- **API key de Postman** — obtener en [postman.com/settings/me/api-keys](https://www.postman.com/settings/me/api-keys)

---

## Instalación y compilación

```bash
git clone https://github.com/ronaldfelix/aref-postman-mcp.git
cd aref-postman-mcp
cargo build --release
```

El binario quedará en:
- **Linux / macOS:** `target/release/postman-mcp`
- **Windows:** `target\release\postman-mcp.exe`

---

## Configuración del servidor MCP

El servidor se comunica exclusivamente por **stdio** (stdin/stdout) siguiendo el protocolo MCP. El IDE lo lanza como proceso hijo inyectando la API key como variable de entorno.

### GitHub Copilot — JetBrains

Edita `%APPDATA%\Local\github-copilot\intellij\mcp.json` (Windows) o `~/.config/github-copilot/intellij/mcp.json` (Linux/macOS):

```json
{
  "servers": {
    "postman": {
      "type": "stdio",
      "command": "C:\\ruta\\a\\postman-mcp.exe",
      "env": {
        "POSTMAN_API_KEY": "PMAK-tu-api-key-aqui",
        "POSTMAN_WORKSPACE_ID": "uid-de-workspace"
      }
    }
  }
}
```

> **`POSTMAN_WORKSPACE_ID`** es opcional. Solo es necesario si tu cuenta tiene múltiples workspaces y las variables globales van al workspace incorrecto. Si no se define, el servidor usa automáticamente el primer workspace de tu cuenta.

### GitHub Copilot — VS Code

Edita `.vscode/mcp.json` en tu workspace o el archivo global de settings:

```json
{
  "servers": {
    "postman": {
      "type": "stdio",
      "command": "/ruta/a/postman-mcp",
      "env": {
        "POSTMAN_API_KEY": "${env:POSTMAN_API_KEY}"
      }
    }
  }
}
```

### Claude Desktop

Edita `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "postman": {
      "command": "/ruta/a/postman-mcp",
      "env": {
        "POSTMAN_API_KEY": "PMAK-tu-api-key-aqui"
      }
    }
  }
}
```

---

## Arquitectura del proyecto

### Estructura de directorios

```
src/
├── main.rs                        # Punto de entrada: logging, señales OS, bootstrap
├── server.rs                      # PostmanServer — handler MCP + ToolRegistrar
│
├── client/                        # Capa HTTP — comunicación con la API REST de Postman
│   ├── mod.rs                     # Struct PostmanApiClient, new(), http_client()
│   ├── http.rs                    # send<T>() genérico (único punto de salida de red)
│   ├── collections.rs             # list, get, create, update, delete, run
│   ├── requests.rs                # create, update, delete + helper collection_uuid()
│   ├── environments.rs            # list, get, update_environment_variables
│   └── variables.rs               # variables globales + variables de colección
│
├── models/                        # Tipos de datos — respuestas JSON de la API
│   ├── mod.rs
│   ├── collection.rs              # CollectionDetail, CollectionItem, RequestDetail, RunDetail…
│   └── environment.rs             # EnvironmentDetail, EnvironmentValue…
│
├── utils/                         # Helpers reutilizables sin dependencias de dominio
│   ├── mod.rs
│   ├── errors.rs                  # to_internal_err() / internal_err() — conversión a ErrorData
│   ├── executor.rs                # execute_item() — lógica HTTP compartida (execute_request + run_collection_local)
│   ├── vars.rs                    # resolve_vars() — interpolación {{variable}}
│   ├── auth.rs                    # apply_auth() — Basic, Bearer, ApiKey, Digest
│   └── items.rs                   # find_request_by_name(), collect_requests(), summarize_items(), count_requests()
│
└── tools/                         # Implementaciones de tools MCP
    ├── mod.rs                     # Declaración de módulos + tabla de documentación
    ├── common.rs                  # Tipos compartidos: CrudOutput, HeaderEntry, build_request_payload()
    ├── collections.rs             # list/get/create/update/delete_collection + register_tools()
    ├── requests.rs                # create/update/delete_request + register_tools()
    ├── environments.rs            # list/get_environment + register_tools()
    ├── variables.rs               # set/delete env/collection/global variables + register_tools()
    ├── request_executor.rs        # execute_request + register_tools()
    └── runner.rs                  # run_collection + run_collection_local + register_tools()
```

---

### Diagrama de capas

```
┌─────────────────────────────────────────────────────────────┐
│                        IDE / Agente IA                       │
│                    (GitHub Copilot, Claude…)                  │
└──────────────────────────┬──────────────────────────────────┘
                           │ MCP stdio
┌──────────────────────────▼──────────────────────────────────┐
│                    main.rs  +  server.rs                     │
│         Bootstrap · Señales OS · ToolRouter dispatch          │
└──────────┬──────────────────────────────────┬───────────────┘
           │                                  │
┌──────────▼──────────┐           ┌───────────▼──────────────┐
│      tools/         │           │       utils/              │
│  (una tool = un     │◄──────────│  errors · vars · auth     │
│   struct + invoke)  │           │  items                    │
└──────────┬──────────┘           └───────────────────────────┘
           │ service.client.xxx()
┌──────────▼──────────┐
│      client/        │
│  PostmanApiClient   │
│  send<T>() genérico │
└──────────┬──────────┘
           │ HTTPS
┌──────────▼──────────┐
│   api.getpostman.com │
└─────────────────────┘
```

---

### Módulo `client/`

Toda la comunicación HTTP con Postman pasa exclusivamente por `PostmanApiClient`. El método central es `send<T>()` en `client/http.rs`:

```rust
pub(super) async fn send<T: DeserializeOwned>(
    &self,
    req: reqwest::RequestBuilder,
) -> Result<T>
```

- Inyecta automáticamente el header `x-api-key`.
- Convierte errores HTTP no-2xx en `anyhow::Error` con el body incluido.
- Deserializa a cualquier tipo `T`, eliminando código repetido.

Cada dominio extiende `PostmanApiClient` con un bloque `impl` en su propio archivo. Agregar soporte para Mocks, Monitors o Workspaces es simplemente **un archivo nuevo** sin tocar los existentes.

---

### Módulo `utils/`

Helpers puros, sin estado, sin dependencias de dominio específico:

| Archivo | Función principal | Usado por |
|---------|------------------|-----------|
| `errors.rs` | `to_internal_err("ctx")` convierte `anyhow::Error` → `ErrorData` | Todos los tools |
| `executor.rs` | `execute_item()` ejecuta un `CollectionItem` HTTP completo (URL, vars, auth, body) | `execute_request`, `run_collection_local` |
| `vars.rs` | `resolve_vars(input, vars)` interpola `{{key}}` | `executor`, futuras tools |
| `auth.rs` | `apply_auth(builder, req_auth, col_auth, vars)` aplica Basic/Bearer/ApiKey | `executor` |
| `items.rs` | `find_request_by_name`, `collect_requests`, `summarize_items`, `count_requests` | `execute_request`, `run_collection_local`, `collections` |

---

### Módulo `tools/`

Cada archivo de tool sigue siempre el mismo patrón:

```rust
// 1. Struct vacío que identifica la tool
pub struct MiTool;

// 2. Struct de parámetros de entrada (deserializable + JsonSchema)
#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct MiToolInput { pub campo: String }

// 3. Struct de salida (serializable + JsonSchema)
#[derive(Debug, Serialize, JsonSchema)]
pub struct MiToolOutput { pub resultado: String }

// 4. Metadata de la tool (nombre, descripción, tipos)
impl ToolBase for MiTool {
    type Parameter = MiToolInput;
    type Output = MiToolOutput;
    type Error = ErrorData;
    fn name() -> Cow<'static, str> { "mi_tool".into() }
    fn description() -> Option<Cow<'static, str>> { Some("…".into()) }
}

// 5. Lógica de ejecución
impl AsyncTool<PostmanServer> for MiTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter)
        -> Result<Self::Output, Self::Error> { … }
}
```

---

### Patrón ToolRegistrar

Cada módulo de tools expone una función `register_tools` que encapsula el registro de sus tools:

```rust
// En tools/collections.rs
pub fn register_tools(router: ToolRouter<PostmanServer>) -> ToolRouter<PostmanServer> {
    router
        .with_async_tool::<ListCollectionsTool>()
        .with_async_tool::<GetCollectionTool>()
        // …
}
```

`server.rs` solo orquesta los registradores — **nunca conoce los structs internos de cada tool**:

```rust
fn tool_router() -> ToolRouter<Self> {
    let r = ToolRouter::new();
    let r = tools::collections::register_tools(r);
    let r = tools::requests::register_tools(r);
    let r = tools::environments::register_tools(r);
    let r = tools::request_executor::register_tools(r);
    let r = tools::runner::register_tools(r);        // run_collection + run_collection_local
    tools::variables::register_tools(r)
}
```

**Consecuencia:** añadir un nuevo módulo de tools completo requiere exactamente **una línea en `server.rs`** y **una línea en `tools/mod.rs`**, sin importar cuántas tools contenga el módulo.

---

## Cómo agregar una nueva funcionalidad paso a paso

A continuación se detalla el proceso completo para dos escenarios: agregar una tool a un **dominio existente** o crear un **dominio completamente nuevo**.

---

### Escenario A — Nueva tool en dominio existente

**Ejemplo:** agregar `duplicate_collection` al dominio de colecciones.

#### Paso 1 — Agregar el método HTTP al cliente (si la API lo requiere)

Si el endpoint no existe en `client/collections.rs`, agrégalo:

```rust
// src/client/collections.rs
impl PostmanApiClient {
    pub async fn duplicate_collection(
        &self,
        collection_id: &str,
    ) -> Result<serde_json::Value> {
        self.send(
            self.http.post(format!("{POSTMAN_API_BASE}/collections/{collection_id}/forks"))
                .json(&serde_json::json!({"label": "copy"})),
        )
        .await
    }
}
```

> Si la tool reutiliza métodos ya existentes (`get_collection`, `create_collection`, etc.), **este paso se omite**.

#### Paso 2 — Definir la tool en el módulo correspondiente

Abre `src/tools/collections.rs` y agrega al final (antes de `register_tools`):

```rust
// src/tools/collections.rs

pub struct DuplicateCollectionTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct DuplicateCollectionInput {
    pub collection_id: String,
}

impl ToolBase for DuplicateCollectionTool {
    type Parameter = DuplicateCollectionInput;
    type Output = CrudOutput;
    type Error = ErrorData;
    fn name() -> Cow<'static, str> { "duplicate_collection".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("Duplicate a Postman collection by its UID.".into())
    }
}

impl AsyncTool<PostmanServer> for DuplicateCollectionTool {
    async fn invoke(service: &PostmanServer, param: Self::Parameter)
        -> Result<Self::Output, Self::Error>
    {
        let data = service.client
            .duplicate_collection(&param.collection_id)
            .await
            .map_err(to_internal_err("duplicate_collection failed"))?;
        Ok(CrudOutput::success(data))
    }
}
```

#### Paso 3 — Registrar la tool en `register_tools`

En el mismo archivo `src/tools/collections.rs`, agrega una línea:

```rust
pub fn register_tools(router: ToolRouter<PostmanServer>) -> ToolRouter<PostmanServer> {
    router
        .with_async_tool::<ListCollectionsTool>()
        .with_async_tool::<GetCollectionTool>()
        .with_async_tool::<CreateCollectionTool>()
        .with_async_tool::<UpdateCollectionTool>()
        .with_async_tool::<DeleteCollectionTool>()
        .with_async_tool::<DuplicateCollectionTool>()  // ← nueva línea
}
```


---

### Escenario B — Nuevo dominio completo

**Ejemplo:** agregar soporte para **Mocks** de Postman (`list_mocks`, `create_mock`, `delete_mock`).

#### Paso 1 — Crear el módulo cliente `src/client/mocks.rs`

```rust
// src/client/mocks.rs
use anyhow::Result;
use super::{PostmanApiClient, POSTMAN_API_BASE};

impl PostmanApiClient {
    pub async fn list_mocks(&self) -> Result<serde_json::Value> {
        self.send(self.http.get(format!("{POSTMAN_API_BASE}/mocks"))).await
    }

    pub async fn create_mock(
        &self,
        collection_id: &str,
        name: &str,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "mock": { "collection": { "id": collection_id }, "name": name }
        });
        self.send(
            self.http.post(format!("{POSTMAN_API_BASE}/mocks")).json(&body)
        ).await
    }

    pub async fn delete_mock(&self, mock_id: &str) -> Result<serde_json::Value> {
        self.send(
            self.http.delete(format!("{POSTMAN_API_BASE}/mocks/{mock_id}"))
        ).await
    }
}
```

#### Paso 2 — Declarar el submódulo en `src/client/mod.rs`

```rust
// src/client/mod.rs — agregar una línea
mod collections;
mod environments;
mod http;
mod mocks;          // ← nueva línea
mod requests;
mod variables;
```

#### Paso 3 — Crear el módulo de tools `src/tools/mocks.rs`

```rust
// src/tools/mocks.rs
use std::borrow::Cow;
use rmcp::handler::server::router::tool::{AsyncTool, ToolBase, ToolRouter};
use rmcp::ErrorData;
use rmcp::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::server::PostmanServer;
use crate::utils::errors::to_internal_err;

// ── list_mocks ──────────────────────────────────────────────
pub struct ListMocksTool;

#[derive(Debug, Deserialize, JsonSchema, Default)]
pub struct ListMocksInput {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListMocksOutput {
    pub count: usize,
    pub mocks: Vec<serde_json::Value>,
}

impl ToolBase for ListMocksTool {
    type Parameter = ListMocksInput;
    type Output = ListMocksOutput;
    type Error = ErrorData;
    fn name() -> Cow<'static, str> { "list_mocks".into() }
    fn description() -> Option<Cow<'static, str>> {
        Some("List all Postman mocks in the workspace.".into())
    }
}

impl AsyncTool<PostmanServer> for ListMocksTool {
    async fn invoke(service: &PostmanServer, _param: Self::Parameter)
        -> Result<Self::Output, Self::Error>
    {
        let resp = service.client.list_mocks().await
            .map_err(to_internal_err("list_mocks failed"))?;

        let mocks = resp.pointer("/mocks")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(ListMocksOutput { count: mocks.len(), mocks })
    }
}

// ── Registrador ─────────────────────────────────────────────
pub fn register_tools(router: ToolRouter<PostmanServer>) -> ToolRouter<PostmanServer> {
    router
        .with_async_tool::<ListMocksTool>()
        // .with_async_tool::<CreateMockTool>()
        // .with_async_tool::<DeleteMockTool>()
}
```

#### Paso 4 — Declarar el módulo en `src/tools/mod.rs`

```rust
// src/tools/mod.rs — agregar una línea
pub mod collections;
pub mod common;
pub mod environments;
pub mod mocks;          // ← nueva línea
pub mod request_executor;
pub mod requests;
pub mod variables;
```

#### Paso 5 — Registrar el dominio en `src/server.rs`

```rust
// src/server.rs — agregar una línea en tool_router()
fn tool_router() -> ToolRouter<Self> {
    let r = ToolRouter::new();
    let r = tools::collections::register_tools(r);
    let r = tools::requests::register_tools(r);
    let r = tools::environments::register_tools(r);
    let r = tools::mocks::register_tools(r);       // ← nueva línea
    let r = tools::request_executor::register_tools(r);
    let r = tools::runner::register_tools(r);
    tools::variables::register_tools(r)
}
```

---

### Resumen de archivos por escenario

| Escenario | Archivos nuevos | Archivos modificados | Líneas en `server.rs` |
|-----------|----------------|---------------------|----------------------|
| Tool en dominio existente | 0 | 1–2 | 0 |
| Dominio nuevo completo | 2 | 2 | 1 |

---

## Autenticación soportada en `execute_request`

El tool resuelve automáticamente la autenticación con herencia **request → colección**:

| Tipo | Comportamiento |
|------|----------------|
| `basic` | Cabecera `Authorization: Basic <base64(user:pass)>` |
| `bearer` | Cabecera `Authorization: Bearer <token>` |
| `apikey` | Header o query param según configuración de Postman |
| `digest` | Fallback a Basic (Digest real requiere challenge previo) |
| `noauth` | Hereda la auth configurada a nivel colección |

La lógica de autenticación vive en `src/utils/auth.rs`, invocada desde `src/utils/executor.rs`, y es reutilizable por cualquier tool futura.

---

## Dependencias principales

| Crate | Versión | Propósito |
|-------|---------|-----------|
| `rmcp` | 1.3 | Framework MCP: server, transport stdio, ToolRouter |
| `tokio` | 1 | Runtime asíncrono (`full` features) |
| `reqwest` | 0.12 | Cliente HTTP con soporte JSON y multipart |
| `serde` / `serde_json` | 1 | Serialización/deserialización JSON |
| `schemars` | — | Generación de JSON Schema para los parámetros de tools |
| `anyhow` | 1 | Manejo de errores ergonómico con contexto |
| `base64` | 0.22 | Codificación de credenciales Basic Auth |
| `tracing` / `tracing-subscriber` | 0.1 / 0.3 | Logging estructurado a stderr |

---

By @Aref 2026
