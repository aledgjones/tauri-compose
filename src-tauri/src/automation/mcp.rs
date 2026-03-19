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
const ACTION_TOOL_PREFIX: &str = "compose_action_";

fn tool_definitions() -> Vec<Value> {
    let mut tools = vec![
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
                "Get the recommended first-step guide for Compose MCP usage. This is the onboarding entrypoint before calling generated action tools when you need ids, tick rules, track lookup guidance, and safe action ordering."
            ),
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
    ];

    tools.extend(action_tool_definitions());
    tools
}

fn action_tool_definitions() -> Vec<Value> {
    serialized_action_definitions()
        .into_iter()
        .map(|action_definition| {
            let action_name = action_definition
                .get("name")
                .and_then(Value::as_str)
                .expect("Compose action definitions should always include a name");
            let description = action_definition
                .get("description")
                .and_then(Value::as_str)
                .expect("Compose action definitions should always include a description");
            let arg_definitions = action_definition
                .get("args")
                .and_then(Value::as_array)
                .expect("Compose action definitions should always include args");

            let properties = arg_definitions
                .iter()
                .filter_map(|argument| {
                    let arg_name = argument.get("name").and_then(Value::as_str)?;
                    let arg_description = argument.get("description").and_then(Value::as_str)?;

                    Some((
                        arg_name.to_string(),
                        json!({
                            "description": arg_description
                        }),
                    ))
                })
                .collect::<serde_json::Map<String, Value>>();

            let required = arg_definitions
                .iter()
                .filter(|argument| argument.get("required").and_then(Value::as_bool) == Some(true))
                .filter_map(|argument| argument.get("name"))
                .cloned()
                .collect::<Vec<_>>();

            json!({
                "name": action_tool_name(action_name),
                "description": format!(
                    "Invoke Compose action {action_name}. {description}"
                ),
                "inputSchema": {
                    "type": "object",
                    "properties": properties,
                    "required": required,
                    "additionalProperties": false
                }
            })
        })
        .collect()
}

fn serialized_action_definitions() -> Vec<Value> {
    registry::action_definitions()
        .into_iter()
        .map(|action_definition| {
            serde_json::to_value(action_definition)
                .expect("Compose action definitions should always serialize")
        })
        .collect()
}

fn action_tool_name(action_name: &str) -> String {
    format!(
        "{ACTION_TOOL_PREFIX}{}",
        action_name
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '_'
                }
            })
            .collect::<String>()
    )
}

fn find_action_definition_by_tool_name(tool_name: &str) -> Option<Value> {
    serialized_action_definitions()
        .into_iter()
        .find(|action_definition| {
            action_definition
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|action_name| action_tool_name(action_name) == tool_name)
        })
}

async fn handle_tool_call(
    tool_name: &str,
    arguments: Option<&serde_json::Map<String, Value>>,
    state: &AutomationApiState,
) -> Result<Value, String> {
    match tool_name {
        DESCRIBE_MODEL_TOOL_NAME => Ok(text_result(compose_domain_model())),
        GET_STARTED_TOOL_NAME => Ok(text_result(compose_getting_started_guide())),
        _ => {
            if let Some(action_definition) = find_action_definition_by_tool_name(tool_name) {
                call_named_action_tool(arguments, &action_definition, state).await
            } else {
                Err(format!("Unknown tool: {tool_name}"))
            }
        }
    }
}

async fn call_named_action_tool(
    arguments: Option<&serde_json::Map<String, Value>>,
    action_definition: &Value,
    state: &AutomationApiState,
) -> Result<Value, String> {
    let action_name = action_definition
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "Action metadata is missing a name".to_string())?;
    let arg_definitions = action_definition
        .get("args")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Action metadata for {action_name} is missing args"))?;
    let arguments = arguments.ok_or_else(|| {
        format!(
            "Generated MCP tool {} requires an arguments object",
            action_tool_name(action_name)
        )
    })?;

    let allowed_arg_names = arg_definitions
        .iter()
        .filter_map(|argument| argument.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();

    if let Some(unknown_argument) = arguments
        .keys()
        .find(|argument_name| !allowed_arg_names.contains(&argument_name.as_str()))
    {
        return Err(format!(
            "Action {action_name} does not accept argument {unknown_argument}"
        ));
    }

    let mut ordered_args = Vec::with_capacity(arg_definitions.len());
    for argument_definition in arg_definitions {
        let arg_name = argument_definition
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("Action metadata for {action_name} is missing arg names"))?;
        let required = argument_definition
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        match arguments.get(arg_name) {
            Some(value) => ordered_args.push(value.clone()),
            None if required => {
                return Err(format!("Action {action_name} requires argument {arg_name}"));
            }
            None => ordered_args.push(Value::Null),
        }
    }

    let result = registry::execute_action(&state.app_handle, action_name, &ordered_args).await?;
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
            format!("Call {GET_STARTED_TOOL_NAME} before any {ACTION_TOOL_PREFIX}* tool when you need to create or edit score content."),
            "Use score.lookup.getInstrumentTracks to resolve playable user track_ids before calling score.tone.create.",
            "For tuplets, create the container on the playable user track first, then create child tones on that same track with tuplet-local ticks and a tuplet_id.",
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
        "tuplets": {
            "summary": "Tuplet containers live on the host user track, while tuplet child tones use tuplet-local timing on that same track with a tuplet_id.",
            "authoringOrder": [
                "Resolve the playable user track_id with score.lookup.getInstrumentTracks before creating tuplets or tones.",
                "Create the tuplet container on that user track with score.tuplet.create.",
                "Create child tones on the same track with score.tone.create and pass the tuplet_id returned by the container creation call."
            ],
            "timingRules": [
                "The tuplet container tick is a host-track tick.",
                "Tuplet child ticks are local to the tuplet, not absolute host ticks.",
                "Tuplet child durations stay literal in the tuplet-local frame; do not pre-compress them to the host span.",
                "For a 3:2 eighth-note triplet, the host span is 32 ticks while the local child timeline is 48 ticks with child ticks 0, 16, and 32."
            ],
            "ownershipRules": [
                "Tones that start inside the tuplet belong in the tuplet child entries.",
                "Tones that start before the tuplet remain host-owned even if their durations cross the tuplet span.",
                "Crossing host-owned tones keep their literal durations; only the container geometry is compressed on the host track."
            ],
            "example": {
                "label": "3:2 eighth-note triplet in 4/4 starting on beat 2",
                "hostStartTick": 32,
                "hostSpanTicks": 32,
                "localChildTicks": [0, 16, 32],
                "localChildDurations": [16, 16, 16]
            }
        },
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
            format!("{ACTION_TOOL_PREFIX}<action-name>")
        ],
        "nextTools": {
            "describeModel": DESCRIBE_MODEL_TOOL_NAME,
            "actionToolPrefix": ACTION_TOOL_PREFIX
        },
        "recommendedFirstSteps": domain_model["recommendedFirstSteps"].clone(),
        "ids": domain_model["ids"].clone(),
        "whereThingsLive": domain_model["whereThingsLive"].clone(),
        "tuplets": domain_model["tuplets"].clone(),
        "tickSystem": domain_model["tickSystem"].clone()
    })
}
