use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{Method, StatusCode},
    middleware,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use futures_util::stream::{self, Stream, StreamExt};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

mod auth;
mod protocol;
mod session;
mod tools;

use protocol::{JsonRpcRequest, JsonRpcResponse};
use session::{SseEvent, SessionManager};
use tools::{handle_tool_call, list_tools, McpContext};

#[derive(Clone)]
struct AppState {
    session_mgr: SessionManager,
    mcp_ctx: Arc<McpContext>,
}

#[derive(Deserialize)]
struct MessageQuery {
    session_id: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ice_data_mcp=debug".parse().unwrap()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://icedata:icedata@localhost:5432/icedata".into());

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let nhl_client = Arc::new(ice_data_nhl::client::NhlClient::new(60));

    let ai_service = ice_data_ai::AiService::new(pool.clone());

    let mcp_ctx = Arc::new(McpContext {
        pool,
        nhl_client,
        ai_service,
    });

    let state = AppState {
        session_mgr: SessionManager::new(),
        mcp_ctx,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/mcp/sse", get(sse_handler))
        .route("/mcp/message", post(message_handler))
        .layer(middleware::from_fn(auth::api_key_auth))
        .layer(cors)
        .with_state(state);

    let addr: SocketAddr = std::env::var("MCP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3001".into())
        .parse()
        .expect("Invalid MCP_ADDR");

    info!("MCP server starting on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// SSE endpoint — client opens a long-lived SSE connection
async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, std::io::Error>>> {
    let (session_id, rx) = state.session_mgr.create_session().await;
    info!(session_id, "SSE client connected");

    let endpoint_event = Event::default()
        .event("endpoint")
        .data(format!("/mcp/message?session_id={session_id}"));

    let endpoint_stream = stream::once(async move { Ok::<_, std::io::Error>(endpoint_event) });

    let mgr = state.session_mgr.clone();
    let event_stream = stream::unfold(
        (rx, session_id.clone(), mgr),
        |(mut rx, sid, mgr)| async move {
            loop {
                match rx.recv().await {
                    Ok(SseEvent::Message(data)) => {
                        let event = Event::default().data(data).event("message");
                        let new_rx = mgr.subscribe(&sid).await;
                        if let Some(nrx) = new_rx {
                            return Some((Ok(event), (nrx, sid, mgr)));
                        }
                        return None;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!(session_id = sid, "SSE client disconnected");
                        mgr.remove_session(&sid).await;
                        return None;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(session_id = sid, dropped = n, "SSE stream lagged, skipping");
                        continue;
                    }
                }
            }
        },
    );

    let stream = endpoint_stream.chain(event_stream);

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// POST endpoint — receives JSON-RPC requests from the client
async fn message_handler(
    State(state): State<AppState>,
    Query(query): Query<MessageQuery>,
    Json(req): Json<JsonRpcRequest>,
) -> (StatusCode, Json<JsonRpcResponse>) {
    let session_id = &query.session_id;

    if req.jsonrpc != "2.0" {
        return (
            StatusCode::BAD_REQUEST,
            Json(JsonRpcResponse::error(req.id, -32600, "Invalid Request: jsonrpc must be '2.0'")),
        );
    }

    let (status, resp) = match req.method.as_str() {
        "tools/list" => {
            let tools = list_tools();
            (StatusCode::OK, JsonRpcResponse::success(req.id, serde_json::json!({ "tools": tools })))
        }
        "tools/call" => {
            let params = req.params.unwrap_or(Value::Null);
            let tool_name = match params.get("name").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => return (
                    StatusCode::BAD_REQUEST,
                    Json(JsonRpcResponse::error(req.id, -32602, "Invalid params: missing 'name'")),
                ),
            };
            let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

            match handle_tool_call(&state.mcp_ctx, tool_name, &arguments).await {
                Ok(result) => {
                    let resp = JsonRpcResponse::success(req.id, result);
                    let data = serde_json::to_string(&resp).unwrap_or_default();
                    state.session_mgr.send_to_session(session_id, SseEvent::Message(data)).await;
                    (StatusCode::OK, resp)
                }
                Err(err) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, JsonRpcResponse::error(req.id, -32000, err))
                }
            }
        }
        "tools/ping" => {
            (StatusCode::OK, JsonRpcResponse::success(req.id, serde_json::json!({ "status": "ok" })))
        }
        _ => {
            (StatusCode::NOT_FOUND, JsonRpcResponse::error(req.id, -32601, "Method not found"))
        }
    };

    (status, Json(resp))
}
