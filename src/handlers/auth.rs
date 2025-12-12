use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{Duration, Utc};

use crate::{
    auth::model::{ApiKey, RegisterRequest, RegisterResponse},
    state::AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    if payload.device_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": {
                    "code": -32005,
                    "message": "device_id is required"
                }
            })),
        )
            .into_response();
    }

    if let Err(e) = state
        .api_key_repo
        .deactivate_by_device_id(&payload.device_id)
        .await
    {
        tracing::error!("Failed to deactivate existing keys: {:?}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "code": -32603,
                    "message": "Failed to process request"
                }
            })),
        )
            .into_response();
    }

    let expires_at = state
        .settings
        .auth
        .key_expiration_days
        .map(|days| Utc::now() + Duration::days(days));

    let api_key = ApiKey::new(payload.device_id.clone(), expires_at);

    if let Err(e) = state.api_key_repo.create(&api_key).await {
        tracing::error!("Failed to create API key: {:?}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "code": -32603,
                    "message": "Failed to create API key"
                }
            })),
        )
            .into_response();
    }

    tracing::info!(
        device_id = %payload.device_id,
        "API Key issued"
    );

    let response = RegisterResponse {
        api_key: api_key.api_key,
        expires_at: api_key.expires_at,
    };

    (StatusCode::OK, Json(response)).into_response()
}
