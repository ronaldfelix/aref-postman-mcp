# aref-postman-mcp

Servidor **MCP (Model Context Protocol)** escrito  100% en Rust que expone la [API REST de Postman](https://www.postman.com/postman/postman-public-workspace/) como un conjunto de *tools* que cualquier agente de IA compatible con MCP (GitHub Copilot, Claude, Cursor, etc.) puede invocar directamente desde el IDE

PROS:
- Incluye herramientas básicas del mcp de postman por defecto pero tambien ...
- Ejecutar colecciones ilimitadas sin depender de un plan premium de Postman.
- Ejecutar requests individuales 

---

## Tabla de contenidos

1. [Características — Tools disponibles](#características--tools-disponibles)
2. [Requisitos](#requisitos)
3. [Instalación y compilación](#instalación-y-compilación)
4. [Configuración del servidor MCP](#configuración-del-servidor-mcp)
5. [Arquitectura del proyecto](#arquitectura-del-proyecto)
6. [Dependencias principales](#dependencias-principales)

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
