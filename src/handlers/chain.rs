use axum::{extract::State, Json};
use serde_json::Value;

use crate::{models::chain::ChainInfo, state::AppState};

pub async fn list_chains(State(state): State<AppState>) -> Json<Value> {
    let chains: Vec<ChainInfo> = state
        .settings
        .chains
        .iter()
        .map(|(id, cfg)| {
            let mut protocols = Vec::new();
            if cfg.mainnet.has_jsonrpc() {
                protocols.push("jsonrpc".to_string());
            }
            if cfg.mainnet.has_rest() {
                protocols.push("rest".to_string());
            }
            ChainInfo {
                id: id.clone(),
                name: cfg.name.clone(),
                protocols,
            }
        })
        .collect();

    Json(serde_json::json!({ "chains": chains }))
}
