use reqwest::Client;
use sqlx::PgPool;

use crate::auth::ApiKeyRepository;
use crate::config::Settings;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub http_client: Client,
    pub api_key_repo: ApiKeyRepository,
}

impl AppState {
    pub fn new(settings: Settings, pool: PgPool) -> Self {
        Self {
            settings,
            http_client: Client::new(),
            api_key_repo: ApiKeyRepository::new(pool),
        }
    }
}
