use axum::{
    extract::{State},
    response::sse::{Event, Sse},
    routing::post,
    Json, Router,
};
use crate::services::config_service::ConfigService;
use futures::stream::{self, Stream};
use tokio_stream::StreamExt;
use serde_json::{json, Value};
use std::{convert::Infallible, sync::Arc, time::Duration};

/// Representation of an MCP Tool
#[derive(serde::Serialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub fn mcp_routes(service: Arc<ConfigService>) -> Router {
    Router::new()
        .route("/mcp/sse", axum::routing::get(mcp_sse_handler))
        .route("/mcp/call", post(call_tool_handler))
        .with_state(service)
}

async fn mcp_sse_handler(
    State(_service): State<Arc<ConfigService>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Basic SSE implementation for MCP initialization
    let stream = stream::repeat_with(|| {
        Event::default().data(json!({
            "type": "tools/list",
            "tools": list_available_tools()
        }).to_string())
    })
    .map(Ok)
    .throttle(Duration::from_secs(60));

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

fn list_available_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "add_field".to_string(),
            description: "Add a field to a page configuration".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "page_id": { "type": "string" },
                    "range_code": { "type": "string", "enum": ["view", "create", "update", "list"] },
                    "field": { "type": "object" }
                },
                "required": ["page_id", "range_code", "field"]
            }),
        },
        McpTool {
            name: "sync_to_mongo".to_string(),
            description: "Sync current cached configuration to MongoDB".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "page_id": { "type": "string" }
                },
                "required": ["page_id"]
            }),
        }
    ]
}

#[derive(serde::Deserialize)]
pub struct ToolCallRequest {
    pub name: String,
    pub arguments: Value,
}

async fn call_tool_handler(
    State(service): State<Arc<ConfigService>>,
    Json(payload): Json<ToolCallRequest>,
) -> impl axum::response::IntoResponse {
    match payload.name.as_str() {
        "add_field" => {
            let page_id = payload.arguments.get("page_id").and_then(|v| v.as_str()).unwrap_or("");
            let range_code = payload.arguments.get("range_code").and_then(|v| v.as_str()).unwrap_or("");
            let field = payload.arguments.get("field").cloned().unwrap_or(json!({}));
            
            match service.add_field(page_id, range_code, field).await {
                Ok(_) => Json(json!({ "status": "success", "message": "Field added via MCP" })),
                Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
            }
        },
        "sync_to_mongo" => {
            let page_id = payload.arguments.get("page_id").and_then(|v| v.as_str()).unwrap_or("");
            match service.sync_to_mongo(page_id).await {
                Ok(_) => Json(json!({ "status": "success", "message": "Synced via MCP" })),
                Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
            }
        },
        _ => Json(json!({ "status": "error", "message": "Tool not found" })),
    }
}
