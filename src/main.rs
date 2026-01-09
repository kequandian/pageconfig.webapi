mod models;
mod repositories;
mod services;
mod handlers;

use crate::repositories::mongo_repo::MongoRepository;
use crate::repositories::redis_repo::RedisRepository;
use crate::services::config_service::ConfigService;
use axum::Router;
use std::sync::Arc;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".into());
    let mongo_url = std::env::var("MONGO_URL").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "pageconfig_db".into());

    // Initialize repositories
    let redis_repo = Arc::new(RedisRepository::new(&redis_url)?);
    let mongo_repo = Arc::new(MongoRepository::new(&mongo_url, &db_name).await?);

    // Initialize service
    let config_service = Arc::new(ConfigService::new(redis_repo, mongo_repo));

    // Combine routes
    let app = Router::new()
        .merge(handlers::rest::config_routes(config_service.clone()))
        .merge(handlers::mcp::mcp_routes(config_service.clone()));

    // Run server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
