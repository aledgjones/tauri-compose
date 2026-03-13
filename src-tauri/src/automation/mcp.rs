use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use super::api::AutomationApiState;
use super::registry;

const JSONRPC_VERSION: &str = "2.0";
const MCP_PROTOCOL_VERSION: &str = "2025-06-18";
const SERVER_NAME: &str = "solo-compose";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn handle_post(
    State(state): State<AutomationApiState>,
    Json(request): Json<Value>,
) -> Response {
    match handle_request(request, &state).await {
        McpHttpResponse::Json(payload) => {
            with_protocol_header(StatusCode::OK, Json(payload)).into_response()
        }
        McpHttpResponse::Accepted => {
            let mut response = StatusCode::ACCEPTED.into_response();
            response.headers_mut().insert(
                "MCP-Protocol-Version",
                HeaderValue::from_static(MCP_PROTOCOL_VERSION),
            );
            response
        }
        McpHttpResponse::Error(status, payload) => {
            with_protocol_header(status, Json(payload)).into_response()
        }
    }
}

pub async fn handle_get() -> Response {
    with_protocol_header(
        StatusCode::METHOD_NOT_ALLOWED,
        Json(json!({
            "jsonrpc": JSONRPC_VERSION,
            "error": {
                "code": -32000,
                "message": "This MCP endpoint does not expose an SSE stream."
            }
        })),
    )
    .into_response()
}

pub async fn handle_delete() -> Response {
    StatusCode::METHOD_NOT_ALLOWED.into_response()
}

enum McpHttpResponse {
    Json(Value),
    Accepted,
    Error(StatusCode, Value),
}

async fn handle_request(request: Value, state: &AutomationApiState) -> McpHttpResponse {
    let Some(method) = request.get("method").and_then(Value::as_str) else {
        return McpHttpResponse::Error(
            StatusCode::BAD_REQUEST,
            jsonrpc_error(Value::Null, -32600, "Invalid request: missing method"),
        );
    };

    let id = request.get("id").cloned();

    match method {
        "initialize" => {
            let Some(id) = id else {
                return McpHttpResponse::Error(
                    StatusCode::BAD_REQUEST,
                    jsonrpc_error(
                        Value::Null,
                        -32600,
                        "Initialize requests must include an id",
                    ),
                );
            };

            McpHttpResponse::Json(jsonrpc_success(
                id,
                json!({
                    "protocolVersion": MCP_PROTOCOL_VERSION,
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": SERVER_NAME,
                        "version": SERVER_VERSION
                    }
                }),
            ))
        }
        "notifications/initialized" => McpHttpResponse::Accepted,
        "ping" => match id {
            Some(id) => McpHttpResponse::Json(jsonrpc_success(id, json!({}))),
            None => McpHttpResponse::Accepted,
        },
        "tools/list" => {
            let Some(id) = id else {
                return McpHttpResponse::Error(
                    StatusCode::BAD_REQUEST,
                    jsonrpc_error(
                        Value::Null,
                        -32600,
                        "tools/list requests must include an id",
                    ),
                );
            };

            McpHttpResponse::Json(jsonrpc_success(id, json!({ "tools": tool_definitions() })))
        }
        "tools/call" => {
            let Some(id) = id else {
                return McpHttpResponse::Error(
                    StatusCode::BAD_REQUEST,
                    jsonrpc_error(
                        Value::Null,
                        -32600,
                        "tools/call requests must include an id",
                    ),
                );
            };

            let params = request.get("params").and_then(Value::as_object);
            let Some(tool_name) = params
                .and_then(|params| params.get("name"))
                .and_then(Value::as_str)
            else {
                return McpHttpResponse::Json(jsonrpc_error(
                    id,
                    -32602,
                    "Missing tools/call params.name",
                ));
            };

            let arguments = params
                .and_then(|params| params.get("arguments"))
                .and_then(Value::as_object);

            match handle_tool_call(tool_name, arguments, state).await {
                Ok(result) => McpHttpResponse::Json(jsonrpc_success(id, result)),
                Err(error) => McpHttpResponse::Json(jsonrpc_error(id, -32000, &error)),
            }
        }
        _ => match id {
            Some(id) => McpHttpResponse::Json(jsonrpc_error(
                id,
                -32601,
                &format!("Method not found: {method}"),
            )),
            None => McpHttpResponse::Accepted,
        },
    }
}

fn with_protocol_header<T: IntoResponse>(status: StatusCode, payload: T) -> Response {
    let mut response = (status, payload).into_response();
    response.headers_mut().insert(
        "MCP-Protocol-Version",
        HeaderValue::from_static(MCP_PROTOCOL_VERSION),
    );
    response
}

fn jsonrpc_success(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id,
        "result": result
    })
}

fn jsonrpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

const GET_STARTED_TOOL_NAME: &str = "compose_get_started";
const DESCRIBE_MODEL_TOOL_NAME: &str = "compose_describe_model";
const LIST_ACTIONS_TOOL_NAME: &str = "compose_list_actions";
const CALL_ACTION_TOOL_NAME: &str = "compose_call_action";

fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": LIST_ACTIONS_TOOL_NAME,
            "description": format!(
                "List the dotted automation actions currently exposed by the running Compose app. Call {GET_STARTED_TOOL_NAME} first when you need id semantics, tick rules, and track lookup guidance."
            ),
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
        json!({
            "name": DESCRIBE_MODEL_TOOL_NAME,
            "description": format!(
                "Describe the Compose score data model, id semantics, and structural relationships used by the MCP actions. Use {GET_STARTED_TOOL_NAME} for the recommended first-step workflow and {DESCRIBE_MODEL_TOOL_NAME} when you want the full reference model."
            ),
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
        json!({
            "name": GET_STARTED_TOOL_NAME,
            "description": format!(
                "Get the recommended first-step guide for Compose MCP usage. This is the onboarding entrypoint before {CALL_ACTION_TOOL_NAME} when you need ids, tick rules, track lookup guidance, and safe action ordering."
            ),
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
        json!({
            "name": CALL_ACTION_TOOL_NAME,
            "description": format!(
                "Call a Compose automation action by dotted path using positional JSON arguments. Call {GET_STARTED_TOOL_NAME} first for score-editing tasks so you know which ids, tick rules, and track lookups to use."
            ),
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Dotted action path, for example ui.setTab"
                    },
                    "args": {
                        "type": "array",
                        "description": "Positional arguments to pass to the action",
                        "items": {},
                        "default": []
                    },
                    "timeout_ms": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Optional action timeout in milliseconds"
                    }
                },
                "required": ["name"],
                "additionalProperties": false
            }
        }),
    ]
}

async fn handle_tool_call(
    tool_name: &str,
    arguments: Option<&serde_json::Map<String, Value>>,
    state: &AutomationApiState,
) -> Result<Value, String> {
    match tool_name {
        LIST_ACTIONS_TOOL_NAME => Ok(text_result(json!(registry::action_definitions()))),
        DESCRIBE_MODEL_TOOL_NAME => Ok(text_result(compose_domain_model())),
        GET_STARTED_TOOL_NAME => Ok(text_result(compose_getting_started_guide())),
        CALL_ACTION_TOOL_NAME => call_action(arguments, state).await,
        _ => Err(format!("Unknown tool: {tool_name}")),
    }
}

async fn call_action(
    arguments: Option<&serde_json::Map<String, Value>>,
    state: &AutomationApiState,
) -> Result<Value, String> {
    let arguments = arguments.ok_or_else(|| "Missing arguments object".to_string())?;
    let name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing action name".to_string())?;
    let args = arguments
        .get("args")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let action_definitions = registry::action_definitions();
    let action_definition = action_definitions
        .iter()
        .map(|candidate| {
            serde_json::to_value(candidate)
                .expect("Compose action definitions should always serialize")
        })
        .find(|candidate| candidate.get("name").and_then(Value::as_str) == Some(name))
        .ok_or_else(|| format!("Unknown MCP action: {name}"))?;

    let arg_definitions = action_definition
        .get("args")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Action metadata for {name} is missing args"))?;

    let required_args = arg_definitions
        .iter()
        .filter(|argument| argument.get("required").and_then(Value::as_bool) == Some(true))
        .count();

    if args.len() < required_args {
        let missing_args = arg_definitions
            .iter()
            .filter(|argument| argument.get("required").and_then(Value::as_bool) == Some(true))
            .skip(args.len())
            .filter_map(|argument| argument.get("name").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join(", ");

        return Err(format!(
            "Action {name} requires {required_args} arguments. Missing: {missing_args}"
        ));
    }

    let result = registry::execute_action(&state.app_handle, name, &args).await?;
    Ok(text_result(result))
}

fn text_result(payload: Value) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": serde_json::to_string_pretty(&payload)
                    .expect("Compose MCP payloads should always serialize")
            }
        ]
    })
}

fn compose_domain_model() -> Value {
    json!({
        "summary": "Compose stores musical state in a normalized score graph. Most mutations target one entity collection by numeric id, while ordering is usually tracked separately from by-id lookup maps.",
        "recommendedFirstSteps": [
            format!("Call {GET_STARTED_TOOL_NAME} before {LIST_ACTIONS_TOOL_NAME} or {CALL_ACTION_TOOL_NAME} when you need to create or edit score content."),
            "Use score.lookup.getInstrumentTracks to resolve playable user track_ids before calling score.tone.create.",
            "Use score.lookup.getScoreSlice when you need current runtime ids and structural relationships without mutating the score."
        ],
        "ids": {
            "type": "UniqueId",
            "runtimeType": "number",
            "description": "Entity ids are numeric runtime-generated identifiers. They are meaningful only within the currently loaded score state and should be treated as opaque references rather than stable semantic names.",
            "notes": [
                "Different entity kinds share the same numeric id space.",
                "Ids are safe to pass back into later MCP calls after creation.",
                "Ids should not be inferred from labels like instrument long_name or flow title."
            ]
        },
        "whereThingsLive": [
            "Flow-level tempo, time signature, and key signature entries live on the flow master track.",
            "Clef entries live on the stave master track for each stave in a flow.",
            "Tone entries live on user track_ids inside a stave, not on the master track.",
            "An instrument can exist globally before it produces any flow staves or tracks."
        ],
        "tickSystem": {
            "summary": "Ticks are the timeline unit used for both absolute positions and durations.",
            "baseRule": "A quarter note is 32 ticks.",
            "noteValueTicks": {
                "whole": 128,
                "half": 64,
                "quarter": 32,
                "eighth": 16,
                "sixteenth": 8,
                "thirtySecond": 4,
                "sixtyFourth": 2,
                "hundredTwentyEighth": 1
            }
        }
    })
}

fn compose_getting_started_guide() -> Value {
    let domain_model = compose_domain_model();

    json!({
        "summary": "Start here when you are using Compose only through MCP and need enough context to build or edit score content safely.",
        "recommendedOrder": [
            GET_STARTED_TOOL_NAME,
            LIST_ACTIONS_TOOL_NAME,
            CALL_ACTION_TOOL_NAME
        ],
        "nextTools": {
            "listActions": LIST_ACTIONS_TOOL_NAME,
            "describeModel": DESCRIBE_MODEL_TOOL_NAME,
            "callAction": CALL_ACTION_TOOL_NAME
        },
        "recommendedFirstSteps": domain_model["recommendedFirstSteps"].clone(),
        "ids": domain_model["ids"].clone(),
        "whereThingsLive": domain_model["whereThingsLive"].clone(),
        "tickSystem": domain_model["tickSystem"].clone()
    })
}
