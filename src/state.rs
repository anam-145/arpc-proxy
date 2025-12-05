use reqwest::Client;

use crate::config::Settings;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub http_client: Client,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            http_client: Client::new(),
        }
    }
}
