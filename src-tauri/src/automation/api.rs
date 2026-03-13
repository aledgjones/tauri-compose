use std::env;
use std::net::SocketAddr;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use super::mcp;
use super::registry;
use super::{DEFAULT_AUTOMATION_HOST, DEFAULT_AUTOMATION_PORT};

#[derive(Clone)]
pub(crate) struct AutomationApiState {
    pub(crate) app_handle: AppHandle,
}

#[derive(Deserialize)]
struct ActionCallRequest {
    name: String,
    #[serde(default)]
    args: Vec<Value>,
}

pub async fn serve(app_handle: AppHandle) -> Result<(), String> {
    let host =
        env::var("COMPOSE_AUTOMATION_HOST").unwrap_or_else(|_| DEFAULT_AUTOMATION_HOST.to_string());
    let port = env::var("COMPOSE_AUTOMATION_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_AUTOMATION_PORT);

    let address: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|error| format!("Invalid automation api address {host}:{port}: {error}"))?;

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|error| {
            format!("Failed to bind automation api on http://{host}:{port}: {error}")
        })?;

    let router = Router::new()
        .route("/api/v1/automation/health", get(health))
        .route("/api/v1/automation/actions", get(list_actions))
        .route("/api/v1/automation/actions/call", post(call_action))
        .route(
            "/mcp",
            post(mcp::handle_post)
                .get(mcp::handle_get)
                .delete(mcp::handle_delete),
        )
        .with_state(AutomationApiState { app_handle });

    axum::serve(listener, router)
        .await
        .map_err(|error| format!("Automation api server failed: {error}"))
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}

async fn list_actions(State(_state): State<AutomationApiState>) -> Json<Value> {
    Json(json!(registry::action_definitions()))
}

async fn call_action(
    State(state): State<AutomationApiState>,
    Json(request): Json<ActionCallRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    registry::execute_action(&state.app_handle, &request.name, &request.args)
        .await
        .map(Json)
        .map_err(|error| (StatusCode::BAD_REQUEST, Json(json!({ "error": error }))))
}
