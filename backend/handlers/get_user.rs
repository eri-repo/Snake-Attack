// use crate::model::{AppState, User, UserResponse};
// use crate::schema::users;
// use axum::extract::{Path, State};
// use axum::http::StatusCode;
// use axum::response::IntoResponse;
// use axum::Json;
// use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
// use std::sync::Arc;
//
// #[utoipa::path(
//     get,
//     path = "/users/{wallet_address}",
//     params(
//         ("wallet_address" = String, Path, description = "Wallet address of the user", example = "0x1234567890abcdef")
//     ),
//     responses(
//         (status = 200, description = "User found", body = UserResponse),
//         (status = 404, description = "User not found", body = String)
//     ),
//     tag = "Users"
// )]
// pub async fn get_user(
//     State(state): State<Arc<AppState>>,
//     Path(wallet_address): Path<String>,
// ) -> impl IntoResponse {
//     let conn = &mut state.db.get().expect("Couldn't get db connection from pool");
//
//     match users::table
//         .filter(users::address.eq(wallet_address))
//         .first::<User>(conn)
//     {
//         Ok(user) => (
//             StatusCode::OK,
//             Json(UserResponse {
//                 id: user.id,
//                 address: user.address,
//                 username: user.username,
//                 position: user.position.unwrap_or(0),
//                 games_played: user.games_played.unwrap_or(0),
//                 is_registered: user.is_registered,
//                 highest_score: user.highest_score.unwrap_or(0),
//                 updated: user.updated,
//                 registered_at: user.registered_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
//                 status: None,
//             }),
//         )
//             .into_response(),
//         Err(_) => (StatusCode::NOT_FOUND, "User not found".to_string()).into_response(),
//     }
// }

use crate::error::ApiError;
use crate::model::{AppState, User, UserResponse};
use crate::schema::users;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;
use utoipa::ToSchema;

#[utoipa::path(
    get,
    path = "/users/{wallet_address}",
    params(
        (
        "wallet_address" = String, Path, description =
        "Wallet address of the user",
        example = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        )
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 400, description = "Invalid input", body = serde_json::Value),
        (status = 404, description = "User not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    tag = "Users"
)]
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(wallet_address): Path<String>,
) -> impl IntoResponse {
    // Validate wallet_address (addresses starts with 0x)
    if !wallet_address.starts_with("0x") {
        return Err(ApiError::Validation(
            "Invalid wallet address: must start with 0x".to_string(),
        ));
    }

    // Get database connection
    let conn = &mut state.db.get().map_err(|e| {
        tracing::error!("Database connection error: {}", e);
        ApiError::DatabaseConnection(e.to_string())
    })?;
    // Query user
    let user = users::table
        .filter(users::address.eq(&wallet_address))
        .select(User::as_select())
        .first(conn)
        .optional()?;

    match user {
        Some(user) => Ok((
            StatusCode::OK,
            Json(UserResponse {
                address: user.address,
                username: user.username,
                position: user.position.unwrap_or(0),
                games_played: user.games_played.unwrap_or(0),
                is_registered: user.is_registered,
                highest_score: user.highest_score.unwrap_or(0),
                updated: user.updated,
                registered_at: user.registered_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
                status: None,
            }),
        )),
        None => Err(ApiError::Database(diesel::result::Error::NotFound)),
    }
}