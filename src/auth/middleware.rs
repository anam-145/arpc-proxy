use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::state::AppState;

const API_KEY_HEADER: &str = "X-API-Key";

pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let api_key = request
        .headers()
        .get(API_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or(AuthError::MissingApiKey)?;

    let key_record = state
        .api_key_repo
        .find_by_api_key(api_key)
        .await
        .map_err(|_| AuthError::InternalError)?
        .ok_or(AuthError::InvalidApiKey)?;

    if !key_record.is_valid() {
        return Err(AuthError::ExpiredApiKey);
    }

    Ok(next.run(request).await)
}

#[derive(Debug)]
pub enum AuthError {
    MissingApiKey,
    InvalidApiKey,
    ExpiredApiKey,
    InternalError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AuthError::MissingApiKey => (
                StatusCode::UNAUTHORIZED,
                -32003,
                "Missing API Key. Include 'X-API-Key' header.",
            ),
            AuthError::InvalidApiKey => (StatusCode::UNAUTHORIZED, -32003, "Invalid API Key"),
            AuthError::ExpiredApiKey => (StatusCode::UNAUTHORIZED, -32003, "API Key has expired"),
            AuthError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                -32603,
                "Internal server error",
            ),
        };

        let body = serde_json::json!({
            "error": {
                "code": code,
                "message": message
            }
        });

        (status, Json(body)).into_response()
    }
}
