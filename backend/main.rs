use crate::handlers::check_user::check_user;
use crate::handlers::create_user::create_user;
use crate::handlers::delete_user::delete_user;
use crate::handlers::get_user::get_user;
use crate::handlers::leaderboard::leaderboard;
use crate::handlers::update_score::update_score;
use crate::handlers::update_user::update_user;
use crate::logging::setup_logging;
use crate::model::AppState;
use crate::swagger_config::ApiDoc;
use axum::http::HeaderValue;
use axum::routing::{delete, put};
use axum::{
    routing::{get, post},
    Router,
};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tracing::log::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod error;
mod handlers;
mod logging;
mod model;
mod schema;
mod swagger_config;

mod generate_username;

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    dotenv().ok();
    // initialize tracing with environment-based log level (default: DEBUG)
    setup_logging();

    info!("Starting Snake Attack application");


    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let cors_origins = env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:5173".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    info!("cors origins: {:?}", cors_origins);

    let pool = r2d2::Pool::builder()
        .build(ConnectionManager::<PgConnection>::new(database_url))
        .expect("Failed to create pool.");

    let state = Arc::new(AppState { db: pool });

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(
            cors_origins
                .iter()
                .map(|s| s.parse::<HeaderValue>())
                .collect::<Result<Vec<_>, _>>()?,
        );

    let app = Router::new()
        .route("/create_user", post(create_user))
        .route("/update_user", put(update_user))
        .route("/delete_user", delete(delete_user))
        .route("/update_score", put(update_score))
        .route("/users/{wallet_address}", get(get_user))
        .route("/leaderboard", get(leaderboard))
        .route("/check_user", post(check_user))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors)
        .with_state(state);


    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await?;

    info!("Server running on http://{}", addr);
    info!(
        "Swagger UI available at http://{}/swagger-ui/index.html#/",
        addr
    );

    // serve graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C, shutting down"),
        _ = terminate => info!("Received SIGTERM, shutting down"),
    }
}
