use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use arpc_proxy::{config::Settings, handlers, state::AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let settings = Settings::load().expect("Failed to load settings");
    let addr = settings.server_addr();

    tracing::info!("   Configuration loaded");
    tracing::info!("   Server: {}", addr);
    for (id, chain) in &settings.chains {
        tracing::info!("   Chain: {} ({}) - {}", id, chain.protocol, chain.base_url);
    }

    let state = AppState::new(settings);

    let app = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/chains", get(handlers::chain::list_chains))
        .route("/rpc/{chain}", post(handlers::rpc::proxy_rpc))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("   Server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
