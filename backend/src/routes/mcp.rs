use crate::{
    db::products,
    error::AppError,
    models::product::{SearchInput, SearchMode},
    state::AppState,
};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use hmac::{Hmac, Mac};
use mongodb::bson::oid::ObjectId;
use serde_json::{Value, json};
use sha2::Sha256;

const VERSIONS: [&str; 3] = ["2025-03-26", "2025-06-18", "2025-11-25"];

fn authorize(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    let token = state
        .config
        .alex_mcp_token
        .as_deref()
        .filter(|v| v.len() >= 32)
        .ok_or(AppError::Unauthorized)?;
    let presented = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;
    let mut expected =
        Hmac::<Sha256>::new_from_slice(b"alex-mcp-auth").map_err(|_| AppError::Data)?;
    expected.update(token.as_bytes());
    let mut actual =
        Hmac::<Sha256>::new_from_slice(b"alex-mcp-auth").map_err(|_| AppError::Data)?;
    actual.update(presented.as_bytes());
    actual
        .verify_slice(&expected.finalize().into_bytes())
        .map_err(|_| AppError::Unauthorized)?;
    if let Some(origin) = headers.get("origin") {
        let origin = origin.to_str().map_err(|_| AppError::AlexScope)?;
        if state.config.public_base_url.as_deref() != Some(origin)
            && state.config.cors_origin.as_deref() != Some(origin)
        {
            return Err(AppError::AlexScope);
        }
    }
    if let Some(version) = headers.get("mcp-protocol-version")
        && !version.to_str().is_ok_and(|v| VERSIONS.contains(&v))
    {
        return Err(AppError::BadRequest);
    }
    Ok(())
}

fn error(id: Value, code: i64, message: &str) -> Response {
    Json(json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})).into_response()
}

fn schema(image: bool) -> Value {
    let mut properties = json!({"inquiry_id":{"type":"string"},"draft_version":{"type":"integer","minimum":1},
        "context_hash":{"type":"string","minLength":64,"maxLength":64},"item_index":{"type":"integer","minimum":0,"maximum":19},
        "branch_id":{"type":"string"},"query":{"type":"string","maxLength":500},"color":{"type":"string"},"size":{"type":"string"},
        "category":{"type":"string"},"texture":{"type":"string"},"page":{"type":"integer","minimum":1,"maximum":100}});
    let mut required = vec![
        "inquiry_id",
        "draft_version",
        "context_hash",
        "item_index",
        "branch_id",
    ];
    if image {
        properties["image_event_id"] = json!({"type":"string"});
        required.push("image_event_id");
    } else {
        properties["mode"] = json!({"type":"string","enum":["code","text","product_url"]});
        required.push("mode");
    }
    json!({"type":"object","additionalProperties":false,"required":required,"properties":properties})
}

pub async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<Value>, JsonRejection>,
) -> Result<Response, AppError> {
    authorize(&state, &headers)?;
    let Json(body) = match body {
        Ok(value) => value,
        Err(_) => return Ok(error(Value::Null, -32700, "Parse error")),
    };
    let id = body.get("id").cloned().unwrap_or(Value::Null);
    if body["jsonrpc"] != "2.0"
        || !body["method"].is_string()
        || !body.is_object()
        || (body.get("id").is_some() && !(id.is_string() || id.is_number()))
    {
        return Ok(error(id, -32600, "Invalid request"));
    }
    if body.get("id").is_none() {
        return Ok(StatusCode::ACCEPTED.into_response());
    }
    let result = match body["method"].as_str().unwrap_or("") {
        "initialize" => {
            let version = body["params"]["protocolVersion"].as_str().unwrap_or("");
            if version.is_empty() {
                return Ok(error(id, -32602, "protocolVersion is required"));
            }
            json!({"protocolVersion":if VERSIONS.contains(&version){version}else{"2025-11-25"},"capabilities":{"tools":{}},
                "serverInfo":{"name":"alex-catalog","version":"0.1.0"},
                "instructions":"Read-only catalog tools scoped to the configured shop and authorized branches. Results require human confirmation in Alex. Tools cannot confirm products, check queues or create orders."})
        }
        "ping" => json!({}),
        "tools/list" => json!({"tools":[
            {"name":"search_products","description":"Search real Alex catalog by exact code, text or an allowed product URL. Returns candidates for one page; requires explicit human confirmation.","inputSchema":schema(false),"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true,"openWorldHint":true}},
            {"name":"find_product_candidates_by_image","description":"Read a visible code or compare an inquiry image with catalog images in the filtered page. Similarity ranks candidates; it does not confirm identity.","inputSchema":schema(true),"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true,"openWorldHint":true}}
        ]}),
        "tools/call" => {
            let name = body["params"]["name"].as_str().unwrap_or("");
            if !["search_products", "find_product_candidates_by_image"].contains(&name) {
                return Ok(error(id, -32602, "Unknown tool"));
            }
            let Some(mut arguments) = body["params"]["arguments"].as_object().cloned() else {
                return Ok(error(id, -32602, "Invalid arguments"));
            };
            let inquiry = arguments
                .remove("inquiry_id")
                .and_then(|value| value.as_str().and_then(|v| ObjectId::parse_str(v).ok()));
            if name == "find_product_candidates_by_image" {
                arguments.insert("mode".into(), json!("image"));
            }
            let input: SearchInput = match serde_json::from_value(Value::Object(arguments)) {
                Ok(input) => input,
                Err(_) => return Ok(error(id, -32602, "Invalid search arguments")),
            };
            let Some(inquiry) = inquiry else {
                return Ok(error(id, -32602, "Invalid inquiry_id"));
            };
            if name == "search_products" && input.mode == SearchMode::Image {
                return Ok(error(id, -32602, "Use the image search tool"));
            }
            match products::search(&state, inquiry, input).await {
                Ok(result) => {
                    json!({"content":[{"type":"text","text":result.to_string()}],"structuredContent":result,"isError":false})
                }
                Err(error) => {
                    json!({"content":[{"type":"text","text":format!("{}: {}",error.code(),error)}],"isError":true})
                }
            }
        }
        _ => return Ok(error(id, -32601, "Method not found")),
    };
    Ok(Json(json!({"jsonrpc":"2.0","id":id,"result":result})).into_response())
}

pub async fn get(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, AppError> {
    authorize(&state, &headers)?;
    Ok(StatusCode::METHOD_NOT_ALLOWED.into_response())
}
