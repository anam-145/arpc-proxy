use axum::{
    middleware,
    routing::{any, get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use arpc_proxy::{auth::auth_middleware, config::Settings, handlers, state::AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settings = Settings::load().expect("Failed to load settings");
    let addr = settings.server_addr();
    let auth_enabled = settings.auth.enabled;

    tracing::info!("Configuration loaded");
    tracing::info!("Server: {}", addr);
    tracing::info!(
        "Authentication: {}",
        if auth_enabled { "enabled" } else { "disabled" }
    );

    for (id, chain) in &settings.chains {
        let mut protocols = Vec::new();
        if chain.mainnet.has_jsonrpc() {
            protocols.push("jsonrpc");
        }
        if chain.mainnet.has_rest() {
            protocols.push("rest");
        }
        let networks: Vec<&str> = std::iter::once("mainnet")
            .chain(chain.testnets.keys().map(|s| s.as_str()))
            .collect();
        tracing::info!(
            "Chain: {} ({}) - {}",
            id,
            protocols.join("+"),
            networks.join(", ")
        );
    }

    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(settings.database.max_connections)
        .connect(&settings.database.url)
        .await
        .expect("Failed to connect to database");
    tracing::info!("Database connected");

    let state = AppState::new(settings.clone(), pool);

    state
        .api_key_repo
        .init()
        .await
        .expect("Failed to initialize database tables");
    tracing::info!("Database tables initialized");

    if auth_enabled {
        let repo = state.api_key_repo.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // 1시간마다
            loop {
                interval.tick().await;
                match repo.cleanup_expired().await {
                    Ok(count) if count > 0 => {
                        tracing::info!("Cleaned up {} expired API keys", count);
                    }
                    Err(e) => {
                        tracing::error!("Failed to cleanup expired keys: {:?}", e);
                    }
                    _ => {}
                }
            }
        });
    }

    let app = if auth_enabled {
        let protected_routes = Router::new()
            .route("/{chain}", any(handlers::proxy::proxy_mainnet))
            .route("/{chain}/{*path}", any(handlers::proxy::proxy_with_path))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

        Router::new()
            .route("/health", get(handlers::health::health_check))
            .route("/chains", get(handlers::chain::list_chains))
            .route("/auth/register", post(handlers::auth::register))
            .merge(protected_routes)
            .layer(TraceLayer::new_for_http())
            .with_state(state)
    } else {
        Router::new()
            .route("/health", get(handlers::health::health_check))
            .route("/chains", get(handlers::chain::list_chains))
            .route("/auth/register", post(handlers::auth::register))
            .route("/{chain}", any(handlers::proxy::proxy_mainnet))
            .route("/{chain}/{*path}", any(handlers::proxy::proxy_with_path))
            .layer(TraceLayer::new_for_http())
            .with_state(state)
    };

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
