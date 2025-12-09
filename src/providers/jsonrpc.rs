use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use reqwest::Client;

use crate::{error::AppError, models::rpc::RpcRequest};

pub async fn forward(
    client: &Client,
    url: &str,
    request: &RpcRequest,
) -> Result<Response, AppError> {
    let response = client
        .post(url)
        .json(request)
        .send()
        .await
        .map_err(|e| AppError::ProviderError(e.to_string()))?;

    let status = StatusCode::from_u16(response.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let body = response
        .bytes()
        .await
        .map_err(|e| AppError::ParseError(e.to_string()))?;

    tracing::info!("JSON-RPC request successful");

    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()))
}
