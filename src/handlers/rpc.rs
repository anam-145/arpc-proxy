use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};

use crate::{error::AppError, models::rpc::RpcRequest, providers::jsonrpc, state::AppState};

pub async fn proxy_rpc(
    State(state): State<AppState>,
    Path(chain): Path<String>,
    Json(payload): Json<RpcRequest>,
) -> Response {
    tracing::info!(
        chain = %chain,
        method = %payload.method,
        id = %payload.id,
        "Incoming JSON-RPC request"
    );

    let chain_config = match state.settings.get_chain(&chain) {
        Some(cfg) => cfg,
        None => {
            return AppError::ChainNotFound(chain).into_response();
        }
    };

    if !chain_config.is_jsonrpc() {
        return AppError::ProtocolMismatch(format!(
            "Chain '{}' is not a JSON-RPC chain. Use /rest/{}/<path> instead.",
            chain, chain
        ))
        .into_response();
    }

    match jsonrpc::forward(&state.http_client, chain_config, &payload).await {
        Ok(response) => response,
        Err(err) => {
            tracing::error!(error = ?err, "JSON-RPC proxy failed");
            err.into_response()
        }
    }
}
