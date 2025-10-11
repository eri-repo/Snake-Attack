use crate::functions::{check_user, create_user, delete_user, get_user, update_user};
use crate::swagger_config::ApiDoc;
use axum::routing::put;
use axum::{routing::{get, post}, Router};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod schema;
mod model;
mod swagger_config;
mod functions;
mod generate_username;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let app = Router::new()
        .route("/create_user", post(create_user))
        .route("/update_user", put(update_user))
        .route("/delete_user", put(delete_user))
        .route("/users/{wallet_address}", get(get_user))
        .route("/check_user", post(check_user))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(pool)
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any));

    eprintln!("Server running on http://0.0.0.0:8080");

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


// cargo run --package rust_starknake --bin starknake 

// curl -X POST http://127.0.0.1:8080/create_user \                                                                                                ✔  system 
// -H "Content-Type: application/json" \
// -d '{"wallet_address": "0x354624546fdfgnk35432"}'

// curl -X GET http://127.0.0.1:8080/users/0x354624546fdfgnk35432

// psql -U postgres -d rustie -c "\d users"