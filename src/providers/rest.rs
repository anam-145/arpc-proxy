use axum::{
    body::Body,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
};
use reqwest::Client;

use crate::error::AppError;

pub async fn forward(
    client: &Client,
    base_url: &str,
    method: Method,
    path: &str,
    query: Option<&str>,
    api_key: Option<&str>,
    body: Option<Body>,
) -> Result<Response, AppError> {
    let url = build_url(base_url, path, query, api_key);

    let mut request = match method {
        Method::GET => client.get(&url),
        Method::POST => client.post(&url),
        Method::PUT => client.put(&url),
        Method::DELETE => client.delete(&url),
        Method::PATCH => client.patch(&url),
        _ => {
            return Err(AppError::ProtocolMismatch(format!(
                "Unsupported method: {}",
                method
            )))
        }
    };

    if let Some(body) = body {
        let bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .map_err(|e| AppError::ParseError(e.to_string()))?;
        request = request.body(bytes);
    }

    let response = request
        .send()
        .await
        .map_err(|e| AppError::ProviderError(e.to_string()))?;

    let status = StatusCode::from_u16(response.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let headers = response.headers().clone();
    let body = response
        .bytes()
        .await
        .map_err(|e| AppError::ParseError(e.to_string()))?;

    let mut builder = Response::builder().status(status);

    if let Some(content_type) = headers.get("content-type") {
        builder = builder.header("content-type", content_type);
    }

    Ok(builder
        .body(Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()))
}

fn build_url(base_url: &str, path: &str, query: Option<&str>, api_key: Option<&str>) -> String {
    let base = if path.is_empty() {
        base_url.to_string()
    } else {
        format!("{}/{}", base_url, path)
    };

    match (query, api_key) {
        (Some(q), Some(key)) => format!("{}?{}&apikey={}", base, q, key),
        (Some(q), None) => format!("{}?{}", base, q),
        (None, Some(key)) => format!("{}?apikey={}", base, key),
        (None, None) => base,
    }
}
