use axum::{
    body::Body,
    extract::{Path, State},
    http::{Method, Uri},
    response::{IntoResponse, Response},
};

use crate::{
    error::AppError,
    models::rpc::RpcRequest,
    providers::{jsonrpc, rest},
    state::AppState,
};

pub async fn proxy_mainnet(
    State(state): State<AppState>,
    method: Method,
    Path(chain): Path<String>,
    uri: Uri,
    body: Body,
) -> Response {
    let chain_config = match state.settings.get_chain(&chain) {
        Some(cfg) => cfg,
        None => return AppError::ChainNotFound(chain).into_response(),
    };

    let network_config = &chain_config.mainnet;

    if method == Method::POST && network_config.has_jsonrpc() {
        handle_jsonrpc(
            &state,
            &chain,
            network_config.jsonrpc_url.as_ref().unwrap(),
            body,
        )
        .await
    } else if network_config.has_rest() {
        let query = uri.query();
        handle_rest(
            &state,
            &chain,
            network_config.rest_url.as_ref().unwrap(),
            network_config.api_key.as_deref(),
            method,
            "",
            query,
            body,
        )
        .await
    } else {
        AppError::ProtocolMismatch(format!(
            "Chain '{}' mainnet has no supported endpoints",
            chain
        ))
        .into_response()
    }
}

pub async fn proxy_with_path(
    State(state): State<AppState>,
    method: Method,
    Path((chain, path)): Path<(String, String)>,
    uri: Uri,
    body: Body,
) -> Response {
    let chain_config = match state.settings.get_chain(&chain) {
        Some(cfg) => cfg,
        None => return AppError::ChainNotFound(chain).into_response(),
    };

    let first_segment = path.split('/').next().unwrap_or("");

    if let Some(testnet_config) = chain_config.testnets.get(first_segment) {
        let rest_path = path.splitn(2, '/').nth(1).unwrap_or("");

        if method == Method::POST && rest_path.is_empty() && testnet_config.has_jsonrpc() {
            return handle_jsonrpc(
                &state,
                &chain,
                testnet_config.jsonrpc_url.as_ref().unwrap(),
                body,
            )
            .await;
        }

        if testnet_config.has_rest() {
            let query = uri.query();
            return handle_rest(
                &state,
                &chain,
                testnet_config.rest_url.as_ref().unwrap(),
                testnet_config.api_key.as_deref(),
                method,
                rest_path,
                query,
                body,
            )
            .await;
        }

        return AppError::ProtocolMismatch(format!(
            "Network '{}' has no supported endpoints for this request",
            first_segment
        ))
        .into_response();
    }

    let network_config = &chain_config.mainnet;

    if method == Method::POST && network_config.has_jsonrpc() {
        if chain_config.testnets.contains_key(&path) {
            if let Some(testnet) = chain_config.testnets.get(&path) {
                if testnet.has_jsonrpc() {
                    return handle_jsonrpc(
                        &state,
                        &chain,
                        testnet.jsonrpc_url.as_ref().unwrap(),
                        body,
                    )
                    .await;
                }
            }
        }
    }

    if network_config.has_rest() {
        let query = uri.query();
        return handle_rest(
            &state,
            &chain,
            network_config.rest_url.as_ref().unwrap(),
            network_config.api_key.as_deref(),
            method,
            &path,
            query,
            body,
        )
        .await;
    }

    AppError::ProtocolMismatch(format!("Chain '{}' mainnet has no REST endpoint", chain))
        .into_response()
}

async fn handle_jsonrpc(state: &AppState, chain: &str, url: &str, body: Body) -> Response {
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(e) => return AppError::ParseError(e.to_string()).into_response(),
    };

    let payload: RpcRequest = match serde_json::from_slice(&bytes) {
        Ok(p) => p,
        Err(e) => return AppError::ParseError(e.to_string()).into_response(),
    };

    tracing::info!(
        chain = %chain,
        method = %payload.method,
        id = %payload.id,
        "Incoming JSON-RPC request"
    );

    match jsonrpc::forward(&state.http_client, url, &payload).await {
        Ok(response) => response,
        Err(err) => {
            tracing::error!(error = ?err, "JSON-RPC proxy failed");
            err.into_response()
        }
    }
}

async fn handle_rest(
    state: &AppState,
    chain: &str,
    base_url: &str,
    api_key: Option<&str>,
    method: Method,
    path: &str,
    query: Option<&str>,
    body: Body,
) -> Response {
    tracing::info!(
        chain = %chain,
        method = %method,
        path = %path,
        "Incoming REST request"
    );

    let body = if method == Method::GET || method == Method::DELETE {
        None
    } else {
        Some(body)
    };

    match rest::forward(
        &state.http_client,
        base_url,
        method,
        path,
        query,
        api_key,
        body,
    )
    .await
    {
        Ok(response) => response,
        Err(err) => {
            tracing::error!(error = ?err, "REST proxy failed");
            err.into_response()
        }
    }
}
