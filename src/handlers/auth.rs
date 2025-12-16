use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::auth::model::{ApiKey, RegisterRequest, RegisterResponse};

type HmacSha256 = Hmac<Sha256>;

fn verify_signature(device_id: &str, timestamp: i64, signature: &str, client_secret: &str) -> bool {
    let message = format!("{}{}", device_id, timestamp);

    let Ok(mut mac) = HmacSha256::new_from_slice(client_secret.as_bytes()) else {
        return false;
    };

    mac.update(message.as_bytes());

    let Ok(expected_signature) = hex::decode(signature) else {
        return false;
    };

    mac.verify_slice(&expected_signature).is_ok()
}

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

    if let Some(ref client_secret) = state.settings.auth.client_secret {
        let timestamp = match payload.timestamp {
            Some(ts) => ts,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": {
                            "code": -32005,
                            "message": "timestamp is required"
                        }
                    })),
                )
                    .into_response();
            }
        };

        let signature = match payload.signature.as_ref() {
            Some(sig) => sig,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": {
                            "code": -32005,
                            "message": "signature is required"
                        }
                    })),
                )
                    .into_response();
            }
        };

        let now = Utc::now().timestamp();
        let tolerance = state.settings.auth.timestamp_tolerance_secs;
        if (now - timestamp).abs() > tolerance {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": -32003,
                        "message": "Request timestamp expired"
                    }
                })),
            )
                .into_response();
        }

        if !verify_signature(&payload.device_id, timestamp, signature, client_secret) {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": -32003,
                        "message": "Invalid signature"
                    }
                })),
            )
                .into_response();
        }
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
