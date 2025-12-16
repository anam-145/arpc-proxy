use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub device_id: String,
    pub api_key: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

impl ApiKey {
    pub fn new(device_id: String, expires_at: Option<DateTime<Utc>>) -> Self {
        let api_key = format!("sk-{}", Uuid::new_v4().to_string().replace("-", ""));

        Self {
            id: Uuid::new_v4(),
            device_id,
            api_key,
            created_at: Utc::now(),
            expires_at,
            is_active: true,
        }
    }

    pub fn is_valid(&self) -> bool {
        if !self.is_active {
            return false;
        }

        if let Some(expires_at) = self.expires_at {
            if expires_at < Utc::now() {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub device_id: String,
    pub timestamp: Option<i64>,
    pub signature: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub api_key: String,
    pub expires_at: Option<DateTime<Utc>>,
}
