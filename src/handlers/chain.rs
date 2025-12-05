use axum::{extract::State, Json};
use serde_json::Value;

use crate::{models::chain::ChainInfo, state::AppState};

pub async fn list_chains(State(state): State<AppState>) -> Json<Value> {
    let chains: Vec<ChainInfo> = state
        .settings
        .chains
        .iter()
        .map(|(id, cfg)| ChainInfo {
            id: id.clone(),
            name: cfg.name.clone(),
            protocol: cfg.protocol.clone(),
        })
        .collect();

    Json(serde_json::json!({ "chains": chains }))
}
