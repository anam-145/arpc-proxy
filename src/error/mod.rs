use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::models::rpc::RpcErrorResponse;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Unknown chain: {0}")]
    ChainNotFound(String),

    #[error("Protocol mismatch: {0}")]
    ProtocolMismatch(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl AppError {
    pub fn error_code(&self) -> i64 {
        match self {
            AppError::ChainNotFound(_) => -32001,
            AppError::ProtocolMismatch(_) => -32002,
            AppError::ProviderError(_) => -32603,
            AppError::ParseError(_) => -32700,
            AppError::InternalError(_) => -32603,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::ChainNotFound(_) => StatusCode::BAD_REQUEST,
            AppError::ProtocolMismatch(_) => StatusCode::BAD_REQUEST,
            AppError::ProviderError(_) => StatusCode::BAD_GATEWAY,
            AppError::ParseError(_) => StatusCode::BAD_REQUEST,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn to_rpc_response(&self, id: u64) -> RpcErrorResponse {
        RpcErrorResponse::new(self.error_code(), self.to_string(), id)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = serde_json::json!({
            "error": {
                "code": self.error_code(),
                "message": self.to_string()
            }
        });

        (status, Json(body)).into_response()
    }
}
