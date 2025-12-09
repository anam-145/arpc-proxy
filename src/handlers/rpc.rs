use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};

use crate::{error::AppError, models::rpc::RpcRequest, providers::jsonrpc, state::AppState};

pub async fn proxy_rpc_mainnet(
    State(state): State<AppState>,
    Path(chain): Path<String>,
    Json(payload): Json<RpcRequest>,
) -> Response {
    proxy_rpc_internal(&state, &chain, None, payload).await
}

pub async fn proxy_rpc_testnet(
    State(state): State<AppState>,
    Path((chain, network)): Path<(String, String)>,
    Json(payload): Json<RpcRequest>,
) -> Response {
    proxy_rpc_internal(&state, &chain, Some(&network), payload).await
}

async fn proxy_rpc_internal(
    state: &AppState,
    chain: &str,
    network: Option<&str>,
    payload: RpcRequest,
) -> Response {
    tracing::info!(
        chain = %chain,
        network = network.unwrap_or("mainnet"),
        method = %payload.method,
        id = %payload.id,
        "Incoming JSON-RPC request"
    );

    let chain_config = match state.settings.get_chain(chain) {
        Some(cfg) => cfg,
        None => {
            return AppError::ChainNotFound(chain.to_string()).into_response();
        }
    };

    if !chain_config.is_jsonrpc() {
        return AppError::ProtocolMismatch(format!(
            "Chain '{}' is not a JSON-RPC chain.",
            chain
        ))
        .into_response();
    }

    let network_config = match chain_config.get_network(network) {
        Some(cfg) => cfg,
        None => {
            return AppError::NetworkNotFound(network.unwrap_or("mainnet").to_string())
                .into_response();
        }
    };

    match jsonrpc::forward(&state.http_client, network_config, &payload).await {
        Ok(response) => response,
        Err(err) => {
            tracing::error!(error = ?err, "JSON-RPC proxy failed");
            err.into_response()
        }
    }
}
